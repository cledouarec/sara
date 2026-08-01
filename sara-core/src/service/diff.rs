//! Diff service for comparing knowledge graphs.
//!
//! Provides functionality to compute differences between two states of the
//! requirements knowledge graph, supporting Git reference comparisons.

use std::path::PathBuf;

use crate::graph::{GraphDiff, KnowledgeGraphBuilder};
use crate::repository::{GitReader, GitRef};

/// Options for computing a diff between two graph states.
#[derive(Debug, Clone)]
pub struct DiffOptions {
    /// First reference (baseline, e.g., "main", "HEAD~1", commit SHA).
    pub ref1: String,
    /// Second reference (target, e.g., "HEAD", branch name).
    pub ref2: String,
    /// Repository paths to compare.
    pub repositories: Vec<PathBuf>,
    /// Show summary statistics only.
    pub stat: bool,
}

impl DiffOptions {
    /// Creates new diff options.
    pub fn new(ref1: impl Into<String>, ref2: impl Into<String>) -> Self {
        Self {
            ref1: ref1.into(),
            ref2: ref2.into(),
            repositories: Vec::new(),
            stat: false,
        }
    }

    /// Sets the repository paths.
    pub fn with_repositories(mut self, repositories: Vec<PathBuf>) -> Self {
        self.repositories = repositories;
        self
    }

    /// Adds a repository path.
    pub fn add_repository(mut self, path: PathBuf) -> Self {
        self.repositories.push(path);
        self
    }

    /// Sets whether to show only summary statistics.
    pub fn with_stat(mut self, stat: bool) -> Self {
        self.stat = stat;
        self
    }
}

/// Errors that can occur during diff operations.
#[derive(Debug, thiserror::Error)]
pub enum DiffError {
    /// Failed to parse repository.
    #[error("Failed to parse repository {path}: {reason}")]
    ParseError { path: String, reason: String },

    /// Failed to build graph.
    #[error("Failed to build graph: {0}")]
    GraphBuildError(String),

    /// No repository path was configured to compare.
    #[error("No repository path configured: nothing to compare")]
    NoRepositories,

    /// A configured path lies outside any Git repository.
    #[error("No Git repository found for {path}: {reason}")]
    NotAGitRepository { path: String, reason: String },

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result of a diff operation.
#[derive(Debug)]
pub struct DiffResult {
    /// The computed diff.
    pub diff: GraphDiff,
    /// The first reference used.
    pub ref1: String,
    /// The second reference used.
    pub ref2: String,
}

impl DiffResult {
    /// Returns true if there are no changes.
    pub fn is_empty(&self) -> bool {
        self.diff.is_empty()
    }
}

/// Service for computing diffs between knowledge graph states.
#[derive(Debug, Default)]
pub struct DiffService;

impl DiffService {
    /// Creates a new diff service.
    pub fn new() -> Self {
        Self
    }

    /// Computes the diff between two references.
    ///
    /// Each configured path is resolved to its enclosing Git repository and
    /// the scan is scoped to that path, so files outside the configured
    /// repositories are never parsed. The items compared at each reference
    /// are the union over all configured paths.
    ///
    /// # Errors
    ///
    /// Returns [`DiffError::NoRepositories`] when no path is configured and
    /// [`DiffError::NotAGitRepository`] when a configured path lies outside
    /// any Git repository, since neither state can be compared at a
    /// reference.
    pub fn diff(&self, opts: &DiffOptions) -> Result<DiffResult, DiffError> {
        if opts.repositories.is_empty() {
            return Err(DiffError::NoRepositories);
        }

        // Parse Git references
        let git_ref1 = GitRef::parse(&opts.ref1);
        let git_ref2 = GitRef::parse(&opts.ref2);

        // Parse items at each reference, accumulated across all paths
        let mut items1 = Vec::new();
        let mut items2 = Vec::new();

        for repo_path in &opts.repositories {
            let git_reader =
                GitReader::discover(repo_path).map_err(|e| DiffError::NotAGitRepository {
                    path: repo_path.display().to_string(),
                    reason: e.to_string(),
                })?;

            // Scope the scan to the configured path so files outside it
            // are never parsed
            let scope =
                git_reader
                    .scope_from_path(repo_path)
                    .map_err(|e| DiffError::ParseError {
                        path: repo_path.display().to_string(),
                        reason: e.to_string(),
                    })?;

            items1.extend(git_reader.parse_commit(&git_ref1, &scope).map_err(|e| {
                DiffError::ParseError {
                    path: format!("{}@{}", repo_path.display(), opts.ref1),
                    reason: e.to_string(),
                }
            })?);

            items2.extend(git_reader.parse_commit(&git_ref2, &scope).map_err(|e| {
                DiffError::ParseError {
                    path: format!("{}@{}", repo_path.display(), opts.ref2),
                    reason: e.to_string(),
                }
            })?);
        }

        // Build graphs from each reference
        let graph1 = KnowledgeGraphBuilder::new()
            .add_items(items1)
            .build()
            .map_err(|e| DiffError::GraphBuildError(e.to_string()))?;

        let graph2 = KnowledgeGraphBuilder::new()
            .add_items(items2)
            .build()
            .map_err(|e| DiffError::GraphBuildError(e.to_string()))?;

        // Compute diff
        let diff = GraphDiff::compute(&graph1, &graph2);

        Ok(DiffResult {
            diff,
            ref1: opts.ref1.clone(),
            ref2: opts.ref2.clone(),
        })
    }

    /// Computes the diff between two existing graphs.
    ///
    /// Use this method when you already have the graphs loaded.
    pub fn diff_graphs(
        &self,
        old_graph: &crate::graph::KnowledgeGraph,
        new_graph: &crate::graph::KnowledgeGraph,
        ref1: impl Into<String>,
        ref2: impl Into<String>,
    ) -> DiffResult {
        let diff = GraphDiff::compute(old_graph, new_graph);
        DiffResult {
            diff,
            ref1: ref1.into(),
            ref2: ref2.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use tempfile::TempDir;

    use super::*;
    use crate::test_utils::run_git;

    /// Item committed under the first configured path at the baseline.
    const FIRST_PATH_ITEM: &str = r#"---
id: "SOL-001"
type: solution
name: "First Path"
---
# Solution: First Path
"#;

    /// Item added under the second configured path by the second commit.
    const SECOND_PATH_ITEM: &str = r#"---
id: "SOL-010"
type: solution
name: "Second Path"
---
# Solution: Second Path
"#;

    fn create_test_file(dir: &Path, name: &str, content: &str) {
        fs::write(dir.join(name), content).unwrap();
    }

    /// Creates a Git repository with an item under `docs` at the baseline
    /// and an item under `specs` added by the second commit.
    fn multi_path_repo() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let repo = temp_dir.path();

        run_git(repo, &["init"]);
        run_git(repo, &["config", "user.name", "Sara Tests"]);
        run_git(repo, &["config", "user.email", "tests@example.com"]);

        fs::create_dir(repo.join("docs")).unwrap();
        fs::write(repo.join("docs/SOL-001.md"), FIRST_PATH_ITEM).unwrap();
        run_git(repo, &["add", "."]);
        run_git(repo, &["commit", "-m", "baseline"]);

        fs::create_dir(repo.join("specs")).unwrap();
        fs::write(repo.join("specs/SOL-010.md"), SECOND_PATH_ITEM).unwrap();
        run_git(repo, &["add", "."]);
        run_git(repo, &["commit", "-m", "add spec"]);

        temp_dir
    }

    #[test]
    fn test_diff_outside_a_git_repository_errors() {
        let temp_dir = TempDir::new().unwrap();

        create_test_file(
            temp_dir.path(),
            "solution.md",
            r#"---
id: "SOL-001"
type: solution
name: "Test Solution"
---
# Solution
"#,
        );

        let opts = DiffOptions::new("main", "feature")
            .with_repositories(vec![temp_dir.path().to_path_buf()]);

        let error = DiffService::new().diff(&opts).unwrap_err();

        let message = error.to_string();
        assert!(
            matches!(error, DiffError::NotAGitRepository { .. }),
            "got: {message}"
        );
        assert!(
            message.contains(&temp_dir.path().display().to_string()),
            "got: {message}"
        );
    }

    #[test]
    fn test_diff_without_repositories_errors() {
        let opts = DiffOptions::new("HEAD~1", "HEAD");

        let error = DiffService::new().diff(&opts).unwrap_err();

        assert!(matches!(error, DiffError::NoRepositories), "got: {error}");
    }

    #[test]
    fn test_diff_errors_when_one_path_is_outside_a_git_repository() {
        let repo = multi_path_repo();
        let outside = TempDir::new().unwrap();

        let opts = DiffOptions::new("HEAD~1", "HEAD")
            .with_repositories(vec![repo.path().join("docs"), outside.path().to_path_buf()]);

        let error = DiffService::new().diff(&opts).unwrap_err();

        assert!(
            matches!(error, DiffError::NotAGitRepository { .. }),
            "got: {error}"
        );
    }

    #[test]
    fn test_diff_options_builder() {
        let opts = DiffOptions::new("HEAD~1", "HEAD")
            .add_repository("/path/to/repo1".into())
            .add_repository("/path/to/repo2".into());

        assert_eq!(opts.ref1, "HEAD~1");
        assert_eq!(opts.ref2, "HEAD");
        assert_eq!(opts.repositories.len(), 2);
    }

    #[test]
    fn test_git_diff_unions_all_configured_paths() {
        let repo = multi_path_repo();
        let opts = DiffOptions::new("HEAD~1", "HEAD")
            .with_repositories(vec![repo.path().join("docs"), repo.path().join("specs")]);

        let service = DiffService::new();
        let result = service.diff(&opts).unwrap();

        assert_eq!(result.diff.added_items.len(), 1);
        assert_eq!(result.diff.added_items[0].id, "SOL-010");
        assert!(result.diff.removed_items.is_empty());
        assert!(result.diff.modified_items.is_empty());
    }

    #[test]
    fn test_diff_in_git_repo() {
        // Use the current repository for testing Git comparison
        let current_dir = std::env::current_dir().unwrap();

        // Only run this test if we're in a git repo
        if !crate::repository::is_git_repo(&current_dir) {
            return;
        }

        let opts = DiffOptions::new("HEAD", "HEAD").with_repositories(vec![current_dir]);

        let service = DiffService::new();
        let result = service.diff(&opts).unwrap();

        // Comparing HEAD to HEAD should produce no changes
        assert!(result.is_empty());
    }
}

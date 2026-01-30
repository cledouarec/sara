//! Diff service implementation.

use crate::graph::{GraphDiff, KnowledgeGraphBuilder};
use crate::repository::{GitReader, GitRef, parse_directory};

use super::DiffOptions;

/// Errors that can occur during diff operations.
#[derive(Debug, thiserror::Error)]
pub enum DiffError {
    /// Failed to parse repository.
    #[error("Failed to parse repository {path}: {reason}")]
    ParseError { path: String, reason: String },

    /// Failed to build graph.
    #[error("Failed to build graph: {0}")]
    GraphBuildError(String),

    /// Git reference not supported yet.
    #[error(
        "Git reference comparison not fully implemented. Only current state comparison is available."
    )]
    GitRefNotSupported,

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
    /// Whether this was a full Git ref comparison or a workaround.
    pub is_full_comparison: bool,
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
    /// This method supports full Git reference comparison when the repository
    /// paths are Git repositories. It will parse the knowledge graph at each
    /// reference point and compute the differences.
    pub fn diff(&self, opts: &DiffOptions) -> Result<DiffResult, DiffError> {
        // Try Git-based comparison first
        if let Some(result) = self.try_git_diff(opts)? {
            return Ok(result);
        }

        // Fall back to current working directory comparison
        self.diff_working_directory(opts)
    }

    /// Attempts Git-based diff comparison.
    /// Returns None if Git comparison is not possible (e.g., not a Git repo).
    fn try_git_diff(&self, opts: &DiffOptions) -> Result<Option<DiffResult>, DiffError> {
        // We need at least one repository path
        if opts.repositories.is_empty() {
            return Ok(None);
        }

        // Try to open a Git reader for the first repository
        let repo_path = &opts.repositories[0];
        let git_reader = match GitReader::discover(repo_path) {
            Ok(reader) => reader,
            Err(_) => return Ok(None), // Not a Git repo, fall back
        };

        // Parse Git references
        let git_ref1 = GitRef::parse(&opts.ref1);
        let git_ref2 = GitRef::parse(&opts.ref2);

        // Parse items at each reference
        let items1 = git_reader
            .parse_commit(&git_ref1)
            .map_err(|e| DiffError::ParseError {
                path: format!("{}@{}", repo_path.display(), opts.ref1),
                reason: e.to_string(),
            })?;

        let items2 = git_reader
            .parse_commit(&git_ref2)
            .map_err(|e| DiffError::ParseError {
                path: format!("{}@{}", repo_path.display(), opts.ref2),
                reason: e.to_string(),
            })?;

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

        Ok(Some(DiffResult {
            diff,
            ref1: opts.ref1.clone(),
            ref2: opts.ref2.clone(),
            is_full_comparison: true,
        }))
    }

    /// Falls back to comparing current working directory state with itself.
    fn diff_working_directory(&self, opts: &DiffOptions) -> Result<DiffResult, DiffError> {
        let items = self.parse_repositories(&opts.repositories)?;

        let graph1 = KnowledgeGraphBuilder::new()
            .add_items(items.clone())
            .build()
            .map_err(|e| DiffError::GraphBuildError(e.to_string()))?;

        let graph2 = KnowledgeGraphBuilder::new()
            .add_items(items)
            .build()
            .map_err(|e| DiffError::GraphBuildError(e.to_string()))?;

        let diff = GraphDiff::compute(&graph1, &graph2);

        Ok(DiffResult {
            diff,
            ref1: opts.ref1.clone(),
            ref2: opts.ref2.clone(),
            is_full_comparison: false,
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
            is_full_comparison: true,
        }
    }

    /// Parses items from all repository paths.
    fn parse_repositories(
        &self,
        repositories: &[std::path::PathBuf],
    ) -> Result<Vec<crate::model::Item>, DiffError> {
        let mut all_items = Vec::new();

        for repo_path in repositories {
            let items = parse_directory(repo_path).map_err(|e| DiffError::ParseError {
                path: repo_path.display().to_string(),
                reason: e.to_string(),
            })?;
            all_items.extend(items);
        }

        Ok(all_items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    fn create_test_file(dir: &Path, name: &str, content: &str) {
        fs::write(dir.join(name), content).unwrap();
    }

    #[test]
    fn test_diff_empty_repositories_non_git() {
        let temp_dir = TempDir::new().unwrap();

        let opts = DiffOptions::new("HEAD~1", "HEAD")
            .with_repositories(vec![temp_dir.path().to_path_buf()]);

        let service = DiffService::new();
        let result = service.diff(&opts).unwrap();

        // Non-git directory falls back to working directory comparison
        assert!(result.is_empty());
        assert!(!result.is_full_comparison);
    }

    #[test]
    fn test_diff_with_items_non_git() {
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

        let service = DiffService::new();
        let result = service.diff(&opts).unwrap();

        // Non-git: falls back to comparing current state with itself
        assert!(result.is_empty());
        assert!(!result.is_full_comparison);
        assert_eq!(result.ref1, "main");
        assert_eq!(result.ref2, "feature");
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
        // Should be a full Git comparison
        assert!(result.is_full_comparison);
    }
}

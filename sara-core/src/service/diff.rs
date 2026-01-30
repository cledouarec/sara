//! Diff service for comparing knowledge graphs.
//!
//! Provides functionality to compute differences between two states of the
//! requirements knowledge graph, supporting Git reference comparisons.

use std::path::PathBuf;

use crate::graph::{GraphBuilder, GraphDiff};
use crate::repository::{GitReader, GitRef};

// ============================================================================
// Error
// ============================================================================

/// Errors that can occur during diff operations.
#[derive(Debug, thiserror::Error)]
pub enum DiffError {
    /// Failed to parse repository.
    #[error("Failed to parse repository {path}: {reason}")]
    ParseError { path: String, reason: String },

    /// Failed to build graph.
    #[error("Failed to build graph: {0}")]
    GraphBuildError(String),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// ============================================================================
// Options
// ============================================================================

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

// ============================================================================
// Result
// ============================================================================

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
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.diff.is_empty()
    }
}

// ============================================================================
// Public Functions
// ============================================================================

/// Computes the diff between two references.
///
/// This function supports full Git reference comparison when the repository
/// paths are Git repositories. It will parse the knowledge graph at each
/// reference point and compute the differences.
///
/// # Errors
///
/// Returns error if repository parsing or graph building fails.
pub fn diff(opts: &DiffOptions) -> Result<DiffResult, DiffError> {
    // Try Git-based comparison first
    if let Some(result) = try_git_diff(opts)? {
        return Ok(result);
    }

    // Fall back to current working directory comparison
    diff_working_directory(opts)
}

// ============================================================================
// Private Functions
// ============================================================================

/// Attempts Git-based diff comparison.
/// Returns None if Git comparison is not possible (e.g., not a Git repo).
fn try_git_diff(opts: &DiffOptions) -> Result<Option<DiffResult>, DiffError> {
    if opts.repositories.is_empty() {
        return Ok(None);
    }

    let repo_path = &opts.repositories[0];
    let git_reader = match GitReader::discover(repo_path) {
        Ok(reader) => reader,
        Err(_) => return Ok(None),
    };

    let git_ref1 = GitRef::parse(&opts.ref1);
    let git_ref2 = GitRef::parse(&opts.ref2);

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

    let graph1 = GraphBuilder::new()
        .add_items(items1)
        .build()
        .map_err(|e| DiffError::GraphBuildError(e.to_string()))?;

    let graph2 = GraphBuilder::new()
        .add_items(items2)
        .build()
        .map_err(|e| DiffError::GraphBuildError(e.to_string()))?;

    let graph_diff = GraphDiff::compute(&graph1, &graph2);

    Ok(Some(DiffResult {
        diff: graph_diff,
        ref1: opts.ref1.clone(),
        ref2: opts.ref2.clone(),
        is_full_comparison: true,
    }))
}

/// Falls back when Git comparison is not possible.
///
/// Returns an empty diff since we cannot compare different states
/// without Git history.
fn diff_working_directory(opts: &DiffOptions) -> Result<DiffResult, DiffError> {
    Ok(DiffResult {
        diff: GraphDiff::default(),
        ref1: opts.ref1.clone(),
        ref2: opts.ref2.clone(),
        is_full_comparison: false,
    })
}

// ============================================================================
// Tests
// ============================================================================

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

        let result = diff(&opts).unwrap();

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

        let result = diff(&opts).unwrap();

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

        let result = diff(&opts).unwrap();

        // Comparing HEAD to HEAD should produce no changes
        assert!(result.is_empty());
        // Should be a full Git comparison
        assert!(result.is_full_comparison);
    }
}

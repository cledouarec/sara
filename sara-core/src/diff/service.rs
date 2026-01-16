//! Diff service implementation.

use crate::graph::{GraphBuilder, GraphDiff};
use crate::repository::parse_directory;

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
    /// Note: Full Git reference support is not yet implemented. Currently,
    /// this compares the current working directory state with itself.
    pub fn diff(&self, opts: &DiffOptions) -> Result<DiffResult, DiffError> {
        // TODO: Implement full Git reference support
        // For now, we parse the current state and compare with itself

        let is_full_comparison = false;

        // Parse all repositories
        let items = self.parse_repositories(&opts.repositories)?;

        // Build graphs (currently identical since we don't have Git ref support)
        let graph1 = GraphBuilder::new()
            .add_items(items.clone())
            .build()
            .map_err(|e| DiffError::GraphBuildError(e.to_string()))?;

        let graph2 = GraphBuilder::new()
            .add_items(items)
            .build()
            .map_err(|e| DiffError::GraphBuildError(e.to_string()))?;

        // Compute diff
        let diff = GraphDiff::compute(&graph1, &graph2);

        Ok(DiffResult {
            diff,
            ref1: opts.ref1.clone(),
            ref2: opts.ref2.clone(),
            is_full_comparison,
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
    fn test_diff_empty_repositories() {
        let temp_dir = TempDir::new().unwrap();

        let opts = DiffOptions::new("HEAD~1", "HEAD")
            .with_repositories(vec![temp_dir.path().to_path_buf()]);

        let service = DiffService::new();
        let result = service.diff(&opts).unwrap();

        assert!(result.is_empty());
        assert!(!result.is_full_comparison);
    }

    #[test]
    fn test_diff_with_items() {
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

        // Since we compare current state with itself, there should be no changes
        assert!(result.is_empty());
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
}

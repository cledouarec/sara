//! Diff options for comparing knowledge graphs.

use std::path::PathBuf;

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

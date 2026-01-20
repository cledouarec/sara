//! Metadata structures for items and source tracking.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Tracks the file origin of an item for error reporting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    /// Repository path (absolute).
    pub repository: PathBuf,

    /// Relative path within repository.
    pub file_path: PathBuf,

    /// Optional Git commit/branch if reading from history.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_ref: Option<String>,
}

impl SourceLocation {
    /// Creates a new SourceLocation.
    pub fn new(repository: impl Into<PathBuf>, file_path: impl Into<PathBuf>) -> Self {
        Self {
            repository: repository.into(),
            file_path: file_path.into(),
            git_ref: None,
        }
    }

    /// Creates a new SourceLocation with a Git reference.
    pub fn with_git_ref(
        repository: impl Into<PathBuf>,
        file_path: impl Into<PathBuf>,
        git_ref: impl Into<String>,
    ) -> Self {
        Self {
            repository: repository.into(),
            file_path: file_path.into(),
            git_ref: Some(git_ref.into()),
        }
    }

    /// Returns the full path to the file.
    pub fn full_path(&self) -> PathBuf {
        self.repository.join(&self.file_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_location_full_path() {
        let loc = SourceLocation::new("/repo", "docs/SOL-001.md");
        assert_eq!(loc.full_path(), PathBuf::from("/repo/docs/SOL-001.md"));
    }

    #[test]
    fn test_source_location_with_git_ref() {
        let loc = SourceLocation::with_git_ref("/repo", "docs/SOL-001.md", "main");
        assert_eq!(loc.git_ref, Some("main".to_string()));
    }
}

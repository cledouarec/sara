//! Git repository integration using git2.

use std::path::{Path, PathBuf};

use git2::{Commit, ObjectType, Repository};

use crate::error::SaraError;
use crate::model::Item;
use crate::parser::parse_markdown_file;

/// Represents a Git reference that can be used to read files.
#[derive(Debug, Clone)]
pub enum GitRef {
    /// HEAD of the repository.
    Head,
    /// A specific commit SHA.
    Commit(String),
    /// A branch name.
    Branch(String),
    /// A tag name.
    Tag(String),
}

impl GitRef {
    /// Parses a Git reference string.
    ///
    /// Supports formats like:
    /// - `HEAD` - the current HEAD
    /// - `abc123` - a commit SHA (abbreviated or full)
    /// - `refs/heads/main` - a branch reference
    /// - `refs/tags/v1.0` - a tag reference
    /// - `main` - a branch name (shorthand)
    pub fn parse(s: &str) -> Self {
        let s = s.trim();
        if s.eq_ignore_ascii_case("head") {
            GitRef::Head
        } else if s.starts_with("refs/heads/") {
            GitRef::Branch(s.trim_start_matches("refs/heads/").to_string())
        } else if s.starts_with("refs/tags/") {
            GitRef::Tag(s.trim_start_matches("refs/tags/").to_string())
        } else if s.len() >= 7 && s.chars().all(|c| c.is_ascii_hexdigit()) {
            GitRef::Commit(s.to_string())
        } else {
            // Assume it's a branch name
            GitRef::Branch(s.to_string())
        }
    }
}

/// A reader for Git repository contents.
pub struct GitReader {
    repo: Repository,
    repo_path: PathBuf,
}

impl GitReader {
    /// Opens a Git repository at the given path.
    pub fn open(path: &Path) -> Result<Self, SaraError> {
        let repo = Repository::open(path).map_err(|e| SaraError::GitError(e.to_string()))?;
        let repo_path = path.to_path_buf();
        Ok(Self { repo, repo_path })
    }

    /// Discovers and opens the Git repository containing the given path.
    pub fn discover(path: &Path) -> Result<Self, SaraError> {
        let repo = Repository::discover(path).map_err(|e| SaraError::GitError(e.to_string()))?;
        let repo_path = repo
            .workdir()
            .ok_or_else(|| SaraError::GitError("Bare repository not supported".to_string()))?
            .to_path_buf();
        Ok(Self { repo, repo_path })
    }

    /// Returns the repository root path.
    pub fn repo_path(&self) -> &Path {
        &self.repo_path
    }

    /// Resolves a Git reference to a commit.
    pub fn resolve_ref(&self, git_ref: &GitRef) -> Result<Commit<'_>, SaraError> {
        match git_ref {
            GitRef::Head => {
                let head = self
                    .repo
                    .head()
                    .map_err(|e| SaraError::GitError(e.to_string()))?;
                head.peel_to_commit()
                    .map_err(|e| SaraError::GitError(e.to_string()))
            }
            GitRef::Commit(sha) => {
                // Use revparse_single to handle abbreviated SHAs
                let obj = self
                    .repo
                    .revparse_single(sha)
                    .map_err(|e| SaraError::GitError(e.to_string()))?;
                obj.peel_to_commit()
                    .map_err(|e| SaraError::GitError(e.to_string()))
            }
            GitRef::Branch(name) => {
                let branch = self
                    .repo
                    .find_branch(name, git2::BranchType::Local)
                    .map_err(|e| SaraError::GitError(e.to_string()))?;
                branch
                    .get()
                    .peel_to_commit()
                    .map_err(|e| SaraError::GitError(e.to_string()))
            }
            GitRef::Tag(name) => {
                let tag_ref = format!("refs/tags/{}", name);
                let obj = self
                    .repo
                    .revparse_single(&tag_ref)
                    .map_err(|e| SaraError::GitError(e.to_string()))?;
                obj.peel_to_commit()
                    .map_err(|e| SaraError::GitError(e.to_string()))
            }
        }
    }

    /// Reads a file from a specific commit.
    pub fn read_file(&self, commit: &Commit<'_>, path: &Path) -> Result<String, SaraError> {
        let tree = commit
            .tree()
            .map_err(|e| SaraError::GitError(e.to_string()))?;

        let entry = tree
            .get_path(path)
            .map_err(|e| SaraError::GitError(e.to_string()))?;

        let blob = entry
            .to_object(&self.repo)
            .map_err(|e| SaraError::GitError(e.to_string()))?
            .peel_to_blob()
            .map_err(|e| SaraError::GitError(e.to_string()))?;

        String::from_utf8(blob.content().to_vec())
            .map_err(|e| SaraError::GitError(format!("Invalid UTF-8 in file: {}", e)))
    }

    /// Lists all Markdown files in a commit's tree.
    pub fn list_markdown_files(&self, commit: &Commit<'_>) -> Result<Vec<PathBuf>, SaraError> {
        let tree = commit
            .tree()
            .map_err(|e| SaraError::GitError(e.to_string()))?;

        let mut files = Vec::new();
        self.walk_tree(&tree, PathBuf::new(), &mut files)?;
        Ok(files)
    }

    /// Recursively walks a tree to find Markdown files.
    fn walk_tree(
        &self,
        tree: &git2::Tree<'_>,
        prefix: PathBuf,
        files: &mut Vec<PathBuf>,
    ) -> Result<(), SaraError> {
        for entry in tree.iter() {
            let name = entry
                .name()
                .ok_or_else(|| SaraError::GitError("Invalid file name".to_string()))?;

            // Skip hidden files and directories
            if name.starts_with('.') {
                continue;
            }

            let path = prefix.join(name);

            match entry.kind() {
                Some(ObjectType::Blob) => {
                    // Check for Markdown extension
                    if name.ends_with(".md") || name.ends_with(".markdown") {
                        files.push(path);
                    }
                }
                Some(ObjectType::Tree) => {
                    let subtree = entry
                        .to_object(&self.repo)
                        .map_err(|e| SaraError::GitError(e.to_string()))?
                        .peel_to_tree()
                        .map_err(|e| SaraError::GitError(e.to_string()))?;
                    self.walk_tree(&subtree, path, files)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Parses all Markdown files from a specific commit.
    pub fn parse_commit(&self, git_ref: &GitRef) -> Result<Vec<Item>, SaraError> {
        let commit = self.resolve_ref(git_ref)?;
        let files = self.list_markdown_files(&commit)?;

        let mut items = Vec::new();
        let mut parse_errors = Vec::new();

        for file_path in files {
            let content = match self.read_file(&commit, &file_path) {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!("Failed to read {}: {}", file_path.display(), e);
                    continue;
                }
            };

            if !crate::parser::has_frontmatter(&content) {
                continue;
            }

            match parse_markdown_file(&content, &file_path, &self.repo_path) {
                Ok(item) => items.push(item),
                Err(e) => {
                    tracing::warn!("Failed to parse {}: {}", file_path.display(), e);
                    parse_errors.push(e);
                }
            }
        }

        if !parse_errors.is_empty() && items.is_empty() {
            return Err(parse_errors.remove(0).into());
        }

        Ok(items)
    }
}

/// Checks if a path is inside a Git repository.
pub fn is_git_repo(path: &Path) -> bool {
    Repository::discover(path).is_ok()
}

/// Gets the root of the Git repository containing the given path.
pub fn get_repo_root(path: &Path) -> Option<PathBuf> {
    Repository::discover(path)
        .ok()
        .and_then(|r| r.workdir().map(|p| p.to_path_buf()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_ref_parse_head() {
        matches!(GitRef::parse("HEAD"), GitRef::Head);
        matches!(GitRef::parse("head"), GitRef::Head);
    }

    #[test]
    fn test_git_ref_parse_commit() {
        matches!(GitRef::parse("abc1234"), GitRef::Commit(_));
        matches!(GitRef::parse("abc123456789"), GitRef::Commit(_));
    }

    #[test]
    fn test_git_ref_parse_branch() {
        matches!(GitRef::parse("main"), GitRef::Branch(_));
        matches!(GitRef::parse("refs/heads/main"), GitRef::Branch(_));
    }

    #[test]
    fn test_git_ref_parse_tag() {
        matches!(GitRef::parse("refs/tags/v1.0"), GitRef::Tag(_));
    }

    #[test]
    fn test_is_git_repo() {
        let current_dir = std::env::current_dir().unwrap();
        // This test assumes we're running from within the sara repo
        assert!(is_git_repo(&current_dir));
    }
}

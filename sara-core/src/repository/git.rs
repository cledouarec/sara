//! Git repository integration using gitoxide (`gix`).
//!
//! Provides read-only access to commits and tree contents for parsing
//! Markdown items at arbitrary Git references. Pure Rust — no libgit2 or
//! OpenSSL dependency.

use std::path::{Path, PathBuf};

use gix::bstr::ByteSlice;
use gix::object::tree::EntryKind;
use gix::{Commit, Repository, Tree};

use crate::error::SaraError;
use crate::model::Item;
use crate::parser::InputFormat;

/// Wraps a gix error into [`SaraError::Gix`].
fn gix_err<E>(e: E) -> SaraError
where
    E: std::error::Error + Send + Sync + 'static,
{
    SaraError::Gix(Box::new(e))
}

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
    ///
    /// # Errors
    /// Returns [`SaraError::Gix`] if the path is not a valid Git repository.
    pub fn open(path: &Path) -> Result<Self, SaraError> {
        let repo = gix::open(path).map_err(gix_err)?;
        let repo_path = path.to_path_buf();
        Ok(Self { repo, repo_path })
    }

    /// Discovers and opens the Git repository containing the given path.
    ///
    /// # Errors
    /// Returns [`SaraError::Gix`] if no repository is found, or
    /// [`SaraError::Git`] for bare repositories (no working directory).
    pub fn discover(path: &Path) -> Result<Self, SaraError> {
        let repo = gix::discover(path).map_err(gix_err)?;
        let repo_path = repo
            .workdir()
            .ok_or_else(|| SaraError::Git("Bare repository not supported".to_string()))?
            .to_path_buf();
        Ok(Self { repo, repo_path })
    }

    /// Returns the repository root path.
    pub fn repo_path(&self) -> &Path {
        &self.repo_path
    }

    /// Resolves a Git reference to a commit.
    ///
    /// # Errors
    /// Returns [`SaraError::Gix`] if the reference cannot be resolved or
    /// does not point to a commit.
    pub fn resolve_ref(&self, git_ref: &GitRef) -> Result<Commit<'_>, SaraError> {
        match git_ref {
            GitRef::Head => self.repo.head_commit().map_err(gix_err),
            GitRef::Commit(sha) => self.peel_spec_to_commit(sha.as_bytes()),
            GitRef::Branch(name) => {
                let spec = format!("refs/heads/{name}");
                self.peel_spec_to_commit(spec.as_bytes())
            }
            GitRef::Tag(name) => {
                let spec = format!("refs/tags/{name}");
                self.peel_spec_to_commit(spec.as_bytes())
            }
        }
    }

    /// Parses a revspec and peels it to a commit.
    fn peel_spec_to_commit(&self, spec: &[u8]) -> Result<Commit<'_>, SaraError> {
        let id = self.repo.rev_parse_single(spec).map_err(gix_err)?;
        let object = id.object().map_err(gix_err)?;
        // `peel_to_kind` follows annotated tag chains until reaching a commit.
        let commit_object = object
            .peel_to_kind(gix::object::Kind::Commit)
            .map_err(gix_err)?;
        commit_object
            .try_into_commit()
            .map_err(|_| SaraError::Git("Reference does not resolve to a commit".to_string()))
    }

    /// Reads a file from a specific commit.
    ///
    /// # Errors
    /// Returns [`SaraError::Gix`] for tree-lookup failures, or
    /// [`SaraError::Git`] if the path is missing, not a blob, or invalid UTF-8.
    pub fn read_file(&self, commit: &Commit<'_>, path: &Path) -> Result<String, SaraError> {
        let tree = commit.tree().map_err(gix_err)?;
        let entry = tree
            .lookup_entry_by_path(path)
            .map_err(gix_err)?
            .ok_or_else(|| SaraError::Git(format!("Path not found in tree: {}", path.display())))?;
        let object = entry.object().map_err(gix_err)?;
        let mut blob = object
            .try_into_blob()
            .map_err(|_| SaraError::Git(format!("Path is not a blob: {}", path.display())))?;

        String::from_utf8(std::mem::take(&mut blob.data))
            .map_err(|e| SaraError::Git(format!("Invalid UTF-8 in file: {e}")))
    }

    /// Lists all Markdown files in a commit's tree.
    ///
    /// # Errors
    /// Returns [`SaraError::Gix`] if tree traversal fails, or
    /// [`SaraError::Git`] for non-UTF-8 file names.
    pub fn list_markdown_files(&self, commit: &Commit<'_>) -> Result<Vec<PathBuf>, SaraError> {
        let tree = commit.tree().map_err(gix_err)?;

        let mut files = Vec::new();
        self.walk_tree(&tree, PathBuf::new(), &mut files)?;
        Ok(files)
    }

    /// Recursively walks a tree to find Markdown files.
    fn walk_tree(
        &self,
        tree: &Tree<'_>,
        prefix: PathBuf,
        files: &mut Vec<PathBuf>,
    ) -> Result<(), SaraError> {
        for entry in tree.iter() {
            let entry = entry.map_err(gix_err)?;
            let name = entry
                .filename()
                .to_str()
                .map_err(|_| SaraError::Git("Invalid file name".to_string()))?;

            if name.starts_with('.') {
                continue;
            }

            let path = prefix.join(name);

            match entry.mode().kind() {
                EntryKind::Blob | EntryKind::BlobExecutable
                    if name.ends_with(".md") || name.ends_with(".markdown") =>
                {
                    files.push(path);
                }
                EntryKind::Tree => {
                    let object = entry.object().map_err(gix_err)?;
                    let subtree = object
                        .try_into_tree()
                        .map_err(|_| SaraError::Git("Expected tree object".to_string()))?;
                    self.walk_tree(&subtree, path, files)?;
                }
                // EntryKind::Link (symlink) and EntryKind::Commit (submodule) are
                // intentionally skipped, matching the prior git2 behavior.
                _ => {}
            }
        }
        Ok(())
    }

    /// Parses all Markdown files from a specific commit.
    ///
    /// # Errors
    /// Propagates errors from [`Self::resolve_ref`]. Returns the first parse
    /// error only when no items could be parsed at all; otherwise individual
    /// parse failures are logged via `tracing` and the successful items are
    /// returned.
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

            // Skip files without frontmatter
            if !crate::parser::has_frontmatter(&content) {
                continue;
            }

            match crate::parser::parse_metadata(
                &content,
                &file_path,
                &self.repo_path,
                InputFormat::Markdown,
            ) {
                Ok(item) => items.push(item),
                Err(e) => {
                    tracing::warn!("Failed to parse {}: {}", file_path.display(), e);
                    parse_errors.push(e);
                }
            }
        }

        if !parse_errors.is_empty() && items.is_empty() {
            return Err(parse_errors.remove(0));
        }

        Ok(items)
    }
}

/// Checks if a path is inside a Git repository.
pub fn is_git_repo(path: &Path) -> bool {
    gix::discover(path).is_ok()
}

/// Gets the root of the Git repository containing the given path.
pub fn get_repo_root(path: &Path) -> Option<PathBuf> {
    gix::discover(path)
        .ok()
        .and_then(|r| r.workdir().map(|p| p.to_path_buf()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_ref_parse_head() {
        assert!(matches!(GitRef::parse("HEAD"), GitRef::Head));
        assert!(matches!(GitRef::parse("head"), GitRef::Head));
    }

    #[test]
    fn test_git_ref_parse_commit() {
        assert!(matches!(GitRef::parse("abc1234"), GitRef::Commit(_)));
        assert!(matches!(GitRef::parse("abc123456789"), GitRef::Commit(_)));
    }

    #[test]
    fn test_git_ref_parse_branch() {
        assert!(matches!(GitRef::parse("main"), GitRef::Branch(_)));
        assert!(matches!(
            GitRef::parse("refs/heads/main"),
            GitRef::Branch(_)
        ));
    }

    #[test]
    fn test_git_ref_parse_tag() {
        assert!(matches!(GitRef::parse("refs/tags/v1.0"), GitRef::Tag(_)));
    }

    #[test]
    fn test_is_git_repo() {
        let current_dir = std::env::current_dir().unwrap();
        // This test assumes we're running from within the sara repo
        assert!(is_git_repo(&current_dir));
    }
}

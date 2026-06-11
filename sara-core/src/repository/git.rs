//! Git repository integration using gitoxide (`gix`).
//!
//! Provides read-only access to commits and tree contents for parsing
//! Markdown items at arbitrary Git references. Pure Rust — no libgit2 or
//! OpenSSL dependency.

use std::path::{Component, Path, PathBuf};

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

/// Characters that introduce revision-expression syntax, such as `HEAD~1`,
/// `main^2`, or `@{upstream}`. Git forbids `~`, `^`, and `:` in reference
/// names, so their presence always indicates a revision expression. `@` may
/// appear in branch names, but resolving such a name through the revspec
/// grammar still finds the reference.
const REVSPEC_CHARS: &[char] = &['~', '^', '@', ':'];

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
    /// A revision expression such as `HEAD~1` or `main^2`.
    Revspec(String),
}

impl GitRef {
    /// Parses a Git reference string.
    ///
    /// Supports formats like:
    /// - `HEAD` - the current HEAD
    /// - `abc123` - a commit SHA (abbreviated or full)
    /// - `refs/heads/main` - a branch reference
    /// - `refs/tags/v1.0` - a tag reference
    /// - `HEAD~1` - a revision expression
    /// - `main` - a branch name (shorthand)
    ///
    /// Any input containing `~`, `^`, `@`, or `:` is treated as a revision
    /// expression and resolved through the full revspec grammar.
    pub fn parse(s: &str) -> Self {
        let s = s.trim();
        if s.eq_ignore_ascii_case("head") {
            GitRef::Head
        } else if s.contains(REVSPEC_CHARS) {
            GitRef::Revspec(s.to_string())
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

    /// Converts a filesystem path into a tree scope relative to the
    /// repository root.
    ///
    /// Both the repository root and the given path are canonicalized so that
    /// relative components and symbolic links resolve consistently. The
    /// repository root itself yields an empty scope, which selects the whole
    /// tree.
    ///
    /// # Errors
    /// Returns [`SaraError::Io`] if either path cannot be canonicalized, or
    /// [`SaraError::Git`] if the path lies outside the repository.
    pub fn scope_from_path(&self, path: &Path) -> Result<PathBuf, SaraError> {
        let root = self.repo_path.canonicalize()?;
        let path = path.canonicalize()?;
        path.strip_prefix(&root)
            .map(Path::to_path_buf)
            .map_err(|_| {
                SaraError::Git(format!(
                    "Path {} is outside the repository {}",
                    path.display(),
                    root.display()
                ))
            })
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
            GitRef::Revspec(spec) => self.peel_spec_to_commit(spec.as_bytes()),
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

    /// Lists the Markdown files under `scope` in a commit's tree.
    ///
    /// `scope` is a path relative to the repository root; an empty path or
    /// `.` selects the whole tree. Returned paths are always relative to the
    /// repository root. A scope that is absent from the commit yields an
    /// empty list, since the directory may not exist at every reference.
    ///
    /// # Errors
    /// Returns [`SaraError::Gix`] if tree traversal fails, or
    /// [`SaraError::Git`] for non-UTF-8 file names.
    pub fn list_markdown_files(
        &self,
        commit: &Commit<'_>,
        scope: &Path,
    ) -> Result<Vec<PathBuf>, SaraError> {
        let tree = commit.tree().map_err(gix_err)?;
        let Some((tree, prefix)) = self.scoped_tree(tree, scope)? else {
            return Ok(Vec::new());
        };

        let mut files = Vec::new();
        self.walk_tree(&tree, prefix, &mut files)?;
        Ok(files)
    }

    /// Resolves a repository-relative scope to its subtree.
    ///
    /// Returns the subtree together with the path prefix to prepend to
    /// walked entries, or `None` when the scope does not exist as a
    /// directory in the commit. An empty scope or `.` selects the whole
    /// tree.
    fn scoped_tree<'repo>(
        &self,
        tree: Tree<'repo>,
        scope: &Path,
    ) -> Result<Option<(Tree<'repo>, PathBuf)>, SaraError> {
        let scope: PathBuf = scope
            .components()
            .filter(|c| !matches!(c, Component::CurDir))
            .collect();
        if scope.as_os_str().is_empty() {
            return Ok(Some((tree, PathBuf::new())));
        }

        let Some(entry) = tree.lookup_entry_by_path(&scope).map_err(gix_err)? else {
            tracing::debug!("Scope {} not present in commit tree", scope.display());
            return Ok(None);
        };
        if !matches!(entry.mode().kind(), EntryKind::Tree) {
            tracing::debug!("Scope {} is not a directory", scope.display());
            return Ok(None);
        }

        let object = entry.object().map_err(gix_err)?;
        let subtree = object
            .try_into_tree()
            .map_err(|_| SaraError::Git("Expected tree object".to_string()))?;
        Ok(Some((subtree, scope)))
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

    /// Parses the Markdown files under `scope` at a specific commit.
    ///
    /// `scope` is a path relative to the repository root; an empty path or
    /// `.` selects the whole tree. A scope that is absent from the commit
    /// yields no items. Use [`Self::scope_from_path`] to derive the scope
    /// from a filesystem path.
    ///
    /// # Errors
    /// Propagates errors from [`Self::resolve_ref`]. Returns the first parse
    /// error only when no items could be parsed at all; otherwise individual
    /// parse failures are logged via `tracing` and the successful items are
    /// returned.
    pub fn parse_commit(&self, git_ref: &GitRef, scope: &Path) -> Result<Vec<Item>, SaraError> {
        let commit = self.resolve_ref(git_ref)?;
        let files = self.list_markdown_files(&commit, scope)?;

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
    use std::fs;

    use tempfile::TempDir;

    use super::*;
    use crate::test_utils::run_git;

    /// Item committed inside the `docs` directory of the test repository.
    const SCOPED_ITEM: &str = r#"---
id: "SOL-001"
type: solution
name: "Scoped"
---
# Solution: Scoped
"#;

    /// Item committed at the root of the test repository.
    const ROOT_ITEM: &str = r#"---
id: "SOL-002"
type: solution
name: "Root"
---
# Solution: Root
"#;

    /// Creates a Git repository with one item under `docs` and one at the
    /// repository root, committed at HEAD.
    fn scoped_repo() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let repo = temp_dir.path();

        run_git(repo, &["init"]);
        run_git(repo, &["config", "user.name", "Sara Tests"]);
        run_git(repo, &["config", "user.email", "tests@example.com"]);

        fs::create_dir(repo.join("docs")).unwrap();
        fs::write(repo.join("docs/SOL-001.md"), SCOPED_ITEM).unwrap();
        fs::write(repo.join("SOL-002.md"), ROOT_ITEM).unwrap();
        run_git(repo, &["add", "."]);
        run_git(repo, &["commit", "-m", "initial"]);

        temp_dir
    }

    #[test]
    fn test_scope_from_path_repo_root_is_empty() {
        let repo = scoped_repo();
        let reader = GitReader::discover(repo.path()).unwrap();

        let scope = reader.scope_from_path(repo.path()).unwrap();

        assert_eq!(scope, PathBuf::new());
    }

    #[test]
    fn test_scope_from_path_subdirectory() {
        let repo = scoped_repo();
        let reader = GitReader::discover(&repo.path().join("docs")).unwrap();

        let scope = reader.scope_from_path(&repo.path().join("docs")).unwrap();

        assert_eq!(scope, PathBuf::from("docs"));
    }

    #[test]
    fn test_parse_commit_scopes_to_subtree() {
        let repo = scoped_repo();
        let reader = GitReader::discover(repo.path()).unwrap();

        let items = reader
            .parse_commit(&GitRef::Head, Path::new("docs"))
            .unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id.as_str(), "SOL-001");
    }

    #[test]
    fn test_parse_commit_empty_scope_lists_whole_tree() {
        let repo = scoped_repo();
        let reader = GitReader::discover(repo.path()).unwrap();

        let items = reader.parse_commit(&GitRef::Head, Path::new(".")).unwrap();

        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_parse_commit_missing_scope_yields_no_items() {
        let repo = scoped_repo();
        let reader = GitReader::discover(repo.path()).unwrap();

        let items = reader
            .parse_commit(&GitRef::Head, Path::new("no-such-dir"))
            .unwrap();

        assert!(items.is_empty());
    }

    #[test]
    fn test_list_markdown_files_keeps_repo_relative_paths() {
        let repo = scoped_repo();
        let reader = GitReader::discover(repo.path()).unwrap();
        let commit = reader.resolve_ref(&GitRef::Head).unwrap();

        let files = reader
            .list_markdown_files(&commit, Path::new("docs"))
            .unwrap();

        assert_eq!(files, vec![PathBuf::from("docs/SOL-001.md")]);
    }

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
    fn test_git_ref_parse_revspec() {
        assert!(matches!(GitRef::parse("HEAD~1"), GitRef::Revspec(_)));
        assert!(matches!(GitRef::parse("HEAD^"), GitRef::Revspec(_)));
        assert!(matches!(GitRef::parse("main~2"), GitRef::Revspec(_)));
    }

    #[test]
    fn test_is_git_repo() {
        let current_dir = std::env::current_dir().unwrap();
        // This test assumes we're running from within the sara repo
        assert!(is_git_repo(&current_dir));
    }
}

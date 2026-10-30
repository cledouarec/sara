//! Guard on the files the write tools may create.
//!
//! A new item must live where the scanner will find it: in a Markdown file
//! below one of the configured repositories, outside hidden directories.
//! [`resolve_item_file`] enforces this before anything is written, which
//! also keeps the write tools from touching any other file on the machine.

use std::fs;
use std::path::{Component, Path, PathBuf};

use sara_core::repository::{is_hidden, is_markdown_file};

/// Resolves where an item file requested by a client may be written.
///
/// `file` is relative to `base`, the server's working directory, unless it
/// is absolute. The file must be one the scanner reads: below one of
/// `repositories` (themselves relative to `base`), with an `.md` or
/// `.markdown` extension, and with no hidden directory or file name below
/// the repository. Symbolic links in the existing part of the path are
/// resolved first, so a link cannot lead outside the repositories, and a
/// link leading nowhere is refused, since writing through it would create
/// its target wherever it points.
///
/// Returns the path spelled from the repository as configured, the way the
/// other tools report files.
///
/// # Errors
///
/// Returns a message explaining which rule the path breaks.
pub fn resolve_item_file(
    file: &str,
    repositories: &[PathBuf],
    base: &Path,
) -> Result<PathBuf, String> {
    let requested = Path::new(file);
    if !is_markdown_file(requested) {
        return Err(format!(
            "'{file}' is not a Markdown file: give it the .md or .markdown extension."
        ));
    }

    let target = resolve_symlinks(&normalize(&base.join(requested)))?;
    for repository in repositories {
        let Ok(root) = base.join(repository).canonicalize() else {
            continue;
        };
        let Ok(relative) = target.strip_prefix(&root) else {
            continue;
        };
        if relative.as_os_str().is_empty() {
            continue;
        }
        let hidden = relative
            .components()
            .any(|component| matches!(component, Component::Normal(name) if is_hidden(name)));
        if hidden {
            return Err(format!(
                "'{file}' is in a hidden directory or is a hidden file, which sara does not read."
            ));
        }
        return Ok(repository.join(relative));
    }

    let repositories: Vec<String> = repositories
        .iter()
        .map(|repository| repository.display().to_string())
        .collect();
    Err(format!(
        "'{file}' is outside the repositories sara reads ({}); choose a path inside one of them.",
        repositories.join(", ")
    ))
}

/// Resolves the `.` and `..` components of `path` without touching the file
/// system, since the file may not exist yet.
fn normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other),
        }
    }
    normalized
}

/// Resolves the symbolic links of the longest existing ancestor of `path`
/// and appends the components that do not exist yet.
///
/// # Errors
///
/// Returns a message when a component exists but cannot be resolved, such as
/// a symbolic link leading nowhere.
fn resolve_symlinks(path: &Path) -> Result<PathBuf, String> {
    let mut existing = path;
    let mut missing = Vec::new();
    loop {
        if let Ok(canonical) = existing.canonicalize() {
            return Ok(missing
                .iter()
                .rev()
                .fold(canonical, |resolved, name| resolved.join(name)));
        }
        if fs::symlink_metadata(existing).is_ok() {
            return Err(format!(
                "{} is a symbolic link leading nowhere, or cannot be read; sara does not write \
                 through it.",
                existing.display()
            ));
        }
        match (existing.parent(), existing.file_name()) {
            (Some(parent), Some(name)) => {
                missing.push(name.to_os_string());
                existing = parent;
            }
            _ => return Ok(path.to_path_buf()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    /// Creates a project directory holding a `docs` repository.
    fn project() -> TempDir {
        let dir = TempDir::new().unwrap();
        fs::create_dir(dir.path().join("docs")).unwrap();
        dir
    }

    /// Resolves `file` in `project`, whose only repository is `docs`.
    fn resolve(project: &TempDir, file: &str) -> Result<PathBuf, String> {
        resolve_item_file(file, &[PathBuf::from("docs")], project.path())
    }

    #[test]
    fn test_accepts_new_file_below_a_repository() {
        let project = project();

        let file = resolve(&project, "docs/sub/new.md").unwrap();

        assert_eq!(file, Path::new("docs").join("sub").join("new.md"));
    }

    #[test]
    fn test_accepts_absolute_path_inside_a_repository() {
        let project = project();
        let absolute = project.path().join("docs").join("new.markdown");

        let file = resolve(&project, absolute.to_str().unwrap()).unwrap();

        assert_eq!(file, Path::new("docs").join("new.markdown"));
    }

    #[test]
    fn test_rejects_path_outside_the_repositories() {
        let project = project();

        let error = resolve(&project, "notes/new.md").unwrap_err();

        assert!(error.contains("outside"), "{error}");
        assert!(error.contains("docs"), "{error}");
    }

    #[test]
    fn test_rejects_parent_directory_escape() {
        let project = project();

        let error = resolve(&project, "docs/../new.md").unwrap_err();

        assert!(error.contains("outside"), "{error}");
    }

    #[test]
    fn test_rejects_other_extensions() {
        let project = project();

        let error = resolve(&project, "docs/new.txt").unwrap_err();

        assert!(error.contains(".md"), "{error}");
    }

    #[test]
    fn test_rejects_hidden_directories() {
        let project = project();

        let error = resolve(&project, "docs/.drafts/new.md").unwrap_err();

        assert!(error.contains("hidden"), "{error}");
    }

    #[cfg(unix)]
    #[test]
    fn test_rejects_symlink_leading_outside() {
        let project = project();
        let outside = TempDir::new().unwrap();
        std::os::unix::fs::symlink(outside.path(), project.path().join("docs").join("link"))
            .unwrap();

        let error = resolve(&project, "docs/link/new.md").unwrap_err();

        assert!(error.contains("outside"), "{error}");
    }

    #[cfg(unix)]
    #[test]
    fn test_rejects_dangling_symlink_file() {
        // Writing through the link would create its target, wherever it is.
        let project = project();
        let outside = TempDir::new().unwrap();
        std::os::unix::fs::symlink(
            outside.path().join("pwned.md"),
            project.path().join("docs").join("evil.md"),
        )
        .unwrap();

        let error = resolve(&project, "docs/evil.md").unwrap_err();

        assert!(error.contains("symbolic link"), "{error}");
    }

    #[cfg(unix)]
    #[test]
    fn test_rejects_dangling_symlink_directory() {
        let project = project();
        let outside = TempDir::new().unwrap();
        std::os::unix::fs::symlink(
            outside.path().join("missing"),
            project.path().join("docs").join("link"),
        )
        .unwrap();

        let error = resolve(&project, "docs/link/new.md").unwrap_err();

        assert!(error.contains("symbolic link"), "{error}");
    }
}

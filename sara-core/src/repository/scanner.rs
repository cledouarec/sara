//! File scanner for discovering Markdown files.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use rayon::prelude::*;

use crate::error::SaraError;
use crate::model::Item;
use crate::parser::{InputFormat, has_frontmatter, parse_metadata};

/// Scans a directory for Markdown files and returns their paths.
pub fn scan_directory(path: &Path) -> Result<Vec<PathBuf>, SaraError> {
    let mut files = Vec::new();
    scan_directory_recursive(path, &mut files)?;
    Ok(files)
}

fn scan_directory_recursive(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), SaraError> {
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Skip hidden directories
            if path
                .file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with('.'))
            {
                continue;
            }
            scan_directory_recursive(&path, files)?;
        } else if path.is_file() {
            // Check for Markdown extension
            if let Some(ext) = path.extension()
                && (ext == "md" || ext == "markdown")
            {
                files.push(path);
            }
        }
    }

    Ok(())
}

/// A path skipped during a scan, together with the reason.
///
/// Unreadable or unparseable files do not abort a scan; each one is recorded
/// so callers can report the problem instead of silently dropping items.
#[derive(Debug, Clone)]
pub struct ScanWarning {
    /// The file or repository path that was skipped.
    pub path: PathBuf,
    /// Why the path was skipped.
    pub reason: String,
}

impl fmt::Display for ScanWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "skipped {}: {}", self.path.display(), self.reason)
    }
}

/// Items collected by a scan, along with the paths that were skipped.
#[derive(Debug, Default)]
pub struct ScanResult {
    /// Items parsed successfully.
    pub items: Vec<Item>,
    /// One warning per skipped file or repository path.
    pub warnings: Vec<ScanWarning>,
}

/// Result of parsing a single file.
#[derive(Debug)]
enum ParseResult {
    /// Successfully parsed item.
    Item(Box<Item>),
    /// File had no frontmatter, skip it.
    Skipped,
    /// Error reading file.
    ReadError(std::io::Error),
    /// Error parsing file.
    ParseError(SaraError),
}

/// Parses all Markdown files in a directory.
///
/// Files are parsed in parallel with rayon, significantly improving
/// performance on large document sets. Target: 500 documents in <1 second
/// (SC-001). Files that cannot be read or parsed are skipped and reported
/// in [`ScanResult::warnings`].
pub fn parse_directory(repository_path: &Path) -> Result<ScanResult, SaraError> {
    let files = scan_directory(repository_path)?;

    // Parse files in parallel
    let results: Vec<ParseResult> = files
        .par_iter()
        .map(|file_path| {
            // Read file content
            let content = match fs::read_to_string(file_path) {
                Ok(c) => c,
                Err(e) => return ParseResult::ReadError(e),
            };

            // Skip files without frontmatter
            if !has_frontmatter(&content) {
                return ParseResult::Skipped;
            }

            // Calculate relative path from repository root
            let relative_path = file_path
                .strip_prefix(repository_path)
                .unwrap_or(file_path)
                .to_path_buf();

            // Parse the markdown file
            match parse_metadata(
                &content,
                &relative_path,
                repository_path,
                InputFormat::Markdown,
            ) {
                Ok(item) => ParseResult::Item(Box::new(item)),
                Err(e) => ParseResult::ParseError(e),
            }
        })
        .collect();

    let mut scan = ScanResult::default();
    for (file_path, result) in files.iter().zip(results) {
        match result {
            ParseResult::Item(item) => scan.items.push(*item),
            ParseResult::Skipped => {}
            ParseResult::ReadError(e) => {
                tracing::warn!("Failed to read file {}: {}", file_path.display(), e);
                scan.warnings.push(ScanWarning {
                    path: file_path.clone(),
                    reason: e.to_string(),
                });
            }
            ParseResult::ParseError(e) => {
                tracing::warn!("Parse error: {}", e);
                scan.warnings.push(ScanWarning {
                    path: file_path.clone(),
                    reason: e.to_string(),
                });
            }
        }
    }

    Ok(scan)
}

/// Parses multiple repository paths and collects all items.
///
/// Repository paths that do not exist or cannot be scanned are skipped and
/// reported in [`ScanResult::warnings`], together with the warnings from
/// every scanned repository.
pub fn parse_repositories(paths: &[PathBuf]) -> ScanResult {
    let mut scan = ScanResult::default();

    let mut valid_paths = Vec::new();
    for path in paths {
        if path.exists() {
            valid_paths.push(path);
        } else {
            tracing::warn!("Repository path does not exist: {}", path.display());
            scan.warnings.push(ScanWarning {
                path: path.clone(),
                reason: "repository path does not exist".to_string(),
            });
        }
    }

    // For small number of repositories, parallelism at file level is more efficient
    // For larger numbers, we could parallelize at the repository level too
    let results: Vec<Result<ScanResult, SaraError>> = valid_paths
        .par_iter()
        .map(|path| parse_directory(path))
        .collect();

    // Combine results
    for (path, result) in valid_paths.iter().zip(results) {
        match result {
            Ok(repo_scan) => {
                scan.items.extend(repo_scan.items);
                scan.warnings.extend(repo_scan.warnings);
            }
            Err(e) => {
                tracing::warn!("Failed to parse repository: {}", e);
                scan.warnings.push(ScanWarning {
                    path: (*path).clone(),
                    reason: e.to_string(),
                });
            }
        }
    }

    scan
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_scan_fixtures() {
        let fixtures_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("tests/fixtures/valid_graph");

        if fixtures_path.exists() {
            let files = scan_directory(&fixtures_path).unwrap();
            assert!(!files.is_empty(), "Should find fixture files");
            assert!(
                files
                    .iter()
                    .all(|f| f.extension().is_some_and(|e| e == "md"))
            );
        }
    }

    #[test]
    fn test_parse_directory_reports_skipped_files() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        std::fs::write(
            temp_dir.path().join("good.md"),
            "---\nid: \"SOL-001\"\ntype: solution\nname: \"Good\"\n---\n# Good\n",
        )
        .unwrap();
        std::fs::write(
            temp_dir.path().join("bad.md"),
            "---\nid: \"SOL-002\ntype: solution\n---\n# Bad\n",
        )
        .unwrap();

        let scan = parse_directory(temp_dir.path()).unwrap();

        assert_eq!(scan.items.len(), 1);
        assert_eq!(scan.items[0].id.as_str(), "SOL-001");
        assert_eq!(scan.warnings.len(), 1);
        assert!(scan.warnings[0].path.ends_with("bad.md"));
        assert!(scan.warnings[0].to_string().starts_with("skipped "));
    }

    #[test]
    fn test_parse_repositories_reports_missing_path() {
        let missing = PathBuf::from("/nonexistent/sara-repository");

        let scan = parse_repositories(std::slice::from_ref(&missing));

        assert!(scan.items.is_empty());
        assert_eq!(scan.warnings.len(), 1);
        assert_eq!(scan.warnings[0].path, missing);
    }
}

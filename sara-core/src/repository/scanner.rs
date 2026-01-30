//! File scanner for discovering Markdown files.

use std::fs;
use std::path::{Path, PathBuf};

use rayon::prelude::*;

use crate::error::SaraError;
use crate::model::Item;
use crate::parser::parse_markdown_file;

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

/// Parses all Markdown files in a directory and returns items.
///
/// Uses parallel parsing with rayon for improved performance on large directories.
pub fn parse_directory(repository_path: &Path) -> Result<Vec<Item>, SaraError> {
    parse_directory_parallel(repository_path)
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
    ParseError(crate::error::ParseError),
}

/// Parses files in parallel using rayon for improved performance.
///
/// This function uses rayon's parallel iterator to parse multiple files
/// concurrently, significantly improving performance on large document sets.
/// Target: 500 documents in <1 second (SC-001).
pub fn parse_directory_parallel(repository_path: &Path) -> Result<Vec<Item>, SaraError> {
    let files = scan_directory(repository_path)?;

    let results: Vec<ParseResult> = files
        .par_iter()
        .map(|file_path| {
            let content = match fs::read_to_string(file_path) {
                Ok(c) => c,
                Err(e) => return ParseResult::ReadError(e),
            };

            if !crate::parser::has_frontmatter(&content) {
                return ParseResult::Skipped;
            }

            let relative_path = file_path
                .strip_prefix(repository_path)
                .unwrap_or(file_path)
                .to_path_buf();

            match parse_markdown_file(&content, &relative_path, repository_path) {
                Ok(item) => ParseResult::Item(Box::new(item)),
                Err(e) => ParseResult::ParseError(e),
            }
        })
        .collect();

    let mut items = Vec::new();
    let mut parse_errors = Vec::new();

    for result in results {
        match result {
            ParseResult::Item(item) => items.push(*item),
            ParseResult::Skipped => {}
            ParseResult::ReadError(e) => {
                tracing::warn!("Failed to read file: {}", e);
            }
            ParseResult::ParseError(e) => parse_errors.push(e),
        }
    }

    // Log parse errors but don't fail unless all files failed
    for error in &parse_errors {
        tracing::warn!("Parse error: {}", error);
    }

    if !parse_errors.is_empty() && items.is_empty() {
        return Err(parse_errors.remove(0).into());
    }

    Ok(items)
}

/// Parses multiple repository paths and returns all items.
///
/// Uses parallel parsing across repositories for improved performance.
pub fn parse_repositories(paths: &[PathBuf]) -> Result<Vec<Item>, SaraError> {
    // For small number of repositories, parallelism at file level is more efficient
    // For larger numbers, we could parallelize at the repository level too
    let valid_paths: Vec<&PathBuf> = paths
        .iter()
        .filter(|path| {
            if !path.exists() {
                tracing::warn!("Repository path does not exist: {}", path.display());
                false
            } else {
                true
            }
        })
        .collect();

    let results: Vec<Result<Vec<Item>, SaraError>> = valid_paths
        .par_iter()
        .map(|path| parse_directory(path))
        .collect();

    let mut all_items = Vec::new();
    for result in results {
        match result {
            Ok(items) => all_items.extend(items),
            Err(e) => tracing::warn!("Failed to parse repository: {}", e),
        }
    }

    Ok(all_items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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
}

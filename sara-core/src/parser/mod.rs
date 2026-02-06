//! Input parsers for reading documents into core structures.
//!
//! This module is the input adapter in the hexagonal architecture. It reads
//! various formats and fills core structures (`Item`, `Relationship`, etc.).
//!
//! Use [`InputFormat`] with [`parse_metadata`] or [`parse_document`] to parse
//! content without depending on format-specific functions.

mod frontmatter;
mod markdown;
mod yaml;

#[doc(inline)]
pub use frontmatter::{extract_body, has_frontmatter, update_frontmatter};
#[doc(inline)]
pub use markdown::{ParsedDocument, extract_name_from_content};

use std::path::Path;

use crate::error::SaraError;
use crate::model::Item;

/// Supported input formats for document parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputFormat {
    /// Markdown with YAML frontmatter.
    Markdown,
}

/// Parses content and extracts an [`Item`].
///
/// Dispatches to the appropriate format-specific parser based on `format`.
///
/// # Errors
///
/// Returns `SaraError` if the content cannot be parsed in the given format.
pub fn parse_metadata(
    content: &str,
    file_path: &Path,
    repository: &Path,
    format: InputFormat,
) -> Result<Item, SaraError> {
    match format {
        InputFormat::Markdown => markdown::parse_markdown_file(content, file_path, repository),
    }
}

/// Parses content and returns both the [`Item`] and body text.
///
/// Dispatches to the appropriate format-specific parser based on `format`.
///
/// # Errors
///
/// Returns `SaraError` if the content cannot be parsed in the given format.
pub fn parse_document(
    content: &str,
    file_path: &Path,
    repository: &Path,
    format: InputFormat,
) -> Result<ParsedDocument, SaraError> {
    match format {
        InputFormat::Markdown => markdown::parse_document(content, file_path, repository),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ItemType, RelationshipType};
    use std::path::PathBuf;

    const SOLUTION_MD: &str = r#"---
id: "SOL-001"
type: solution
name: "Test Solution"
description: "A test solution"
is_refined_by:
  - "UC-001"
---
# Test Solution

Body content.
"#;

    #[test]
    fn test_parse_metadata_markdown() {
        let item = parse_metadata(
            SOLUTION_MD,
            &PathBuf::from("SOL-001.md"),
            &PathBuf::from("/repo"),
            InputFormat::Markdown,
        )
        .unwrap();

        assert_eq!(item.id.as_str(), "SOL-001");
        assert_eq!(item.item_type, ItemType::Solution);
        assert_eq!(item.name, "Test Solution");
        let is_refined_by: Vec<_> = item
            .relationship_ids(RelationshipType::IsRefinedBy)
            .collect();
        assert_eq!(is_refined_by.len(), 1);
        assert_eq!(is_refined_by[0].as_str(), "UC-001");
    }

    #[test]
    fn test_parse_document_markdown() {
        let doc = parse_document(
            SOLUTION_MD,
            &PathBuf::from("SOL-001.md"),
            &PathBuf::from("/repo"),
            InputFormat::Markdown,
        )
        .unwrap();

        assert_eq!(doc.item.id.as_str(), "SOL-001");
        assert!(doc.body.contains("# Test Solution"));
        assert!(doc.body.contains("Body content."));
    }

    #[test]
    fn test_input_format_debug() {
        assert_eq!(format!("{:?}", InputFormat::Markdown), "Markdown");
    }
}

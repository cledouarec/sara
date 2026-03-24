//! Output generators for producing documents from core `Item` structures.
//!
//! This module is the output adapter in the hexagonal architecture. It reads
//! from core structures (`Item`, `ItemAttributes`, relationships) and produces
//! various output formats.
//!
//! Use [`OutputFormat`] with [`generate_document`] or [`generate_metadata`]
//! to produce output without depending on format-specific functions.

mod markdown;
mod yaml;

use crate::model::Item;

/// Supported output formats for document generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Markdown with YAML frontmatter.
    Markdown,
}

/// Generates a complete document (frontmatter + body) from an [`Item`].
///
/// Dispatches to the appropriate format-specific generator based on `format`.
#[must_use]
pub fn generate_document(item: &Item, format: OutputFormat) -> String {
    match format {
        OutputFormat::Markdown => markdown::generate_document(item),
    }
}

/// Generates YAML frontmatter (including `---` delimiters) from an [`Item`].
///
/// Dispatches to the appropriate format-specific generator based on `format`.
#[must_use]
pub fn generate_metadata(item: &Item, format: OutputFormat) -> String {
    match format {
        OutputFormat::Markdown => yaml::generate_metadata(item),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ItemBuilder, ItemId, ItemType, SourceLocation};
    use std::path::PathBuf;

    fn test_source() -> SourceLocation {
        SourceLocation {
            repository: PathBuf::from("/repo"),
            file_path: PathBuf::from("docs/test.md"),
            git_ref: None,
        }
    }

    #[test]
    fn test_generate_document_markdown() {
        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("SOL-001"))
            .item_type(ItemType::Solution)
            .name("Test Solution")
            .source(test_source())
            .build()
            .unwrap();

        let doc = generate_document(&item, OutputFormat::Markdown);

        assert!(doc.contains("id: \"SOL-001\""));
        assert!(doc.contains("# Solution: Test Solution"));
    }

    #[test]
    fn test_generate_metadata_markdown() {
        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("SOL-001"))
            .item_type(ItemType::Solution)
            .name("Test Solution")
            .source(test_source())
            .build()
            .unwrap();

        let fm = generate_metadata(&item, OutputFormat::Markdown);

        assert!(fm.starts_with("---"));
        assert!(fm.ends_with("---"));
        assert!(fm.contains("id: \"SOL-001\""));
        assert!(!fm.contains("## Overview"));
    }

    #[test]
    fn test_output_format_debug() {
        assert_eq!(format!("{:?}", OutputFormat::Markdown), "Markdown");
    }
}

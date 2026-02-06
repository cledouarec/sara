//! YAML frontmatter generation from core `Item` structures.
//!
//! Produces the YAML frontmatter portion (between `---` delimiters) by rendering
//! the full Markdown document via the `markdown` generator and extracting the
//! frontmatter section.

use crate::model::Item;

use super::markdown;

/// Generates YAML frontmatter (including `---` delimiters) from an `Item`.
///
/// Returns the frontmatter block from `---` through the closing `---`,
/// suitable for insertion into a Markdown document.
#[must_use]
pub fn generate_metadata(item: &Item) -> String {
    let document = markdown::generate_document(item);
    extract_frontmatter(&document).to_string()
}

/// Extracts the frontmatter block (including delimiters) from a document string.
///
/// Returns an empty string if no valid frontmatter delimiters are found.
fn extract_frontmatter(content: &str) -> &str {
    if !content.starts_with("---") {
        return "";
    }
    let after_first = &content[3..];
    if let Some(end_pos) = after_first.find("\n---") {
        // Include "---" + content + "\n---"
        &content[..end_pos + 3 + 4]
    } else {
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ItemBuilder, ItemId, ItemType};
    use crate::model::SourceLocation;
    use std::path::PathBuf;

    fn test_source() -> SourceLocation {
        SourceLocation {
            repository: PathBuf::from("/repo"),
            file_path: PathBuf::from("docs/test.md"),
            git_ref: None,
        }
    }

    #[test]
    fn test_generate_metadata_solution() {
        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("SOL-001"))
            .item_type(ItemType::Solution)
            .name("Test Solution")
            .source(test_source())
            .build()
            .unwrap();

        let fm = generate_metadata(&item);

        assert!(fm.starts_with("---"));
        assert!(fm.ends_with("---"));
        assert!(fm.contains("id: \"SOL-001\""));
        assert!(fm.contains("type: solution"));
        assert!(fm.contains("name: \"Test Solution\""));
        // Should not contain body content
        assert!(!fm.contains("## Overview"));
    }

    #[test]
    fn test_extract_frontmatter() {
        let doc = "---\nid: test\ntype: solution\n---\n\n# Body";
        let fm = extract_frontmatter(doc);
        assert_eq!(fm, "---\nid: test\ntype: solution\n---");
    }

    #[test]
    fn test_extract_frontmatter_no_frontmatter() {
        let doc = "# Just a heading\n\nSome content.";
        let fm = extract_frontmatter(doc);
        assert_eq!(fm, "");
    }
}

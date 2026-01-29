//! Output adapters for generating Markdown documents and YAML frontmatter.
//!
//! This module provides format-specific generation from domain types:
//! - `yaml` - YAML frontmatter generation
//! - `markdown` - Complete Markdown document generation

pub mod markdown;
pub mod yaml;

use crate::graph::KnowledgeGraph;
use crate::model::ItemType;

// Convenience re-exports
pub use markdown::generate_document;
pub use yaml::{generate_edit_frontmatter, generate_frontmatter};

/// Generates a new ID for the given type, optionally using a sequence number.
pub fn generate_id(item_type: ItemType, sequence: Option<u32>) -> String {
    let prefix = item_type.prefix();
    let num = sequence.unwrap_or(1);
    format!("{}-{:03}", prefix, num)
}

/// Suggests the next ID based on existing items in the graph.
///
/// Finds the highest existing ID for the given type and returns the next sequential ID.
/// If no graph is provided or no items exist, returns the first ID (e.g., "SOL-001").
pub fn suggest_next_id(item_type: ItemType, graph: Option<&KnowledgeGraph>) -> String {
    let Some(graph) = graph else {
        return generate_id(item_type, None);
    };

    let prefix = item_type.prefix();
    let max_num = graph
        .items()
        .filter(|item| item.item_type == item_type)
        .filter_map(|item| {
            item.id
                .as_str()
                .strip_prefix(prefix)
                .and_then(|suffix| suffix.trim_start_matches('-').parse::<u32>().ok())
        })
        .max()
        .unwrap_or(0);

    format!("{}-{:03}", prefix, max_num + 1)
}

/// Extracts a name from a markdown file's first heading.
pub fn extract_name_from_content(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(heading) = trimmed.strip_prefix("# ") {
            return Some(heading.trim().to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_id() {
        assert_eq!(generate_id(ItemType::Solution, Some(1)), "SOL-001");
        assert_eq!(generate_id(ItemType::UseCase, Some(42)), "UC-042");
        assert_eq!(generate_id(ItemType::SystemRequirement, None), "SYSREQ-001");
        assert_eq!(
            generate_id(ItemType::ArchitectureDecisionRecord, Some(1)),
            "ADR-001"
        );
    }

    #[test]
    fn test_extract_name_from_content() {
        let content = "# My Document\n\nSome content here.";
        assert_eq!(
            extract_name_from_content(content),
            Some("My Document".to_string())
        );

        let content_no_heading = "No heading here";
        assert_eq!(extract_name_from_content(content_no_heading), None);
    }
}

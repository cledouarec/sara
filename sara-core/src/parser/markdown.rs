//! Markdown file parsing and document extraction.

use std::path::Path;

use serde::Deserialize;

use crate::error::ParseError;
use crate::model::{
    DownstreamRefs, Item, ItemAttributes, ItemBuilder, ItemId, ItemType, SourceLocation,
    UpstreamRefs,
};
use crate::parser::frontmatter::extract_frontmatter;

/// Raw frontmatter structure for deserialization.
///
/// This represents the YAML frontmatter as it appears in Markdown files.
/// All relationship fields accept both single values and arrays for flexibility.
#[derive(Debug, Clone, Deserialize)]
pub struct RawFrontmatter {
    /// Unique identifier (required).
    pub id: String,

    /// Item type (required).
    #[serde(rename = "type")]
    pub item_type: ItemType,

    /// Human-readable name (required).
    pub name: String,

    /// Optional description.
    #[serde(default)]
    pub description: Option<String>,

    // Upstream references (toward Solution)
    /// Items this item refines (for UseCase, Scenario).
    #[serde(default)]
    pub refines: Vec<String>,

    /// Items this item derives from (for SystemRequirement, HW/SW Requirement).
    #[serde(default)]
    pub derives_from: Vec<String>,

    /// Items this item satisfies (for SystemArchitecture, HW/SW DetailedDesign).
    #[serde(default)]
    pub satisfies: Vec<String>,

    // Downstream references (toward Detailed Designs)
    /// Items that refine this item (for Solution, UseCase).
    #[serde(default)]
    pub is_refined_by: Vec<String>,

    /// Items derived from this item (for Scenario, SystemArchitecture).
    #[serde(default)]
    pub derives: Vec<String>,

    /// Items that satisfy this item (for SystemRequirement, HW/SW Requirement).
    #[serde(default)]
    pub is_satisfied_by: Vec<String>,

    // Type-specific attributes
    /// Specification statement (required for requirement types).
    #[serde(default)]
    pub specification: Option<String>,

    /// Peer dependencies (for requirement types).
    #[serde(default)]
    pub depends_on: Vec<String>,

    /// Target platform (for SystemArchitecture).
    #[serde(default)]
    pub platform: Option<String>,

    /// ADR links (reserved for future use).
    #[serde(default)]
    pub justified_by: Option<Vec<String>>,
}

impl RawFrontmatter {
    /// Converts string IDs to ItemIds for upstream refs.
    pub fn upstream_refs(&self) -> Result<UpstreamRefs, ParseError> {
        Ok(UpstreamRefs {
            refines: self.refines.iter().map(ItemId::new_unchecked).collect(),
            derives_from: self
                .derives_from
                .iter()
                .map(ItemId::new_unchecked)
                .collect(),
            satisfies: self.satisfies.iter().map(ItemId::new_unchecked).collect(),
        })
    }

    /// Converts string IDs to ItemIds for downstream refs.
    pub fn downstream_refs(&self) -> Result<DownstreamRefs, ParseError> {
        Ok(DownstreamRefs {
            is_refined_by: self
                .is_refined_by
                .iter()
                .map(ItemId::new_unchecked)
                .collect(),
            derives: self.derives.iter().map(ItemId::new_unchecked).collect(),
            is_satisfied_by: self
                .is_satisfied_by
                .iter()
                .map(ItemId::new_unchecked)
                .collect(),
        })
    }

    /// Converts to ItemAttributes.
    pub fn attributes(&self) -> ItemAttributes {
        ItemAttributes {
            specification: self.specification.clone(),
            depends_on: self.depends_on.iter().map(ItemId::new_unchecked).collect(),
            platform: self.platform.clone(),
            justified_by: self
                .justified_by
                .as_ref()
                .map(|ids| ids.iter().map(ItemId::new_unchecked).collect()),
        }
    }
}

/// Parses a Markdown file and extracts the item.
///
/// # Arguments
/// * `content` - The raw file content.
/// * `file_path` - Relative path within the repository.
/// * `repository` - Absolute path to the repository root.
///
/// # Returns
/// The parsed Item, or a ParseError if parsing fails.
pub fn parse_markdown_file(
    content: &str,
    file_path: &Path,
    repository: &Path,
) -> Result<Item, ParseError> {
    let extracted = extract_frontmatter(content, file_path)?;

    let frontmatter: RawFrontmatter =
        serde_yaml::from_str(&extracted.yaml).map_err(|e| ParseError::InvalidYaml {
            file: file_path.to_path_buf(),
            reason: e.to_string(),
        })?;

    // Validate item ID format
    let item_id = ItemId::new(&frontmatter.id).map_err(|e| ParseError::InvalidFrontmatter {
        file: file_path.to_path_buf(),
        reason: format!("Invalid item ID: {}", e),
    })?;

    // Create source location
    let source = SourceLocation::new(repository, file_path);

    // Build the item
    let mut builder = ItemBuilder::new()
        .id(item_id)
        .item_type(frontmatter.item_type)
        .name(&frontmatter.name)
        .source(source)
        .upstream(frontmatter.upstream_refs()?)
        .downstream(frontmatter.downstream_refs()?)
        .attributes(frontmatter.attributes());

    if let Some(desc) = &frontmatter.description {
        builder = builder.description(desc);
    }

    builder.build().map_err(|e| ParseError::InvalidFrontmatter {
        file: file_path.to_path_buf(),
        reason: e.to_string(),
    })
}

/// Represents a parsed document with its item and body content.
#[derive(Debug)]
pub struct ParsedDocument {
    /// The extracted item.
    pub item: Item,
    /// The Markdown body content after frontmatter.
    pub body: String,
}

/// Parses a Markdown file and returns the item and body.
pub fn parse_document(
    content: &str,
    file_path: &Path,
    repository: &Path,
) -> Result<ParsedDocument, ParseError> {
    let extracted = extract_frontmatter(content, file_path)?;
    let item = parse_markdown_file(content, file_path, repository)?;

    Ok(ParsedDocument {
        item,
        body: extracted.body,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
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

This is the body content.
"#;

    const REQUIREMENT_MD: &str = r#"---
id: "SYSREQ-001"
type: system_requirement
name: "Performance Requirement"
specification: "The system SHALL respond within 100ms."
derives_from:
  - "SCEN-001"
is_satisfied_by:
  - "SYSARCH-001"
---
# Requirement
"#;

    #[test]
    fn test_parse_solution() {
        let item = parse_markdown_file(
            SOLUTION_MD,
            &PathBuf::from("SOL-001.md"),
            &PathBuf::from("/repo"),
        )
        .unwrap();

        assert_eq!(item.id.as_str(), "SOL-001");
        assert_eq!(item.item_type, ItemType::Solution);
        assert_eq!(item.name, "Test Solution");
        assert_eq!(item.description, Some("A test solution".to_string()));
        assert_eq!(item.downstream.is_refined_by.len(), 1);
        assert_eq!(item.downstream.is_refined_by[0].as_str(), "UC-001");
    }

    #[test]
    fn test_parse_requirement() {
        let item = parse_markdown_file(
            REQUIREMENT_MD,
            &PathBuf::from("SYSREQ-001.md"),
            &PathBuf::from("/repo"),
        )
        .unwrap();

        assert_eq!(item.id.as_str(), "SYSREQ-001");
        assert_eq!(item.item_type, ItemType::SystemRequirement);
        assert_eq!(
            item.attributes.specification,
            Some("The system SHALL respond within 100ms.".to_string())
        );
        assert_eq!(item.upstream.derives_from.len(), 1);
        assert_eq!(item.downstream.is_satisfied_by.len(), 1);
    }

    #[test]
    fn test_parse_document() {
        let doc = parse_document(
            SOLUTION_MD,
            &PathBuf::from("SOL-001.md"),
            &PathBuf::from("/repo"),
        )
        .unwrap();

        assert_eq!(doc.item.id.as_str(), "SOL-001");
        assert!(doc.body.contains("# Test Solution"));
    }

    #[test]
    fn test_parse_invalid_id() {
        let content = r#"---
id: "invalid id with spaces"
type: solution
name: "Test"
---
"#;
        let result =
            parse_markdown_file(content, &PathBuf::from("test.md"), &PathBuf::from("/repo"));
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_type() {
        let content = r#"---
id: "SOL-001"
name: "Test"
---
"#;
        let result =
            parse_markdown_file(content, &PathBuf::from("test.md"), &PathBuf::from("/repo"));
        assert!(result.is_err());
    }
}

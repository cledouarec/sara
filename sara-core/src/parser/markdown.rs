//! Markdown file parsing and document extraction.

use std::path::Path;

use crate::error::SaraError;
use crate::model::{Item, ItemBuilder, ItemId, ItemType, SourceLocation};
use crate::parser::frontmatter::extract_frontmatter;
use crate::parser::yaml::parse_yaml_frontmatter;

/// Parses a Markdown file and extracts the item.
///
/// The file must contain YAML frontmatter delimited by `---`.
pub fn parse_markdown_file(
    content: &str,
    file_path: &Path,
    repository: &Path,
) -> Result<Item, SaraError> {
    let extracted = extract_frontmatter(content, file_path)?;

    let frontmatter = parse_yaml_frontmatter(&extracted.yaml, file_path)?;

    // Validate item ID format
    let item_id = ItemId::new(&frontmatter.id).map_err(|e| SaraError::InvalidFrontmatter {
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
        .relationships(frontmatter.to_relationships());

    if let Some(desc) = &frontmatter.description {
        builder = builder.description(desc);
    }

    // Set type-specific attributes based on item type
    match frontmatter.item_type {
        ItemType::Solution | ItemType::UseCase | ItemType::Scenario => {}
        ItemType::SystemRequirement
        | ItemType::SoftwareRequirement
        | ItemType::HardwareRequirement => {
            if let Some(spec) = &frontmatter.specification {
                builder = builder.specification(spec);
            }
            for id in &frontmatter.depends_on {
                builder = builder.depends_on(ItemId::new_unchecked(id));
            }
        }
        ItemType::SystemArchitecture => {
            if let Some(platform) = &frontmatter.platform {
                builder = builder.platform(platform);
            }
        }
        ItemType::SoftwareDetailedDesign | ItemType::HardwareDetailedDesign => {}
        ItemType::ArchitectureDecisionRecord => {
            if let Some(status) = frontmatter.status {
                builder = builder.status(status);
            }
            builder = builder.deciders(frontmatter.deciders.clone());
            builder = builder.supersedes_all(
                frontmatter
                    .supersedes
                    .iter()
                    .map(ItemId::new_unchecked)
                    .collect(),
            );
        }
    }

    builder.build().map_err(|e| SaraError::InvalidFrontmatter {
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
) -> Result<ParsedDocument, SaraError> {
    let extracted = extract_frontmatter(content, file_path)?;
    let item = parse_markdown_file(content, file_path, repository)?;

    Ok(ParsedDocument {
        item,
        body: extracted.body,
    })
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
    use crate::model::{AdrStatus, RelationshipType};
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
        let is_refined_by: Vec<_> = item
            .relationship_ids(RelationshipType::IsRefinedBy)
            .collect();
        assert_eq!(is_refined_by.len(), 1);
        assert_eq!(is_refined_by[0].as_str(), "UC-001");
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
            item.attributes.specification().map(String::as_str),
            Some("The system SHALL respond within 100ms.")
        );
        let derives_from: Vec<_> = item
            .relationship_ids(RelationshipType::DerivesFrom)
            .collect();
        let is_satisfied_by: Vec<_> = item
            .relationship_ids(RelationshipType::IsSatisfiedBy)
            .collect();
        assert_eq!(derives_from.len(), 1);
        assert_eq!(is_satisfied_by.len(), 1);
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

    const ADR_MD: &str = r#"---
id: "ADR-001"
type: architecture_decision_record
name: "Use Microservices Architecture"
description: "Decision to adopt microservices"
status: proposed
deciders:
  - "Alice Smith"
  - "Bob Jones"
justifies:
  - "SYSARCH-001"
  - "SWDD-001"
supersedes: []
superseded_by: null
---
# Architecture Decision: Use Microservices Architecture

## Context and problem statement

We need to choose an architecture pattern for our system.

## Decision Outcome

Chosen option: Microservices, because it provides better scalability.
"#;

    #[test]
    fn test_parse_adr() {
        let item = parse_markdown_file(
            ADR_MD,
            &PathBuf::from("ADR-001.md"),
            &PathBuf::from("/repo"),
        )
        .unwrap();

        assert_eq!(item.id.as_str(), "ADR-001");
        assert_eq!(item.item_type, ItemType::ArchitectureDecisionRecord);
        assert_eq!(item.name, "Use Microservices Architecture");
        assert_eq!(
            item.description,
            Some("Decision to adopt microservices".to_string())
        );

        assert_eq!(item.attributes.status(), Some(AdrStatus::Proposed));
        assert_eq!(item.attributes.deciders().len(), 2);
        let justifies: Vec<_> = item.relationship_ids(RelationshipType::Justifies).collect();
        assert_eq!(justifies.len(), 2);
        assert_eq!(justifies[0].as_str(), "SYSARCH-001");
        assert_eq!(justifies[1].as_str(), "SWDD-001");
        assert!(item.attributes.supersedes().is_empty());
    }

    #[test]
    fn test_parse_adr_missing_deciders() {
        let content = r#"---
id: "ADR-002"
type: architecture_decision_record
name: "Test Decision"
status: proposed
---
"#;
        let result =
            parse_markdown_file(content, &PathBuf::from("test.md"), &PathBuf::from("/repo"));
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_adr_missing_status() {
        let content = r#"---
id: "ADR-003"
type: architecture_decision_record
name: "Test Decision"
deciders:
  - "Alice"
---
"#;
        let result =
            parse_markdown_file(content, &PathBuf::from("test.md"), &PathBuf::from("/repo"));
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_adr_with_supersession() {
        let content = r#"---
id: "ADR-005"
type: architecture_decision_record
name: "Updated Architecture Decision"
status: accepted
deciders:
  - "Alice Smith"
justifies:
  - "SYSARCH-001"
supersedes:
  - "ADR-001"
  - "ADR-002"
---
"#;
        let item = parse_markdown_file(
            content,
            &PathBuf::from("ADR-005.md"),
            &PathBuf::from("/repo"),
        )
        .unwrap();

        let justifies: Vec<_> = item.relationship_ids(RelationshipType::Justifies).collect();
        assert_eq!(justifies.len(), 1);
        assert_eq!(justifies[0].as_str(), "SYSARCH-001");
        assert_eq!(item.attributes.supersedes().len(), 2);
        assert_eq!(item.attributes.supersedes()[0].as_str(), "ADR-001");
        assert_eq!(item.attributes.supersedes()[1].as_str(), "ADR-002");
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

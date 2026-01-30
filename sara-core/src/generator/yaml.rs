//! YAML frontmatter generation using Tera templates.
//!
//! Generates YAML frontmatter by rendering the full document template
//! and extracting the frontmatter portion.

use super::markdown;
use crate::model::Item;

/// Generate YAML frontmatter from an Item.
///
/// Uses Tera templates to ensure consistency with full document generation.
/// Renders the complete document and extracts just the frontmatter portion.
pub fn generate_frontmatter(item: &Item) -> String {
    let full_document = markdown::generate_document(item);
    extract_frontmatter(&full_document)
}

/// Extracts the frontmatter portion from a document (including `---` delimiters).
fn extract_frontmatter(document: &str) -> String {
    let lines: Vec<&str> = document.lines().collect();

    if lines.is_empty() || lines[0].trim() != "---" {
        return String::new();
    }

    // Find closing delimiter
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "---" {
            // Include the closing delimiter
            let frontmatter_lines: Vec<&str> = lines[..=i].to_vec();
            return frontmatter_lines.join("\n") + "\n";
        }
    }

    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        AdrStatus, ItemAttributes, ItemBuilder, ItemId, ItemType, Relationship, RelationshipType,
        SourceLocation,
    };
    use std::path::PathBuf;

    fn test_source() -> SourceLocation {
        SourceLocation::new(PathBuf::from("/test"), "test.md")
    }

    #[test]
    fn test_generate_frontmatter_solution() {
        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("SOL-001"))
            .item_type(ItemType::Solution)
            .name("Test Solution")
            .description("A test solution")
            .source(test_source())
            .attributes(ItemAttributes::Solution)
            .build()
            .unwrap();

        let yaml = generate_frontmatter(&item);

        assert!(yaml.starts_with("---\n"));
        assert!(yaml.ends_with("---\n"));
        assert!(yaml.contains("id: \"SOL-001\""));
        assert!(yaml.contains("type: solution"));
        assert!(yaml.contains("name: \"Test Solution\""));
        assert!(yaml.contains("description: \"A test solution\""));
    }

    #[test]
    fn test_generate_frontmatter_use_case_with_refines() {
        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("UC-001"))
            .item_type(ItemType::UseCase)
            .name("Test Use Case")
            .source(test_source())
            .relationships(vec![Relationship::new(
                ItemId::new_unchecked("SOL-001"),
                RelationshipType::Refines,
            )])
            .attributes(ItemAttributes::UseCase)
            .build()
            .unwrap();

        let yaml = generate_frontmatter(&item);

        assert!(yaml.contains("refines:"));
        assert!(yaml.contains("SOL-001"));
    }

    #[test]
    fn test_generate_frontmatter_requirement_with_spec() {
        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("SYSREQ-001"))
            .item_type(ItemType::SystemRequirement)
            .name("Test Requirement")
            .source(test_source())
            .relationships(vec![Relationship::new(
                ItemId::new_unchecked("SCEN-001"),
                RelationshipType::DerivesFrom,
            )])
            .attributes(ItemAttributes::SystemRequirement {
                specification: "The system SHALL do X".to_string(),
                depends_on: Vec::new(),
            })
            .build()
            .unwrap();

        let yaml = generate_frontmatter(&item);

        assert!(yaml.contains("specification:"));
        assert!(yaml.contains("derives_from:"));
        assert!(yaml.contains("SCEN-001"));
    }

    #[test]
    fn test_generate_frontmatter_architecture_with_platform() {
        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("SYSARCH-001"))
            .item_type(ItemType::SystemArchitecture)
            .name("Web Platform")
            .source(test_source())
            .relationships(vec![Relationship::new(
                ItemId::new_unchecked("SYSREQ-001"),
                RelationshipType::Satisfies,
            )])
            .attributes(ItemAttributes::SystemArchitecture {
                platform: Some("AWS Lambda".to_string()),
            })
            .build()
            .unwrap();

        let yaml = generate_frontmatter(&item);

        assert!(yaml.contains("platform:"));
        assert!(yaml.contains("satisfies:"));
        assert!(yaml.contains("SYSREQ-001"));
    }

    #[test]
    fn test_generate_frontmatter_adr() {
        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("ADR-001"))
            .item_type(ItemType::ArchitectureDecisionRecord)
            .name("Use Microservices")
            .description("Architectural decision")
            .source(test_source())
            .relationships(vec![Relationship::new(
                ItemId::new_unchecked("SYSARCH-001"),
                RelationshipType::Justifies,
            )])
            .attributes(ItemAttributes::Adr {
                status: AdrStatus::Proposed,
                deciders: vec!["Alice".to_string(), "Bob".to_string()],
                supersedes: Vec::new(),
            })
            .build()
            .unwrap();

        let yaml = generate_frontmatter(&item);

        assert!(yaml.contains("status: proposed"));
        assert!(yaml.contains("deciders:"));
        assert!(yaml.contains("Alice"));
        assert!(yaml.contains("Bob"));
        assert!(yaml.contains("justifies:"));
        assert!(yaml.contains("SYSARCH-001"));
    }

    #[test]
    fn test_extract_frontmatter() {
        let doc = "---\nid: \"TEST\"\ntype: solution\n---\n# Heading\n\nBody content";
        let frontmatter = extract_frontmatter(doc);

        assert_eq!(frontmatter, "---\nid: \"TEST\"\ntype: solution\n---\n");
    }
}

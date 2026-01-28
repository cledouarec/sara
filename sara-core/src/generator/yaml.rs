//! YAML frontmatter generation using Tera templates.
//!
//! Generates YAML frontmatter by rendering the full document template
//! and extracting the frontmatter portion.

use super::markdown;
use crate::model::{InitData, ItemType, TypeFields};

/// Generate YAML frontmatter from domain data.
///
/// Uses Tera templates to ensure consistency with full document generation.
/// Renders the complete document and extracts just the frontmatter portion.
pub fn generate_frontmatter(
    item_type: ItemType,
    id: &str,
    name: &str,
    description: Option<&str>,
    fields: &TypeFields,
) -> String {
    let data = InitData {
        id: id.to_string(),
        name: name.to_string(),
        item_type,
        description: description.map(ToString::to_string),
        fields: fields.clone(),
    };

    let full_document = markdown::generate_document(&data);
    extract_frontmatter(&full_document)
}

/// Generate YAML frontmatter for editing (updates existing item).
pub fn generate_edit_frontmatter(
    item_id: &str,
    item_type: ItemType,
    name: &str,
    description: Option<&str>,
    fields: &TypeFields,
) -> String {
    generate_frontmatter(item_type, item_id, name, description, fields)
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
    use crate::model::AdrStatus;

    #[test]
    fn test_generate_frontmatter_solution() {
        let fields = TypeFields::default();
        let yaml = generate_frontmatter(
            ItemType::Solution,
            "SOL-001",
            "Test Solution",
            Some("A test solution"),
            &fields,
        );

        assert!(yaml.starts_with("---\n"));
        assert!(yaml.ends_with("---\n"));
        assert!(yaml.contains("id: \"SOL-001\""));
        assert!(yaml.contains("type: solution"));
        assert!(yaml.contains("name: \"Test Solution\""));
        assert!(yaml.contains("description: \"A test solution\""));
    }

    #[test]
    fn test_generate_frontmatter_use_case_with_refines() {
        let fields = TypeFields::new().with_refines(vec!["SOL-001".to_string()]);
        let yaml =
            generate_frontmatter(ItemType::UseCase, "UC-001", "Test Use Case", None, &fields);

        assert!(yaml.contains("refines:"));
        assert!(yaml.contains("SOL-001"));
    }

    #[test]
    fn test_generate_frontmatter_requirement_with_spec() {
        let fields = TypeFields::new()
            .with_specification("The system SHALL do X")
            .with_derives_from(vec!["SCEN-001".to_string()]);

        let yaml = generate_frontmatter(
            ItemType::SystemRequirement,
            "SYSREQ-001",
            "Test Requirement",
            None,
            &fields,
        );

        assert!(yaml.contains("specification:"));
        assert!(yaml.contains("derives_from:"));
        assert!(yaml.contains("SCEN-001"));
    }

    #[test]
    fn test_generate_frontmatter_architecture_with_platform() {
        let fields = TypeFields::new()
            .with_platform("AWS Lambda")
            .with_satisfies(vec!["SYSREQ-001".to_string()]);

        let yaml = generate_frontmatter(
            ItemType::SystemArchitecture,
            "SYSARCH-001",
            "Web Platform",
            None,
            &fields,
        );

        assert!(yaml.contains("platform:"));
        assert!(yaml.contains("satisfies:"));
        assert!(yaml.contains("SYSREQ-001"));
    }

    #[test]
    fn test_generate_frontmatter_adr() {
        let fields = TypeFields::new()
            .with_status(AdrStatus::Proposed)
            .with_deciders(vec!["Alice".to_string(), "Bob".to_string()])
            .with_justifies(vec!["SYSARCH-001".to_string()]);

        let yaml = generate_frontmatter(
            ItemType::ArchitectureDecisionRecord,
            "ADR-001",
            "Use Microservices",
            Some("Architectural decision"),
            &fields,
        );

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

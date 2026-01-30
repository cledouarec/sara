//! YAML frontmatter parsing.
//!
//! Converts YAML strings to domain types for the parser.

use std::path::Path;

use serde::Deserialize;

use crate::error::ParseError;
use crate::model::{AdrStatus, ItemType};

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

    /// Description (optional).
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

    /// ADR links (for SystemArchitecture, HW/SW DetailedDesign).
    #[serde(default)]
    pub justified_by: Option<Vec<String>>,

    /// ADR lifecycle status (required for ADR items).
    #[serde(default)]
    pub status: Option<AdrStatus>,

    /// ADR deciders (required for ADR items).
    #[serde(default)]
    pub deciders: Vec<String>,

    /// Design artifacts this ADR justifies (for ADR items).
    #[serde(default)]
    pub justifies: Vec<String>,

    /// Older ADRs this decision supersedes (for ADR items).
    #[serde(default)]
    pub supersedes: Vec<String>,

    /// Newer ADR that supersedes this one.
    #[serde(default)]
    pub superseded_by: Option<String>,
}

/// Parse YAML frontmatter string into RawFrontmatter.
pub fn parse_frontmatter(yaml_str: &str, file: &Path) -> Result<RawFrontmatter, ParseError> {
    serde_yaml::from_str(yaml_str).map_err(|e| ParseError::InvalidYaml {
        file: file.to_path_buf(),
        reason: e.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_solution() {
        let yaml = r#"
id: "SOL-001"
type: solution
name: "Test Solution"
description: "A test"
"#;
        let raw = parse_frontmatter(yaml, &PathBuf::from("test.md")).unwrap();
        assert_eq!(raw.id, "SOL-001");
        assert_eq!(raw.item_type, ItemType::Solution);
        assert_eq!(raw.name, "Test Solution");
    }

    #[test]
    fn test_parse_requirement_with_fields() {
        let yaml = r#"
id: "SYSREQ-001"
type: system_requirement
name: "Performance"
specification: "The system SHALL respond in 100ms"
derives_from:
  - "SCEN-001"
depends_on:
  - "SYSREQ-002"
"#;
        let raw = parse_frontmatter(yaml, &PathBuf::from("test.md")).unwrap();

        assert_eq!(
            raw.specification,
            Some("The system SHALL respond in 100ms".to_string())
        );
        assert_eq!(raw.derives_from, vec!["SCEN-001"]);
        assert_eq!(raw.depends_on, vec!["SYSREQ-002"]);
    }

    #[test]
    fn test_parse_adr() {
        let yaml = r#"
id: "ADR-001"
type: architecture_decision_record
name: "Use Microservices"
status: proposed
deciders:
  - "Alice"
  - "Bob"
justifies:
  - "SYSARCH-001"
"#;
        let raw = parse_frontmatter(yaml, &PathBuf::from("test.md")).unwrap();

        assert_eq!(raw.status, Some(AdrStatus::Proposed));
        assert_eq!(raw.deciders, vec!["Alice", "Bob"]);
        assert_eq!(raw.justifies, vec!["SYSARCH-001"]);
    }

    #[test]
    fn test_parse_architecture_with_platform() {
        let yaml = r#"
id: "SYSARCH-001"
type: system_architecture
name: "Web Platform"
platform: "AWS Lambda"
satisfies:
  - "SYSREQ-001"
"#;
        let raw = parse_frontmatter(yaml, &PathBuf::from("test.md")).unwrap();

        assert_eq!(raw.platform, Some("AWS Lambda".to_string()));
        assert_eq!(raw.satisfies, vec!["SYSREQ-001"]);
    }

    #[test]
    fn test_parse_invalid_yaml() {
        let yaml = "invalid: [unclosed";
        let result = parse_frontmatter(yaml, &PathBuf::from("test.md"));
        assert!(result.is_err());
    }
}

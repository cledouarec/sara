//! YAML frontmatter parsing and deserialization.
//!
//! Provides the input adapter for YAML frontmatter. Deserializes raw YAML
//! strings into `RawFrontmatter` and converts them to core model types
//! (`Relationship`, `ItemId`, etc.).

use std::path::Path;

use serde::Deserialize;

use crate::error::SaraError;
use crate::model::{
    AdrStatus, ItemId, ItemType, Relationship, RelationshipType,
};

/// Raw frontmatter structure for deserialization.
///
/// Represents the YAML frontmatter as it appears in Markdown files.
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

impl RawFrontmatter {
    /// Converts all relationship fields to a Vec of Relationships.
    #[must_use]
    pub fn to_relationships(&self) -> Vec<Relationship> {
        let mut rels = Vec::new();

        // Upstream relationships
        for id in &self.refines {
            rels.push(Relationship::new(
                ItemId::new_unchecked(id),
                RelationshipType::Refines,
            ));
        }
        for id in &self.derives_from {
            rels.push(Relationship::new(
                ItemId::new_unchecked(id),
                RelationshipType::DerivesFrom,
            ));
        }
        for id in &self.satisfies {
            rels.push(Relationship::new(
                ItemId::new_unchecked(id),
                RelationshipType::Satisfies,
            ));
        }
        for id in &self.justifies {
            rels.push(Relationship::new(
                ItemId::new_unchecked(id),
                RelationshipType::Justifies,
            ));
        }

        // Downstream relationships
        for id in &self.is_refined_by {
            rels.push(Relationship::new(
                ItemId::new_unchecked(id),
                RelationshipType::IsRefinedBy,
            ));
        }
        for id in &self.derives {
            rels.push(Relationship::new(
                ItemId::new_unchecked(id),
                RelationshipType::Derives,
            ));
        }
        for id in &self.is_satisfied_by {
            rels.push(Relationship::new(
                ItemId::new_unchecked(id),
                RelationshipType::IsSatisfiedBy,
            ));
        }
        if let Some(justified_by) = &self.justified_by {
            for id in justified_by {
                rels.push(Relationship::new(
                    ItemId::new_unchecked(id),
                    RelationshipType::IsJustifiedBy,
                ));
            }
        }

        rels
    }
}

/// Parses a raw YAML string into a `RawFrontmatter`.
pub fn parse_yaml_frontmatter(yaml: &str, file: &Path) -> Result<RawFrontmatter, SaraError> {
    serde_yaml::from_str(yaml).map_err(|e| SaraError::InvalidYaml {
        file: file.to_path_buf(),
        reason: e.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_yaml_frontmatter_solution() {
        let yaml = r#"
id: "SOL-001"
type: solution
name: "Test Solution"
description: "A test solution"
"#;
        let fm = parse_yaml_frontmatter(yaml, Path::new("test.md")).unwrap();
        assert_eq!(fm.id, "SOL-001");
        assert_eq!(fm.item_type, ItemType::Solution);
        assert_eq!(fm.name, "Test Solution");
        assert_eq!(fm.description, Some("A test solution".to_string()));
    }

    #[test]
    fn test_parse_yaml_frontmatter_with_relationships() {
        let yaml = r#"
id: "UC-001"
type: use_case
name: "Login"
refines:
  - "SOL-001"
"#;
        let fm = parse_yaml_frontmatter(yaml, Path::new("test.md")).unwrap();
        let rels = fm.to_relationships();
        assert_eq!(rels.len(), 1);
        assert_eq!(rels[0].to.as_str(), "SOL-001");
        assert_eq!(rels[0].relationship_type, RelationshipType::Refines);
    }

    #[test]
    fn test_parse_yaml_frontmatter_invalid() {
        let yaml = "not: valid: yaml: [";
        let result = parse_yaml_frontmatter(yaml, Path::new("test.md"));
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_yaml_frontmatter_adr() {
        let yaml = r#"
id: "ADR-001"
type: architecture_decision_record
name: "Use REST API"
status: proposed
deciders:
  - "Alice"
justifies:
  - "SYSARCH-001"
supersedes:
  - "ADR-000"
"#;
        let fm = parse_yaml_frontmatter(yaml, Path::new("test.md")).unwrap();
        assert_eq!(fm.status, Some(AdrStatus::Proposed));
        assert_eq!(fm.deciders, vec!["Alice"]);
        assert_eq!(fm.justifies, vec!["SYSARCH-001"]);
        assert_eq!(fm.supersedes, vec!["ADR-000"]);

        let rels = fm.to_relationships();
        assert_eq!(rels.len(), 1); // Only justifies as relationship
        assert_eq!(rels[0].relationship_type, RelationshipType::Justifies);
    }
}

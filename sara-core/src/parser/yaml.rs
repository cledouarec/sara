//! YAML frontmatter parsing and deserialization.
//!
//! Provides the input adapter for YAML frontmatter. Deserializes raw YAML
//! strings into `RawFrontmatter` and converts them to core model types
//! (`Relationship`, `ItemId`, etc.).

use std::path::Path;

use indexmap::IndexMap;
use serde::Deserialize;

use crate::error::SaraError;
use crate::model::{FieldValue, ItemId, ItemType, Relationship, RelationshipType};
use crate::schema::{FieldDef, FieldType};

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
    pub status: Option<String>,

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

    /// Remaining frontmatter entries, keyed by field name.
    ///
    /// Captures the values of fields declared by a custom schema that have no
    /// dedicated struct field above; read via [`Self::declared_field_value`].
    #[serde(flatten)]
    pub extra: IndexMap<String, serde_yaml::Value>,
}

/// Extends a relationship vector with relationships built from a slice of ID strings.
fn extend_rels(rels: &mut Vec<Relationship>, ids: &[String], rel_type: RelationshipType) {
    rels.extend(
        ids.iter()
            .map(|id| Relationship::new(ItemId::new_unchecked(id), rel_type)),
    );
}

impl RawFrontmatter {
    /// Returns the value of a schema-declared field as a typed [`FieldValue`].
    ///
    /// Fields with a dedicated struct member (`specification`, `platform`,
    /// `status`, `deciders`, `depends_on`, `supersedes`) are read from it;
    /// any other declared field is read from the flattened remainder of the
    /// frontmatter. Absent fields and empty lists yield `Ok(None)`.
    ///
    /// # Errors
    ///
    /// Returns a human-readable reason when the value does not match the
    /// declared type, e.g. an enum value outside the allowed set.
    pub fn declared_field_value(&self, field: &FieldDef) -> Result<Option<FieldValue>, String> {
        match field.name.as_str() {
            "specification" => Ok(self
                .specification
                .as_ref()
                .map(|s| FieldValue::Text(s.clone()))),
            "platform" => Ok(self.platform.as_ref().map(|s| FieldValue::Text(s.clone()))),
            "status" => self
                .status
                .as_ref()
                .map(|s| {
                    field_value_from_yaml(&serde_yaml::Value::String(s.clone()), &field.field_type)
                })
                .transpose()
                .map(Option::flatten),
            "deciders" => Ok(non_empty_list(
                self.deciders.iter().map(|s| FieldValue::Text(s.clone())),
            )),
            "depends_on" => Ok(non_empty_list(item_ref_values(&self.depends_on))),
            "supersedes" => Ok(non_empty_list(item_ref_values(&self.supersedes))),
            _ => self
                .extra
                .get(&field.name)
                .map(|value| field_value_from_yaml(value, &field.field_type))
                .transpose()
                .map(Option::flatten),
        }
    }

    /// Converts all relationship fields to a Vec of Relationships.
    #[must_use]
    pub fn to_relationships(&self) -> Vec<Relationship> {
        let mut rels = Vec::new();

        // Upstream relationships
        extend_rels(&mut rels, &self.refines, RelationshipType::Refines);
        extend_rels(&mut rels, &self.derives_from, RelationshipType::DerivesFrom);
        extend_rels(&mut rels, &self.satisfies, RelationshipType::Satisfies);
        extend_rels(&mut rels, &self.justifies, RelationshipType::Justifies);

        // Downstream relationships
        extend_rels(
            &mut rels,
            &self.is_refined_by,
            RelationshipType::IsRefinedBy,
        );
        extend_rels(&mut rels, &self.derives, RelationshipType::Derives);
        extend_rels(
            &mut rels,
            &self.is_satisfied_by,
            RelationshipType::IsSatisfiedBy,
        );
        if let Some(justified_by) = &self.justified_by {
            extend_rels(&mut rels, justified_by, RelationshipType::IsJustifiedBy);
        }

        // Peer relationships
        extend_rels(&mut rels, &self.supersedes, RelationshipType::Supersedes);
        if let Some(id) = &self.superseded_by {
            rels.push(Relationship::new(
                ItemId::new_unchecked(id),
                RelationshipType::IsSupersededBy,
            ));
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

/// Wraps the IDs of a string slice as item-reference field values.
fn item_ref_values(ids: &[String]) -> impl Iterator<Item = FieldValue> + '_ {
    ids.iter()
        .map(|id| FieldValue::ItemRef(ItemId::new_unchecked(id)))
}

/// Collects values into a list field value, mapping an empty list to `None`.
fn non_empty_list(values: impl Iterator<Item = FieldValue>) -> Option<FieldValue> {
    let list: Vec<FieldValue> = values.collect();
    if list.is_empty() {
        None
    } else {
        Some(FieldValue::List(list))
    }
}

/// Converts a raw YAML value to a [`FieldValue`] of the declared type.
///
/// Values whose shape does not match the declared type yield `Ok(None)`,
/// mirroring the historical tolerance of the parser; only enum values outside
/// the allowed set are reported as errors.
fn field_value_from_yaml(
    value: &serde_yaml::Value,
    field_type: &FieldType,
) -> Result<Option<FieldValue>, String> {
    match field_type {
        FieldType::Text => Ok(value.as_str().map(|s| FieldValue::Text(s.to_string()))),
        FieldType::Date => Ok(value.as_str().map(|s| FieldValue::Date(s.to_string()))),
        FieldType::ItemRef => Ok(value
            .as_str()
            .map(|s| FieldValue::ItemRef(ItemId::new_unchecked(s)))),
        FieldType::Enum { values } => match value.as_str() {
            Some(s) if values.iter().any(|v| v == s) => Ok(Some(FieldValue::Enum(s.to_string()))),
            Some(s) => Err(format!(
                "invalid value `{s}`, expected one of: {}",
                values.join(", ")
            )),
            None => Ok(None),
        },
        FieldType::List(inner) => {
            let Some(sequence) = value.as_sequence() else {
                return Ok(None);
            };
            let mut list = Vec::with_capacity(sequence.len());
            for entry in sequence {
                if let Some(converted) = field_value_from_yaml(entry, inner)? {
                    list.push(converted);
                }
            }
            Ok(non_empty_list(list.into_iter()))
        }
    }
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
        assert_eq!(fm.item_type, ItemType::SOLUTION);
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
        assert_eq!(fm.status.as_deref(), Some("proposed"));
        assert_eq!(fm.deciders, vec!["Alice"]);
        assert_eq!(fm.justifies, vec!["SYSARCH-001"]);
        assert_eq!(fm.supersedes, vec!["ADR-000"]);

        let rels = fm.to_relationships();
        assert_eq!(rels.len(), 2);
        assert_eq!(rels[0].relationship_type, RelationshipType::Justifies);
        assert_eq!(rels[1].relationship_type, RelationshipType::Supersedes);
    }
}

//! YAML frontmatter parsing and deserialization.
//!
//! Provides the input adapter for YAML frontmatter. Deserializes raw YAML
//! strings into `RawFrontmatter` and converts them to core model types
//! (`Relationship`, `ItemId`, etc.) by resolving every field and relation
//! name against the active schema.

use std::path::Path;

use indexmap::IndexMap;
use serde::Deserialize;

use crate::error::SaraError;
use crate::model::{FieldValue, ItemId, ItemType, Relationship, RelationshipType};
use crate::schema::{self, FieldDef, FieldType};

/// Raw frontmatter structure for deserialization.
///
/// Represents the YAML frontmatter as it appears in Markdown files. Only the
/// core identity fields have a dedicated member; every schema-declared field
/// or relation is captured by name in [`Self::extra`] and resolved against
/// the active schema, so custom types and relations parse exactly like the
/// built-in ones.
///
/// The serde member names must mirror the canonical core field names
/// (`crate::model::{FIELD_ID, FIELD_TYPE, FIELD_NAME, FIELD_DESCRIPTION}`) —
/// serde attributes cannot reference constants, so a guard test pins the
/// correspondence.
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

    /// Remaining frontmatter entries, keyed by field or relation name.
    #[serde(flatten)]
    pub extra: IndexMap<String, serde_yaml::Value>,
}

impl RawFrontmatter {
    /// Returns the value of a schema-declared field as a typed [`FieldValue`].
    ///
    /// Absent fields and empty lists yield `Ok(None)`.
    ///
    /// # Errors
    ///
    /// Returns a human-readable reason when the value does not match the
    /// declared type, e.g. an enum value outside the allowed set.
    pub fn declared_field_value(&self, field: &FieldDef) -> Result<Option<FieldValue>, String> {
        self.extra
            .get(&field.name)
            .map(|value| field_value_from_yaml(value, &field.field_type))
            .transpose()
            .map(Option::flatten)
    }

    /// Converts all relation entries to a Vec of Relationships.
    ///
    /// Each relation of the active schema is read from the frontmatter entry
    /// carrying its id; both a single id string and a sequence of ids are
    /// accepted. Entries that match no relation of the schema are ignored.
    #[must_use]
    pub fn to_relationships(&self) -> Vec<Relationship> {
        let mut rels = Vec::new();

        for def in &schema::active().relations {
            let Some(value) = self.extra.get(&def.id) else {
                continue;
            };
            let Some(rel_type) = RelationshipType::from_id(&def.id) else {
                continue;
            };
            match value {
                serde_yaml::Value::String(id) => {
                    rels.push(Relationship::new(ItemId::new_unchecked(id), rel_type));
                }
                serde_yaml::Value::Sequence(ids) => {
                    rels.extend(ids.iter().filter_map(|entry| {
                        let id = entry.as_str()?;
                        Some(Relationship::new(ItemId::new_unchecked(id), rel_type))
                    }));
                }
                _ => {}
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

    use crate::schema::builtin;

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
        assert_eq!(fm.item_type, builtin::SOLUTION);
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
        assert_eq!(rels[0].relationship_type, builtin::REFINES);
    }

    #[test]
    fn test_parse_yaml_frontmatter_single_id_relationship() {
        let yaml = r#"
id: "UC-001"
type: use_case
name: "Login"
refines: "SOL-001"
"#;
        let fm = parse_yaml_frontmatter(yaml, Path::new("test.md")).unwrap();
        let rels = fm.to_relationships();
        assert_eq!(rels.len(), 1);
        assert_eq!(rels[0].to.as_str(), "SOL-001");
        assert_eq!(rels[0].relationship_type, builtin::REFINES);
    }

    #[test]
    fn test_core_field_names_match_model_constants() {
        use crate::model::{FIELD_DESCRIPTION, FIELD_ID, FIELD_NAME, FIELD_TYPE};

        let yaml = format!(
            "{FIELD_ID}: \"SOL-001\"\n{FIELD_TYPE}: solution\n{FIELD_NAME}: \"Named\"\n{FIELD_DESCRIPTION}: \"Described\"\n"
        );
        let fm = parse_yaml_frontmatter(&yaml, Path::new("test.md")).unwrap();
        assert_eq!(fm.id, "SOL-001");
        assert_eq!(fm.item_type, builtin::SOLUTION);
        assert_eq!(fm.name, "Named");
        assert_eq!(fm.description, Some("Described".to_string()));
        assert!(
            fm.extra.is_empty(),
            "core fields must not leak into the flattened remainder"
        );
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

        let fields = builtin::ARCHITECTURE_DECISION_RECORD.declared_fields();
        let status_field = fields.iter().find(|f| f.name == "status").unwrap();
        let deciders_field = fields.iter().find(|f| f.name == "deciders").unwrap();
        assert_eq!(
            fm.declared_field_value(status_field).unwrap(),
            Some(FieldValue::Enum("proposed".to_string()))
        );
        assert_eq!(
            fm.declared_field_value(deciders_field).unwrap(),
            Some(FieldValue::List(vec![FieldValue::Text(
                "Alice".to_string()
            )]))
        );

        let rels = fm.to_relationships();
        assert_eq!(rels.len(), 2);
        assert_eq!(rels[0].relationship_type, builtin::JUSTIFIES);
        assert_eq!(rels[1].relationship_type, builtin::SUPERSEDES);
    }
}

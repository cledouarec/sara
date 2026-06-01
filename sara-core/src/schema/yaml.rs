//! YAML loading and structural validation for [`Schema`].

use std::path::Path;

use super::{FieldType, Schema};
use crate::error::SaraError;

impl Schema {
    /// Parses a schema from a YAML string.
    ///
    /// `source` is only used for error reporting (mirrors the convention of
    /// `parser::yaml::parse_yaml_frontmatter`).
    ///
    /// # Errors
    ///
    /// Returns [`SaraError::InvalidYaml`] if the YAML is syntactically
    /// invalid, or [`SaraError::InvalidConfig`] if the schema is internally
    /// inconsistent (see [`Schema::validate`]).
    pub fn from_yaml_str(yaml: &str, source: &Path) -> Result<Self, SaraError> {
        let schema: Schema = serde_yaml::from_str(yaml).map_err(|e| SaraError::InvalidYaml {
            file: source.to_path_buf(),
            reason: e.to_string(),
        })?;
        schema.validate(source)?;
        Ok(schema)
    }

    /// Loads and validates a schema from a YAML file.
    ///
    /// # Errors
    ///
    /// Returns [`SaraError::ConfigRead`] if the file cannot be read, plus any
    /// error from [`Schema::from_yaml_str`].
    pub fn from_path(path: &Path) -> Result<Self, SaraError> {
        let yaml = std::fs::read_to_string(path).map_err(|e| SaraError::ConfigRead {
            path: path.to_path_buf(),
            reason: e.to_string(),
        })?;
        Self::from_yaml_str(&yaml, path)
    }

    /// Validates internal consistency of the schema.
    ///
    /// Checks: relation inverses are symmetric, every referenced relation,
    /// parent and target type id exists, and `Enum` fields list at least one
    /// value.
    ///
    /// # Errors
    ///
    /// Returns [`SaraError::InvalidConfig`] describing the first inconsistency.
    pub fn validate(&self, source: &Path) -> Result<(), SaraError> {
        let invalid = |reason: String| SaraError::InvalidConfig {
            path: source.to_path_buf(),
            reason,
        };

        // Relation inverses must be symmetric.
        for rel in &self.relations {
            let Some(inv) = self.relation(&rel.inverse) else {
                return Err(invalid(format!(
                    "relation '{}' has unknown inverse '{}'",
                    rel.id, rel.inverse
                )));
            };
            if inv.inverse != rel.id {
                return Err(invalid(format!(
                    "relation '{}' inverse '{}' is not symmetric",
                    rel.id, rel.inverse
                )));
            }
        }

        // Type-level references must resolve.
        for def in &self.item_types {
            for parent in &def.parent_types {
                if self.item_type(parent).is_none() {
                    return Err(invalid(format!(
                        "type '{}' references unknown parent type '{}'",
                        def.id, parent
                    )));
                }
            }
            for target in &def.allowed_targets {
                if self.relation(&target.relation).is_none() {
                    return Err(invalid(format!(
                        "type '{}' references unknown relation '{}'",
                        def.id, target.relation
                    )));
                }
                for t in &target.targets {
                    if self.item_type(t).is_none() {
                        return Err(invalid(format!(
                            "type '{}' relation '{}' references unknown target type '{}'",
                            def.id, target.relation, t
                        )));
                    }
                }
            }
            for field in &def.fields {
                Self::validate_field_type(&field.field_type, &def.id, &field.name, &invalid)?;
            }
        }

        Ok(())
    }

    /// Recursively validates a field type declaration.
    fn validate_field_type(
        field_type: &FieldType,
        type_id: &str,
        field_name: &str,
        invalid: &impl Fn(String) -> SaraError,
    ) -> Result<(), SaraError> {
        match field_type {
            FieldType::Enum { values } if values.is_empty() => Err(invalid(format!(
                "type '{type_id}' field '{field_name}' is an enum with no values"
            ))),
            FieldType::List(inner) => {
                Self::validate_field_type(inner, type_id, field_name, invalid)
            }
            FieldType::Text | FieldType::Enum { .. } | FieldType::ItemRef | FieldType::Date => {
                Ok(())
            }
        }
    }
}

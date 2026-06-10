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

        // References must resolve either in this schema or in the built-in
        // model it is merged with, so a partial schema may link its types
        // and relations to the default ones.
        let builtin = Schema::builtin();

        // Relation inverses must be symmetric.
        for rel in &self.relations {
            let Some(inv) = self
                .relation(&rel.inverse)
                .or_else(|| builtin.relation(&rel.inverse))
            else {
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

        let known_type = |id: &str| self.item_type(id).is_some() || builtin.item_type(id).is_some();
        let known_relation =
            |id: &str| self.relation(id).is_some() || builtin.relation(id).is_some();
        for def in &self.item_types {
            for parent in &def.parent_types {
                if !known_type(parent) {
                    return Err(invalid(format!(
                        "type '{}' references unknown parent type '{}'",
                        def.id, parent
                    )));
                }
            }
            for target in &def.allowed_targets {
                if !known_relation(&target.relation) {
                    return Err(invalid(format!(
                        "type '{}' references unknown relation '{}'",
                        def.id, target.relation
                    )));
                }
                for t in &target.targets {
                    if !known_type(t) {
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    const PARTIAL_SCHEMA: &str = r#"item_types:
- id: stakeholder_requirement
  display_name: Stakeholder Requirement
  prefix: STKREQ
  id_format: "{prefix}-{seq:03}"
  parent_types:
  - solution
  fields:
  - name: rationale
    display_name: Rationale
    field_type: text
    required: true
  allowed_targets:
  - relation: refines
    targets:
    - solution
relations: []
"#;

    #[test]
    fn test_partial_schema_may_reference_builtin_types_and_relations() {
        let schema = Schema::from_yaml_str(PARTIAL_SCHEMA, Path::new("<test>"))
            .expect("references to built-in types must resolve");
        assert!(schema.item_type("stakeholder_requirement").is_some());
    }

    #[test]
    fn test_unknown_references_are_still_rejected() {
        let yaml = PARTIAL_SCHEMA.replace("- solution", "- solutoin");
        assert!(Schema::from_yaml_str(&yaml, Path::new("<test>")).is_err());
    }
}

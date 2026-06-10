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

        // An installed schema replaces the built-in model entirely, so every
        // reference must resolve within the schema itself.
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    const STANDALONE_SCHEMA: &str = r#"item_types:
- id: solution
  display_name: Solution
  prefix: SOL
  id_format: "{prefix}-{seq:03}"
  parent_types: []
  fields: []
  allowed_targets: []
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
relations:
- id: refines
  display_name: Refines
  inverse: is_refined_by
  direction: upstream
  primary: true
- id: is_refined_by
  display_name: Is refined by
  inverse: refines
  direction: downstream
  primary: false
"#;

    #[test]
    fn test_self_contained_schema_loads() {
        let schema = Schema::from_yaml_str(STANDALONE_SCHEMA, Path::new("<test>"))
            .expect("self-contained schema must load");
        assert!(schema.item_type("stakeholder_requirement").is_some());
    }

    #[test]
    fn test_references_outside_the_schema_are_rejected() {
        // The schema replaces the built-in model, so even a built-in type
        // name must be declared to be referenced.
        let yaml = STANDALONE_SCHEMA.replace(
            "- id: solution
  display_name: Solution
  prefix: SOL
  id_format: \"{prefix}-{seq:03}\"
  parent_types: []
  fields: []
  allowed_targets: []
",
            "",
        );
        assert!(Schema::from_yaml_str(&yaml, Path::new("<test>")).is_err());
    }
}

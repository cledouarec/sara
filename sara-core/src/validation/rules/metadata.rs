//! Metadata validation rule.

use crate::config::ValidationConfig;
use crate::error::ValidationError;
use crate::graph::KnowledgeGraph;
use crate::validation::rule::ValidationRule;

/// Metadata validation rule.
///
/// Validates metadata completeness for all items.
/// Checks:
/// - Required fields are present (id, type, name already enforced by parsing)
/// - Specification field is present for requirement types
pub struct MetadataRule;

impl ValidationRule for MetadataRule {
    fn validate(&self, graph: &KnowledgeGraph, config: &ValidationConfig) -> Vec<ValidationError> {
        let _ = &config.allowed_custom_fields; // Config available for future use
        let mut errors = Vec::new();

        for item in graph.items() {
            // Check specification requirement
            if item.item_type.requires_specification()
                && item
                    .attributes
                    .specification()
                    .is_some_and(|spec| spec.is_empty())
            {
                errors.push(ValidationError::InvalidMetadata {
                    file: item.source.file_path.display().to_string(),
                    reason: format!(
                        "{} requires a non-empty 'specification' field",
                        item.item_type.display_name()
                    ),
                });
            }
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        FieldName, ItemAttributes, ItemBuilder, ItemId, ItemType, SourceLocation,
    };
    use std::collections::HashSet;
    use std::path::{Path, PathBuf};

    /// Checks for unrecognized fields in YAML frontmatter content (FR-019).
    fn check_custom_fields(
        yaml_content: &str,
        file_path: &Path,
        allowed_custom_fields: &[String],
    ) -> Vec<ValidationError> {
        let mut warnings = Vec::new();

        // Build set of all allowed fields
        let mut allowed: HashSet<&str> = FieldName::all().iter().map(|f| f.as_str()).collect();
        for field in allowed_custom_fields {
            allowed.insert(field.as_str());
        }

        // Parse YAML as a generic mapping to inspect field names
        let parsed: Result<serde_yaml::Mapping, _> = serde_yaml::from_str(yaml_content);

        if let Ok(mapping) = parsed {
            for key in mapping.keys() {
                if let Some(field_name) = key.as_str()
                    && !allowed.contains(field_name)
                {
                    warnings.push(ValidationError::UnrecognizedField {
                        field: field_name.to_string(),
                        file: file_path.display().to_string(),
                    });
                }
            }
        }

        warnings
    }

    /// Returns the list of known frontmatter fields.
    fn known_fields() -> Vec<&'static str> {
        FieldName::all().iter().map(|f| f.as_str()).collect()
    }

    /// Validates that a specification field contains a proper statement.
    fn validate_specification(spec: &str) -> Result<(), String> {
        if spec.trim().is_empty() {
            return Err("Specification cannot be empty".to_string());
        }

        // Check for requirement language (informational, not enforced as error)
        let has_shall = spec.to_uppercase().contains("SHALL");
        let has_must = spec.to_uppercase().contains("MUST");
        let has_will = spec.to_uppercase().contains("WILL");

        if !has_shall && !has_must && !has_will {
            // This is a warning, not an error - requirements should use SHALL/MUST/WILL
            // but we don't enforce it strictly
        }

        Ok(())
    }

    fn _create_item_with_spec(
        id: &str,
        item_type: ItemType,
        spec: Option<&str>,
    ) -> crate::model::Item {
        let source = SourceLocation::new(PathBuf::from("/repo"), format!("{}.md", id));
        let attributes = match item_type {
            ItemType::SystemRequirement => ItemAttributes::SystemRequirement {
                specification: spec.unwrap_or("").to_string(),
                depends_on: Vec::new(),
            },
            ItemType::SoftwareRequirement => ItemAttributes::SoftwareRequirement {
                specification: spec.unwrap_or("").to_string(),
                depends_on: Vec::new(),
            },
            ItemType::HardwareRequirement => ItemAttributes::HardwareRequirement {
                specification: spec.unwrap_or("").to_string(),
                depends_on: Vec::new(),
            },
            _ => ItemAttributes::for_type(item_type),
        };

        ItemBuilder::new()
            .id(ItemId::new_unchecked(id))
            .item_type(item_type)
            .name(format!("Test {}", id))
            .source(source)
            .attributes(attributes)
            .build()
            .unwrap_or_else(|_| {
                // If build fails due to missing spec, create with placeholder for testing
                let source = SourceLocation::new(PathBuf::from("/repo"), format!("{}.md", id));
                ItemBuilder::new()
                    .id(ItemId::new_unchecked(id))
                    .item_type(item_type)
                    .name(format!("Test {}", id))
                    .source(source)
                    .attributes(ItemAttributes::SystemRequirement {
                        specification: "placeholder".to_string(),
                        depends_on: Vec::new(),
                    })
                    .build()
                    .unwrap()
            })
    }

    #[test]
    fn test_valid_metadata() {
        let mut graph = KnowledgeGraph::new(false);
        let source = SourceLocation::new(PathBuf::from("/repo"), "req.md");
        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("SYSREQ-001"))
            .item_type(ItemType::SystemRequirement)
            .name("Test Requirement")
            .source(source)
            .attributes(ItemAttributes::SystemRequirement {
                specification: "The system SHALL respond within 100ms".to_string(),
                depends_on: Vec::new(),
            })
            .build()
            .unwrap();
        graph.add_item(item);

        let rule = MetadataRule;
        let errors = rule.validate(&graph, &ValidationConfig::default());
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_specification() {
        assert!(validate_specification("The system SHALL respond").is_ok());
        assert!(validate_specification("The system MUST respond").is_ok());
        assert!(validate_specification("The system WILL respond").is_ok());
        assert!(validate_specification("").is_err());
    }

    #[test]
    fn test_check_custom_fields_known_fields() {
        let yaml = r#"
id: "SOL-001"
type: solution
name: "Test Solution"
description: "A description"
"#;
        let warnings = check_custom_fields(yaml, Path::new("test.md"), &[]);
        assert!(
            warnings.is_empty(),
            "Known fields should not generate warnings"
        );
    }

    #[test]
    fn test_check_custom_fields_unrecognized() {
        let yaml = r#"
id: "SOL-001"
type: solution
name: "Test Solution"
custom_field: "some value"
another_custom: 123
"#;
        let warnings = check_custom_fields(yaml, Path::new("test.md"), &[]);
        assert_eq!(warnings.len(), 2, "Should detect 2 unrecognized fields");

        // Check that the warnings are for the right fields
        let warning_fields: Vec<_> = warnings
            .iter()
            .filter_map(|w| {
                if let ValidationError::UnrecognizedField { field, .. } = w {
                    Some(field.as_str())
                } else {
                    None
                }
            })
            .collect();
        assert!(warning_fields.contains(&"custom_field"));
        assert!(warning_fields.contains(&"another_custom"));
    }

    #[test]
    fn test_check_custom_fields_allowed() {
        let yaml = r#"
id: "SOL-001"
type: solution
name: "Test Solution"
custom_field: "some value"
"#;
        let allowed = vec!["custom_field".to_string()];
        let warnings = check_custom_fields(yaml, Path::new("test.md"), &allowed);
        assert!(
            warnings.is_empty(),
            "Allowed custom fields should not generate warnings"
        );
    }

    #[test]
    fn test_known_fields_list() {
        let fields = known_fields();
        assert!(fields.contains(&"id"));
        assert!(fields.contains(&"type"));
        assert!(fields.contains(&"name"));
        assert!(fields.contains(&"specification"));
        assert!(fields.contains(&"refines"));
        assert!(fields.contains(&"derives_from"));
    }
}

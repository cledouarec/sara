//! Metadata validation rule.

use crate::config::ValidationConfig;
use crate::error::ValidationError;
use crate::graph::KnowledgeGraph;
use crate::model::Item;
use crate::validation::rule::ValidationRule;

/// RFC2119 keywords that should be present in requirement specifications.
/// See: <https://www.ietf.org/rfc/rfc2119.txt>
const RFC2119_KEYWORDS: &[&str] = &[
    "MUST",
    "MUST NOT",
    "REQUIRED",
    "SHALL",
    "SHALL NOT",
    "SHOULD",
    "SHOULD NOT",
    "RECOMMENDED",
    "MAY",
    "OPTIONAL",
];

/// Metadata validation rule.
///
/// Validates metadata completeness for all items.
/// Checks:
/// - Required fields are present (id, type, name already enforced by parsing)
/// - Specification field is present and non-empty for requirement types
/// - Specification contains at least one RFC2119 keyword
///
/// This rule supports pre-validation (fail-fast) since it only examines
/// individual items without requiring graph context.
pub struct MetadataRule;

impl ValidationRule for MetadataRule {
    fn pre_validate(&self, items: &[Item], _config: &ValidationConfig) -> Vec<ValidationError> {
        items.iter().flat_map(validate_item_metadata).collect()
    }

    fn validate(&self, graph: &KnowledgeGraph, _config: &ValidationConfig) -> Vec<ValidationError> {
        graph.items().flat_map(validate_item_metadata).collect()
    }
}

/// Validates metadata for a single item.
fn validate_item_metadata(item: &Item) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // Check specification requirement
    if item.item_type.requires_specification() {
        match item.attributes.specification() {
            Some(spec) if spec.is_empty() => {
                errors.push(ValidationError::InvalidMetadata {
                    file: item.source.file_path.display().to_string(),
                    reason: format!(
                        "{} requires a non-empty 'specification' field",
                        item.item_type.display_name()
                    ),
                });
            }
            Some(spec) if !contains_rfc2119_keyword(spec) => {
                errors.push(ValidationError::InvalidMetadata {
                    file: item.source.file_path.display().to_string(),
                    reason: format!(
                        "{} specification must contain at least one RFC2119 keyword (MUST, SHALL, SHOULD, etc.)",
                        item.item_type.display_name()
                    ),
                });
            }
            _ => {}
        }
    }

    errors
}

/// Checks if a specification text contains at least one RFC2119 keyword.
fn contains_rfc2119_keyword(text: &str) -> bool {
    let upper = text.to_uppercase();
    RFC2119_KEYWORDS.iter().any(|keyword| {
        // Check for whole word match to avoid false positives
        // e.g., "MUST" shouldn't match in "MUSTARD"
        upper
            .split(|c: char| !c.is_alphabetic())
            .any(|word| word == *keyword)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::KnowledgeGraphBuilder;
    use crate::model::{ItemAttributes, ItemBuilder, ItemId, ItemType, SourceLocation};
    use std::path::PathBuf;

    fn create_item_with_spec(id: &str, item_type: ItemType, spec: &str) -> crate::model::Item {
        let source = SourceLocation::new(PathBuf::from("/repo"), format!("{}.md", id));
        let attributes = match item_type {
            ItemType::SystemRequirement => ItemAttributes::SystemRequirement {
                specification: spec.to_string(),
                depends_on: Vec::new(),
            },
            ItemType::SoftwareRequirement => ItemAttributes::SoftwareRequirement {
                specification: spec.to_string(),
                depends_on: Vec::new(),
            },
            ItemType::HardwareRequirement => ItemAttributes::HardwareRequirement {
                specification: spec.to_string(),
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
            .unwrap()
    }

    #[test]
    fn test_valid_metadata_with_rfc2119_keyword() {
        let item = create_item_with_spec(
            "SYSREQ-001",
            ItemType::SystemRequirement,
            "The system SHALL respond within 100ms",
        );
        let graph = KnowledgeGraphBuilder::new().add_item(item).build().unwrap();

        let rule = MetadataRule;
        let errors = rule.validate(&graph, &ValidationConfig::default());
        assert!(errors.is_empty());
    }

    #[test]
    fn test_empty_specification_fails() {
        let item = create_item_with_spec("SYSREQ-001", ItemType::SystemRequirement, "");
        let graph = KnowledgeGraphBuilder::new().add_item(item).build().unwrap();

        let rule = MetadataRule;
        let errors = rule.validate(&graph, &ValidationConfig::default());
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            &errors[0],
            ValidationError::InvalidMetadata { reason, .. } if reason.contains("non-empty")
        ));
    }

    #[test]
    fn test_specification_without_rfc2119_keyword_fails() {
        let item = create_item_with_spec(
            "SYSREQ-001",
            ItemType::SystemRequirement,
            "The system responds within 100ms",
        );
        let graph = KnowledgeGraphBuilder::new().add_item(item).build().unwrap();

        let rule = MetadataRule;
        let errors = rule.validate(&graph, &ValidationConfig::default());
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            &errors[0],
            ValidationError::InvalidMetadata { reason, .. } if reason.contains("RFC2119")
        ));
    }

    #[test]
    fn test_contains_rfc2119_keyword() {
        // Valid specifications with RFC2119 keywords
        assert!(contains_rfc2119_keyword("The system MUST respond"));
        assert!(contains_rfc2119_keyword("The system MUST NOT fail"));
        assert!(contains_rfc2119_keyword("The system SHALL respond"));
        assert!(contains_rfc2119_keyword("The system SHALL NOT fail"));
        assert!(contains_rfc2119_keyword("The system SHOULD respond"));
        assert!(contains_rfc2119_keyword("The system SHOULD NOT fail"));
        assert!(contains_rfc2119_keyword("This feature is REQUIRED"));
        assert!(contains_rfc2119_keyword("This feature is RECOMMENDED"));
        assert!(contains_rfc2119_keyword("This feature is OPTIONAL"));
        assert!(contains_rfc2119_keyword("The system MAY respond"));

        // Case insensitivity
        assert!(contains_rfc2119_keyword("The system must respond"));
        assert!(contains_rfc2119_keyword("The system Must respond"));

        // Invalid specifications without RFC2119 keywords
        assert!(!contains_rfc2119_keyword("The system responds"));
        assert!(!contains_rfc2119_keyword("The system will respond"));
        assert!(!contains_rfc2119_keyword(""));

        // Should not match partial words
        assert!(!contains_rfc2119_keyword("MUSTARD is a condiment"));
        assert!(!contains_rfc2119_keyword("MAYONNAISE is also a condiment"));
    }

    #[test]
    fn test_all_rfc2119_keywords_accepted() {
        let keywords = [
            "MUST",
            "MUST NOT",
            "REQUIRED",
            "SHALL",
            "SHALL NOT",
            "SHOULD",
            "SHOULD NOT",
            "RECOMMENDED",
            "MAY",
            "OPTIONAL",
        ];

        for keyword in keywords {
            let spec = format!("The system {} do something", keyword);
            assert!(
                contains_rfc2119_keyword(&spec),
                "Keyword '{}' should be accepted",
                keyword
            );
        }
    }

    #[test]
    fn test_pre_validate_valid_items() {
        let items = vec![create_item_with_spec(
            "SYSREQ-001",
            ItemType::SystemRequirement,
            "The system SHALL respond within 100ms",
        )];

        let rule = MetadataRule;
        let errors = rule.pre_validate(&items, &ValidationConfig::default());
        assert!(errors.is_empty());
    }

    #[test]
    fn test_pre_validate_empty_specification() {
        let items = vec![create_item_with_spec(
            "SYSREQ-001",
            ItemType::SystemRequirement,
            "",
        )];

        let rule = MetadataRule;
        let errors = rule.pre_validate(&items, &ValidationConfig::default());
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            &errors[0],
            ValidationError::InvalidMetadata { reason, .. } if reason.contains("non-empty")
        ));
    }

    #[test]
    fn test_pre_validate_missing_rfc2119_keyword() {
        let items = vec![create_item_with_spec(
            "SYSREQ-001",
            ItemType::SystemRequirement,
            "The system responds within 100ms",
        )];

        let rule = MetadataRule;
        let errors = rule.pre_validate(&items, &ValidationConfig::default());
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            &errors[0],
            ValidationError::InvalidMetadata { reason, .. } if reason.contains("RFC2119")
        ));
    }

    #[test]
    fn test_pre_validate_non_requirement_type() {
        // Solution type doesn't require specification
        let source = SourceLocation::new(PathBuf::from("/repo"), "SOL-001.md");
        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("SOL-001"))
            .item_type(ItemType::Solution)
            .name("Test Solution")
            .source(source)
            .attributes(ItemAttributes::for_type(ItemType::Solution))
            .build()
            .unwrap();

        let rule = MetadataRule;
        let errors = rule.pre_validate(&[item], &ValidationConfig::default());
        assert!(
            errors.is_empty(),
            "Solution should not require specification"
        );
    }

    #[test]
    fn test_pre_validate_multiple_items() {
        let items = vec![
            create_item_with_spec(
                "SYSREQ-001",
                ItemType::SystemRequirement,
                "The system SHALL respond within 100ms",
            ),
            create_item_with_spec(
                "SYSREQ-002",
                ItemType::SystemRequirement,
                "Missing keyword here", // Invalid
            ),
            create_item_with_spec(
                "SYSREQ-003",
                ItemType::SystemRequirement,
                "The system MUST be secure",
            ),
        ];

        let rule = MetadataRule;
        let errors = rule.pre_validate(&items, &ValidationConfig::default());
        assert_eq!(errors.len(), 1, "Should detect one invalid item");
        assert!(matches!(
            &errors[0],
            ValidationError::InvalidMetadata { reason, .. } if reason.contains("RFC2119")
        ));
    }
}

//! Metadata validation rule.

use crate::config::ValidationConfig;
use crate::error::SaraError;
use crate::graph::KnowledgeGraph;
use crate::model::{FieldValue, Item};
use crate::validation::rule::ValidationRule;

/// Field whose text is checked for RFC2119 requirement keywords.
const FIELD_SPECIFICATION: &str = "specification";

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
/// - Every field the schema marks as required carries a non-empty value
/// - A specification text contains at least one RFC2119 keyword
///
/// This rule supports pre-validation (fail-fast) since it only examines
/// individual items without requiring graph context.
pub struct MetadataRule;

impl ValidationRule for MetadataRule {
    fn pre_validate(&self, items: &[Item], _config: &ValidationConfig) -> Vec<SaraError> {
        items.iter().flat_map(validate_item_metadata).collect()
    }

    fn validate(&self, graph: &KnowledgeGraph, _config: &ValidationConfig) -> Vec<SaraError> {
        graph.items().flat_map(validate_item_metadata).collect()
    }
}

/// Validates metadata for a single item.
fn validate_item_metadata(item: &Item) -> Vec<SaraError> {
    let mut errors = Vec::new();

    // Every required declared field must carry a non-empty value.
    for field in item
        .item_type
        .declared_fields()
        .iter()
        .filter(|f| f.required)
    {
        if item.attributes.get(&field.name).is_none_or(is_empty_value) {
            errors.push(SaraError::InvalidMetadata {
                file: item.source.file_path.display().to_string(),
                reason: format!(
                    "{} requires a non-empty '{}' field",
                    item.item_type.display_name(),
                    field.name
                ),
            });
        }
    }

    // Requirement-writing quality: a specification text must state its
    // obligation level with an RFC2119 keyword.
    if let Some(FieldValue::Text(spec)) = item.attributes.get(FIELD_SPECIFICATION)
        && !spec.is_empty()
        && !contains_rfc2119_keyword(spec)
    {
        errors.push(SaraError::InvalidMetadata {
            file: item.source.file_path.display().to_string(),
            reason: format!(
                "{} specification must contain at least one RFC2119 keyword (MUST, SHALL, SHOULD, etc.)",
                item.item_type.display_name()
            ),
        });
    }

    errors
}

/// Returns true when a field value carries no information.
fn is_empty_value(value: &FieldValue) -> bool {
    match value {
        FieldValue::Text(s) | FieldValue::Enum(s) | FieldValue::Date(s) => s.is_empty(),
        FieldValue::ItemRef(id) => id.as_str().is_empty(),
        FieldValue::List(values) => values.is_empty(),
    }
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
    use std::path::PathBuf;

    use super::*;

    use crate::graph::KnowledgeGraphBuilder;
    use crate::model::{ItemBuilder, ItemId, ItemType, SourceLocation};
    use crate::schema::builtin;

    fn create_item_with_spec(id: &str, item_type: ItemType, spec: &str) -> crate::model::Item {
        let source = SourceLocation::new(PathBuf::from("/repo"), format!("{}.md", id));
        let mut builder = ItemBuilder::new()
            .id(ItemId::new_unchecked(id))
            .item_type(item_type)
            .name(format!("Test {}", id))
            .source(source);

        if matches!(
            item_type,
            builtin::SYSTEM_REQUIREMENT
                | builtin::SOFTWARE_REQUIREMENT
                | builtin::HARDWARE_REQUIREMENT
        ) {
            builder = builder.attribute("specification", FieldValue::text(spec));
        }

        builder.build().unwrap()
    }

    #[test]
    fn test_valid_metadata_with_rfc2119_keyword() {
        let item = create_item_with_spec(
            "SYSREQ-001",
            builtin::SYSTEM_REQUIREMENT,
            "The system SHALL respond within 100ms",
        );
        let graph = KnowledgeGraphBuilder::new().add_item(item).build().unwrap();

        let rule = MetadataRule;
        let errors = rule.validate(&graph, &ValidationConfig::default());
        assert!(errors.is_empty());
    }

    #[test]
    fn test_empty_specification_fails() {
        let item = create_item_with_spec("SYSREQ-001", builtin::SYSTEM_REQUIREMENT, "");
        let graph = KnowledgeGraphBuilder::new().add_item(item).build().unwrap();

        let rule = MetadataRule;
        let errors = rule.validate(&graph, &ValidationConfig::default());
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            &errors[0],
            SaraError::InvalidMetadata { reason, .. } if reason.contains("non-empty")
        ));
    }

    #[test]
    fn test_specification_without_rfc2119_keyword_fails() {
        let item = create_item_with_spec(
            "SYSREQ-001",
            builtin::SYSTEM_REQUIREMENT,
            "The system responds within 100ms",
        );
        let graph = KnowledgeGraphBuilder::new().add_item(item).build().unwrap();

        let rule = MetadataRule;
        let errors = rule.validate(&graph, &ValidationConfig::default());
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            &errors[0],
            SaraError::InvalidMetadata { reason, .. } if reason.contains("RFC2119")
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
            builtin::SYSTEM_REQUIREMENT,
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
            builtin::SYSTEM_REQUIREMENT,
            "",
        )];

        let rule = MetadataRule;
        let errors = rule.pre_validate(&items, &ValidationConfig::default());
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            &errors[0],
            SaraError::InvalidMetadata { reason, .. } if reason.contains("non-empty")
        ));
    }

    #[test]
    fn test_pre_validate_missing_rfc2119_keyword() {
        let items = vec![create_item_with_spec(
            "SYSREQ-001",
            builtin::SYSTEM_REQUIREMENT,
            "The system responds within 100ms",
        )];

        let rule = MetadataRule;
        let errors = rule.pre_validate(&items, &ValidationConfig::default());
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            &errors[0],
            SaraError::InvalidMetadata { reason, .. } if reason.contains("RFC2119")
        ));
    }

    #[test]
    fn test_pre_validate_non_requirement_type() {
        // Solution type doesn't require specification
        let source = SourceLocation::new(PathBuf::from("/repo"), "SOL-001.md");
        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("SOL-001"))
            .item_type(builtin::SOLUTION)
            .name("Test Solution")
            .source(source)
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
                builtin::SYSTEM_REQUIREMENT,
                "The system SHALL respond within 100ms",
            ),
            create_item_with_spec(
                "SYSREQ-002",
                builtin::SYSTEM_REQUIREMENT,
                "Missing keyword here", // Invalid
            ),
            create_item_with_spec(
                "SYSREQ-003",
                builtin::SYSTEM_REQUIREMENT,
                "The system MUST be secure",
            ),
        ];

        let rule = MetadataRule;
        let errors = rule.pre_validate(&items, &ValidationConfig::default());
        assert_eq!(errors.len(), 1, "Should detect one invalid item");
        assert!(matches!(
            &errors[0],
            SaraError::InvalidMetadata { reason, .. } if reason.contains("RFC2119")
        ));
    }
}

//! Main validation orchestrator.

use std::collections::HashMap;

use crate::config::ValidationConfig;
use crate::graph::KnowledgeGraph;
use crate::model::Item;
use crate::validation::report::{ValidationReport, ValidationReportBuilder};
use crate::validation::rule::{Severity, ValidationRule};
use crate::validation::rules::{
    BrokenReferencesRule, CyclesRule, DuplicatesRule, MetadataRule, OrphansRule,
    RedundantRelationshipsRule, RelationshipsRule,
};

/// All validation rules.
static RULES: &[&dyn ValidationRule] = &[
    &BrokenReferencesRule,
    &DuplicatesRule,
    &CyclesRule,
    &RelationshipsRule,
    &MetadataRule,
    &RedundantRelationshipsRule,
    &OrphansRule,
];

/// Orchestrates all validation rules.
pub struct Validator {
    /// Configuration for validation behavior.
    config: ValidationConfig,
}

impl Validator {
    /// Creates a new validator with the given configuration.
    pub fn new(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Creates a validator with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(ValidationConfig::default())
    }

    /// Pre-validates a list of items before adding them to the graph.
    ///
    /// This enables fail-fast validation during parsing/loading. Only rules
    /// that can validate items independently (without graph context) will
    /// produce errors here.
    pub fn pre_validate(&self, items: &[Item]) -> ValidationReport {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        for rule in RULES {
            let issues = rule.pre_validate(items, &self.config);
            let severity = if self.config.strict_mode {
                Severity::Error
            } else {
                rule.severity()
            };
            match severity {
                Severity::Error => errors.extend(issues),
                Severity::Warning => warnings.extend(issues),
            }
        }

        let mut items_by_type = HashMap::new();
        for item in items {
            *items_by_type.entry(item.item_type).or_insert(0) += 1;
        }

        ValidationReportBuilder::new()
            .items_checked(items.len())
            .items_by_type(items_by_type)
            .errors(errors)
            .warnings(warnings)
            .build()
    }

    /// Validates the knowledge graph and returns a report.
    pub fn validate(&self, graph: &KnowledgeGraph) -> ValidationReport {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Run all rules and categorize by severity
        // In strict mode, all issues become errors
        for rule in RULES {
            let issues = rule.validate(graph, &self.config);
            let severity = if self.config.strict_mode {
                Severity::Error
            } else {
                rule.severity()
            };
            match severity {
                Severity::Error => errors.extend(issues),
                Severity::Warning => warnings.extend(issues),
            }
        }

        ValidationReportBuilder::new()
            .items_checked(graph.item_count())
            .relationships_checked(graph.relationship_count())
            .items_by_type(graph.count_by_type())
            .errors(errors)
            .warnings(warnings)
            .build()
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Convenience function to validate a graph.
///
/// When `strict` is true, all issues (including orphans) are treated as errors.
pub fn validate(graph: &KnowledgeGraph, strict: bool) -> ValidationReport {
    let config = ValidationConfig {
        strict_mode: strict,
        ..Default::default()
    };
    Validator::new(config).validate(graph)
}

/// Convenience function to pre-validate items before adding them to the graph.
///
/// When `strict` is true, all issues are treated as errors.
/// If the report contains errors, the items should not be added to the graph.
pub fn pre_validate(items: &[Item], strict: bool) -> ValidationReport {
    let config = ValidationConfig {
        strict_mode: strict,
        ..Default::default()
    };
    Validator::new(config).pre_validate(items)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    use crate::error::SaraError;
    use crate::graph::KnowledgeGraphBuilder;
    use crate::model::{FieldValue, ItemBuilder, ItemId, Relationship, SourceLocation};
    use crate::schema::builtin;
    use crate::test_utils::{create_test_item, create_test_item_with_relationships};

    #[test]
    fn test_valid_graph() {
        let graph = KnowledgeGraphBuilder::new()
            .add_item(create_test_item("SOL-001", builtin::SOLUTION))
            .add_item(create_test_item_with_relationships(
                "UC-001",
                builtin::USE_CASE,
                vec![Relationship::new(
                    ItemId::new_unchecked("SOL-001"),
                    builtin::REFINES,
                )],
            ))
            .build()
            .unwrap();

        let report = validate(&graph, false);
        assert!(report.is_valid(), "Valid graph should pass validation");
        assert_eq!(report.error_count(), 0);
    }

    #[test]
    fn test_broken_reference() {
        let graph = KnowledgeGraphBuilder::new()
            .add_item(create_test_item_with_relationships(
                "UC-001",
                builtin::USE_CASE,
                vec![Relationship::new(
                    ItemId::new_unchecked("SOL-MISSING"),
                    builtin::REFINES,
                )],
            ))
            .build()
            .unwrap();

        let report = validate(&graph, false);
        assert!(!report.is_valid());
        assert!(report.error_count() > 0);
    }

    #[test]
    fn test_orphan_warning() {
        let graph = KnowledgeGraphBuilder::new()
            .add_item(create_test_item("UC-001", builtin::USE_CASE))
            .build()
            .unwrap();

        // Non-strict mode: orphan is a warning
        let report = validate(&graph, false);
        assert!(
            report.is_valid(),
            "Orphan should be warning in non-strict mode"
        );
        assert_eq!(report.warning_count(), 1);
    }

    #[test]
    fn test_orphan_error_strict() {
        let graph = KnowledgeGraphBuilder::new()
            .add_item(create_test_item("UC-001", builtin::USE_CASE))
            .build()
            .unwrap();

        // Strict mode: orphan is an error
        let report = validate(&graph, true);
        assert!(!report.is_valid(), "Orphan should be error in strict mode");
        assert_eq!(report.error_count(), 1);
    }

    #[test]
    fn test_cycle_detection() {
        // Create a cycle
        let scen1 = create_test_item_with_relationships(
            "SCEN-001",
            builtin::SCENARIO,
            vec![Relationship::new(
                ItemId::new_unchecked("SCEN-002"),
                builtin::REFINES,
            )],
        );
        let scen2 = create_test_item_with_relationships(
            "SCEN-002",
            builtin::SCENARIO,
            vec![Relationship::new(
                ItemId::new_unchecked("SCEN-001"),
                builtin::REFINES,
            )],
        );

        let graph = KnowledgeGraphBuilder::new()
            .add_item(scen1)
            .add_item(scen2)
            .build()
            .unwrap();

        let report = validate(&graph, false);
        assert!(!report.is_valid(), "Cycle should be detected");
    }

    #[test]
    fn test_invalid_relationship() {
        // Scenario trying to refine Solution directly (invalid)
        let graph = KnowledgeGraphBuilder::new()
            .add_item(create_test_item("SOL-001", builtin::SOLUTION))
            .add_item(create_test_item_with_relationships(
                "SCEN-001",
                builtin::SCENARIO,
                vec![Relationship::new(
                    ItemId::new_unchecked("SOL-001"),
                    builtin::REFINES,
                )],
            ))
            .build()
            .unwrap();

        let report = validate(&graph, false);
        assert!(
            !report.is_valid(),
            "Invalid relationship should be detected"
        );
    }

    #[test]
    fn test_pre_validate_valid_items() {
        let source = SourceLocation::new(PathBuf::from("/repo"), "SYSREQ-001.md");
        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("SYSREQ-001"))
            .item_type(builtin::SYSTEM_REQUIREMENT)
            .name("Test Requirement")
            .source(source)
            .attribute(
                "specification",
                FieldValue::text("The system SHALL respond within 100ms"),
            )
            .build()
            .unwrap();

        let report = pre_validate(&[item], false);
        assert!(
            report.is_valid(),
            "Valid item should have no pre-validation errors"
        );
        assert_eq!(
            report.warning_count(),
            0,
            "Valid item should have no pre-validation warnings"
        );
    }

    #[test]
    fn test_pre_validate_invalid_specification() {
        let source = SourceLocation::new(PathBuf::from("/repo"), "SYSREQ-001.md");
        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("SYSREQ-001"))
            .item_type(builtin::SYSTEM_REQUIREMENT)
            .name("Test Requirement")
            .source(source)
            .attribute(
                "specification",
                FieldValue::text("The system responds within 100ms"),
            ) // Missing RFC2119 keyword
            .build()
            .unwrap();

        let report = pre_validate(&[item], false);
        assert_eq!(
            report.error_count(),
            1,
            "Should detect missing RFC2119 keyword"
        );
        let errors = report.errors();
        assert!(matches!(
            errors[0],
            SaraError::InvalidMetadata { reason, .. } if reason.contains("RFC2119")
        ));
    }

    #[test]
    fn test_pre_validate_empty_specification() {
        let source = SourceLocation::new(PathBuf::from("/repo"), "SYSREQ-001.md");
        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("SYSREQ-001"))
            .item_type(builtin::SYSTEM_REQUIREMENT)
            .name("Test Requirement")
            .source(source)
            .attribute("specification", FieldValue::text(String::new()))
            .build()
            .unwrap();

        let report = pre_validate(&[item], false);
        assert_eq!(report.error_count(), 1, "Should detect empty specification");
        let errors = report.errors();
        assert!(matches!(
            errors[0],
            SaraError::InvalidMetadata { reason, .. } if reason.contains("non-empty")
        ));
    }

    #[test]
    fn test_pre_validate_solution_no_errors() {
        // Solution type doesn't require specification - should pass pre-validation
        let source = SourceLocation::new(PathBuf::from("/repo"), "SOL-001.md");
        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("SOL-001"))
            .item_type(builtin::SOLUTION)
            .name("Test Solution")
            .source(source)
            .build()
            .unwrap();

        let report = pre_validate(&[item], false);
        assert!(report.is_valid(), "Solution should pass pre-validation");
        assert_eq!(report.warning_count(), 0);
    }

    #[test]
    fn test_pre_validate_multiple_items() {
        let items = vec![
            ItemBuilder::new()
                .id(ItemId::new_unchecked("SYSREQ-001"))
                .item_type(builtin::SYSTEM_REQUIREMENT)
                .name("Valid Requirement")
                .source(SourceLocation::new(PathBuf::from("/repo"), "SYSREQ-001.md"))
                .attribute(
                    "specification",
                    FieldValue::text("The system SHALL respond"),
                )
                .build()
                .unwrap(),
            ItemBuilder::new()
                .id(ItemId::new_unchecked("SYSREQ-002"))
                .item_type(builtin::SYSTEM_REQUIREMENT)
                .name("Invalid Requirement")
                .source(SourceLocation::new(PathBuf::from("/repo"), "SYSREQ-002.md"))
                .attribute("specification", FieldValue::text("Missing keyword")) // Invalid
                .build()
                .unwrap(),
            ItemBuilder::new()
                .id(ItemId::new_unchecked("SOL-001"))
                .item_type(builtin::SOLUTION)
                .name("Solution")
                .source(SourceLocation::new(PathBuf::from("/repo"), "SOL-001.md"))
                .build()
                .unwrap(),
        ];

        let report = pre_validate(&items, false);
        assert_eq!(report.error_count(), 1, "Should detect one invalid item");
        assert_eq!(report.warning_count(), 0);
    }
}

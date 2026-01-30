//! Main validation orchestrator.

use std::time::Instant;

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
        let start = Instant::now();
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

        ValidationReportBuilder::new()
            .items_checked(items.len())
            .duration(start.elapsed())
            .errors(errors)
            .warnings(warnings)
            .build()
    }

    /// Validates the knowledge graph and returns a report.
    pub fn validate(&self, graph: &KnowledgeGraph) -> ValidationReport {
        let start = Instant::now();

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
            .relationships_checked(graph.relationship_count())
            .duration(start.elapsed())
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
    use super::*;
    use crate::error::ValidationError;
    use crate::graph::GraphBuilder;
    use crate::model::{
        ItemAttributes, ItemBuilder, ItemId, ItemType, RelationshipType, SourceLocation,
        UpstreamRefs,
    };
    use crate::test_utils::{create_test_item, create_test_item_with_upstream};
    use std::path::PathBuf;

    #[test]
    fn test_valid_graph() {
        let graph = GraphBuilder::new()
            .add_item(create_test_item("SOL-001", ItemType::Solution))
            .add_item(create_test_item_with_upstream(
                "UC-001",
                ItemType::UseCase,
                UpstreamRefs {
                    refines: vec![ItemId::new_unchecked("SOL-001")],
                    ..Default::default()
                },
            ))
            .build()
            .unwrap();

        let report = validate(&graph, false);
        assert!(report.is_valid(), "Valid graph should pass validation");
        assert_eq!(report.error_count(), 0);
    }

    #[test]
    fn test_broken_reference() {
        let graph = GraphBuilder::new()
            .add_item(create_test_item_with_upstream(
                "UC-001",
                ItemType::UseCase,
                UpstreamRefs {
                    refines: vec![ItemId::new_unchecked("SOL-MISSING")],
                    ..Default::default()
                },
            ))
            .build()
            .unwrap();

        let report = validate(&graph, false);
        assert!(!report.is_valid());
        assert!(report.error_count() > 0);
    }

    #[test]
    fn test_orphan_warning() {
        let graph = GraphBuilder::new()
            .add_item(create_test_item("UC-001", ItemType::UseCase))
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
        let graph = GraphBuilder::new()
            .add_item(create_test_item("UC-001", ItemType::UseCase))
            .build()
            .unwrap();

        // Strict mode: orphan is an error
        let report = validate(&graph, true);
        assert!(!report.is_valid(), "Orphan should be error in strict mode");
        assert_eq!(report.error_count(), 1);
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = KnowledgeGraph::new(false);

        // Create a cycle
        let scen1 = create_test_item_with_upstream(
            "SCEN-001",
            ItemType::Scenario,
            UpstreamRefs {
                refines: vec![ItemId::new_unchecked("SCEN-002")],
                ..Default::default()
            },
        );
        let scen2 = create_test_item_with_upstream(
            "SCEN-002",
            ItemType::Scenario,
            UpstreamRefs {
                refines: vec![ItemId::new_unchecked("SCEN-001")],
                ..Default::default()
            },
        );

        graph.add_item(scen1);
        graph.add_item(scen2);

        graph.add_relationship(
            &ItemId::new_unchecked("SCEN-001"),
            &ItemId::new_unchecked("SCEN-002"),
            RelationshipType::Refines,
        );
        graph.add_relationship(
            &ItemId::new_unchecked("SCEN-002"),
            &ItemId::new_unchecked("SCEN-001"),
            RelationshipType::Refines,
        );

        let report = validate(&graph, false);
        assert!(!report.is_valid(), "Cycle should be detected");
    }

    #[test]
    fn test_invalid_relationship() {
        let mut graph = KnowledgeGraph::new(false);

        // Scenario trying to refine Solution directly (invalid)
        graph.add_item(create_test_item("SOL-001", ItemType::Solution));
        graph.add_item(create_test_item_with_upstream(
            "SCEN-001",
            ItemType::Scenario,
            UpstreamRefs {
                refines: vec![ItemId::new_unchecked("SOL-001")],
                ..Default::default()
            },
        ));

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
            .item_type(ItemType::SystemRequirement)
            .name("Test Requirement")
            .source(source)
            .attributes(ItemAttributes::SystemRequirement {
                specification: "The system SHALL respond within 100ms".to_string(),
                depends_on: Vec::new(),
            })
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
            .item_type(ItemType::SystemRequirement)
            .name("Test Requirement")
            .source(source)
            .attributes(ItemAttributes::SystemRequirement {
                specification: "The system responds within 100ms".to_string(), // Missing RFC2119 keyword
                depends_on: Vec::new(),
            })
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
            ValidationError::InvalidMetadata { reason, .. } if reason.contains("RFC2119")
        ));
    }

    #[test]
    fn test_pre_validate_empty_specification() {
        let source = SourceLocation::new(PathBuf::from("/repo"), "SYSREQ-001.md");
        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("SYSREQ-001"))
            .item_type(ItemType::SystemRequirement)
            .name("Test Requirement")
            .source(source)
            .attributes(ItemAttributes::SystemRequirement {
                specification: String::new(),
                depends_on: Vec::new(),
            })
            .build()
            .unwrap();

        let report = pre_validate(&[item], false);
        assert_eq!(report.error_count(), 1, "Should detect empty specification");
        let errors = report.errors();
        assert!(matches!(
            errors[0],
            ValidationError::InvalidMetadata { reason, .. } if reason.contains("non-empty")
        ));
    }

    #[test]
    fn test_pre_validate_solution_no_errors() {
        // Solution type doesn't require specification - should pass pre-validation
        let source = SourceLocation::new(PathBuf::from("/repo"), "SOL-001.md");
        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("SOL-001"))
            .item_type(ItemType::Solution)
            .name("Test Solution")
            .source(source)
            .attributes(ItemAttributes::for_type(ItemType::Solution))
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
                .item_type(ItemType::SystemRequirement)
                .name("Valid Requirement")
                .source(SourceLocation::new(PathBuf::from("/repo"), "SYSREQ-001.md"))
                .attributes(ItemAttributes::SystemRequirement {
                    specification: "The system SHALL respond".to_string(),
                    depends_on: Vec::new(),
                })
                .build()
                .unwrap(),
            ItemBuilder::new()
                .id(ItemId::new_unchecked("SYSREQ-002"))
                .item_type(ItemType::SystemRequirement)
                .name("Invalid Requirement")
                .source(SourceLocation::new(PathBuf::from("/repo"), "SYSREQ-002.md"))
                .attributes(ItemAttributes::SystemRequirement {
                    specification: "Missing keyword".to_string(), // Invalid
                    depends_on: Vec::new(),
                })
                .build()
                .unwrap(),
            ItemBuilder::new()
                .id(ItemId::new_unchecked("SOL-001"))
                .item_type(ItemType::Solution)
                .name("Solution")
                .source(SourceLocation::new(PathBuf::from("/repo"), "SOL-001.md"))
                .attributes(ItemAttributes::for_type(ItemType::Solution))
                .build()
                .unwrap(),
        ];

        let report = pre_validate(&items, false);
        assert_eq!(report.error_count(), 1, "Should detect one invalid item");
        assert_eq!(report.warning_count(), 0);
    }
}

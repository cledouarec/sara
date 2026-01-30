//! Main validation orchestrator.

use std::time::Instant;

use crate::config::ValidationConfig;
use crate::graph::KnowledgeGraph;
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

    /// Validates the knowledge graph and returns a report.
    pub fn validate(&self, graph: &KnowledgeGraph) -> ValidationReport {
        let start = Instant::now();

        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Run all rules and categorize by severity
        // In strict mode, all issues become errors
        for rule in RULES {
            let issues = rule.validate(graph, &self.config);
            let severity = if self.config.strict_orphans {
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

/// Convenience function to validate a graph with default settings.
pub fn validate(graph: &KnowledgeGraph) -> ValidationReport {
    Validator::with_defaults().validate(graph)
}

/// Convenience function to validate a graph with strict orphan checking.
pub fn validate_strict(graph: &KnowledgeGraph) -> ValidationReport {
    let config = ValidationConfig {
        strict_orphans: true,
        ..Default::default()
    };
    Validator::new(config).validate(graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::GraphBuilder;
    use crate::model::{ItemId, ItemType, RelationshipType};
    use crate::test_utils::{create_test_item, create_test_item_with_relationships};

    #[test]
    fn test_valid_graph() {
        let graph = GraphBuilder::new()
            .add_item(create_test_item("SOL-001", ItemType::Solution))
            .add_item(create_test_item_with_relationships(
                "UC-001",
                ItemType::UseCase,
                vec![(ItemId::new_unchecked("SOL-001"), RelationshipType::Refines)],
            ))
            .build()
            .unwrap();

        let report = validate(&graph);
        assert!(report.is_valid(), "Valid graph should pass validation");
        assert_eq!(report.error_count(), 0);
    }

    #[test]
    fn test_broken_reference() {
        let graph = GraphBuilder::new()
            .add_item(create_test_item_with_relationships(
                "UC-001",
                ItemType::UseCase,
                vec![(
                    ItemId::new_unchecked("SOL-MISSING"),
                    RelationshipType::Refines,
                )],
            ))
            .build()
            .unwrap();

        let report = validate(&graph);
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
        let report = validate(&graph);
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
        let report = validate_strict(&graph);
        assert!(!report.is_valid(), "Orphan should be error in strict mode");
        assert_eq!(report.error_count(), 1);
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = KnowledgeGraph::new(false);

        // Create a cycle
        let scen1 = create_test_item_with_relationships(
            "SCEN-001",
            ItemType::Scenario,
            vec![(
                ItemId::new_unchecked("SCEN-002"),
                RelationshipType::Refines,
            )],
        );
        let scen2 = create_test_item_with_relationships(
            "SCEN-002",
            ItemType::Scenario,
            vec![(
                ItemId::new_unchecked("SCEN-001"),
                RelationshipType::Refines,
            )],
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

        let report = validate(&graph);
        assert!(!report.is_valid(), "Cycle should be detected");
    }

    #[test]
    fn test_invalid_relationship() {
        let mut graph = KnowledgeGraph::new(false);

        // Scenario trying to refine Solution directly (invalid)
        graph.add_item(create_test_item("SOL-001", ItemType::Solution));
        graph.add_item(create_test_item_with_relationships(
            "SCEN-001",
            ItemType::Scenario,
            vec![(ItemId::new_unchecked("SOL-001"), RelationshipType::Refines)],
        ));

        let report = validate(&graph);
        assert!(
            !report.is_valid(),
            "Invalid relationship should be detected"
        );
    }
}

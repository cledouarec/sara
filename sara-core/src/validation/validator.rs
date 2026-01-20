//! Main validation orchestrator.

use std::time::Instant;

use crate::config::ValidationConfig;
use crate::graph::KnowledgeGraph;
use crate::validation::report::{ValidationReport, ValidationReportBuilder};
use crate::validation::rules;

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
        let mut builder = ValidationReportBuilder::new()
            .items_checked(graph.item_count())
            .relationships_checked(graph.relationship_count());

        // Run all validation rules

        // 1. Check for broken references (FR-010)
        let broken_refs = rules::check_broken_references(graph);
        builder = builder.errors(broken_refs);

        // 2. Check for orphan items (FR-011)
        // Orphans are errors in strict mode, warnings otherwise
        let orphans = rules::check_orphans(graph, self.config.strict_orphans);
        if self.config.strict_orphans {
            builder = builder.errors(orphans);
        } else {
            builder = builder.warnings(orphans);
        }

        // 3. Check for duplicate identifiers (FR-012)
        // Note: Duplicates are typically caught during parsing/graph construction
        let duplicates = rules::check_duplicates(graph);
        builder = builder.errors(duplicates);

        // 4. Check for circular references (FR-013)
        let cycles = rules::check_cycles(graph);
        builder = builder.errors(cycles);

        // 5. Check metadata validity (FR-014)
        let metadata_errors = rules::check_metadata(graph, &self.config.allowed_custom_fields);
        builder = builder.errors(metadata_errors);

        // 6. Check relationship validity (FR-006, FR-007, FR-008)
        let relationship_errors = rules::check_relationships(graph);
        builder = builder.errors(relationship_errors);

        // 7. Check for redundant relationships (warning only)
        let redundant = rules::check_redundant_relationships(graph);
        builder = builder.warnings(redundant);

        builder.duration(start.elapsed()).build()
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
    use crate::model::{
        ItemBuilder, ItemId, ItemType, RelationshipType, SourceLocation, UpstreamRefs,
    };
    use std::path::PathBuf;

    fn create_item(
        id: &str,
        item_type: ItemType,
        upstream: Option<UpstreamRefs>,
    ) -> crate::model::Item {
        let source = SourceLocation::new(PathBuf::from("/repo"), format!("{}.md", id));
        let mut builder = ItemBuilder::new()
            .id(ItemId::new_unchecked(id))
            .item_type(item_type)
            .name(format!("Test {}", id))
            .source(source);

        if let Some(up) = upstream {
            builder = builder.upstream(up);
        }

        if item_type.requires_specification() {
            builder = builder.specification("Test spec");
        }

        builder.build().unwrap()
    }

    #[test]
    fn test_valid_graph() {
        let graph = GraphBuilder::new()
            .add_item(create_item("SOL-001", ItemType::Solution, None))
            .add_item(create_item(
                "UC-001",
                ItemType::UseCase,
                Some(UpstreamRefs {
                    refines: vec![ItemId::new_unchecked("SOL-001")],
                    ..Default::default()
                }),
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
            .add_item(create_item(
                "UC-001",
                ItemType::UseCase,
                Some(UpstreamRefs {
                    refines: vec![ItemId::new_unchecked("SOL-MISSING")],
                    ..Default::default()
                }),
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
            .add_item(create_item("UC-001", ItemType::UseCase, None))
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
            .add_item(create_item("UC-001", ItemType::UseCase, None))
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
        let scen1 = create_item(
            "SCEN-001",
            ItemType::Scenario,
            Some(UpstreamRefs {
                refines: vec![ItemId::new_unchecked("SCEN-002")],
                ..Default::default()
            }),
        );
        let scen2 = create_item(
            "SCEN-002",
            ItemType::Scenario,
            Some(UpstreamRefs {
                refines: vec![ItemId::new_unchecked("SCEN-001")],
                ..Default::default()
            }),
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
        graph.add_item(create_item("SOL-001", ItemType::Solution, None));
        graph.add_item(create_item(
            "SCEN-001",
            ItemType::Scenario,
            Some(UpstreamRefs {
                refines: vec![ItemId::new_unchecked("SOL-001")],
                ..Default::default()
            }),
        ));

        let report = validate(&graph);
        assert!(
            !report.is_valid(),
            "Invalid relationship should be detected"
        );
    }
}

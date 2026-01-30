//! Relationship type validation rule.

use crate::config::ValidationConfig;
use crate::error::ValidationError;
use crate::graph::KnowledgeGraph;
use crate::model::{Item, ItemId, RelationshipRules, RelationshipType};
use crate::validation::rule::ValidationRule;

/// Relationship type validation rule.
///
/// Validates that all relationships conform to the allowed type rules.
/// For example:
/// - UseCase can only refine Solution
/// - Scenario can only refine UseCase
/// - SystemRequirement can only derive_from Scenario
pub struct RelationshipsRule;

impl ValidationRule for RelationshipsRule {
    fn validate(&self, graph: &KnowledgeGraph, _config: &ValidationConfig) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for item in graph.items() {
            errors.extend(validate_item_relationships(graph, item));
        }

        errors
    }
}

/// Checks references of a specific relationship type and collects validation errors.
fn check_references<'a>(
    item: &Item,
    graph: &KnowledgeGraph,
    refs: impl Iterator<Item = &'a ItemId>,
    rel_type: RelationshipType,
    errors: &mut Vec<ValidationError>,
) {
    for ref_id in refs {
        if let Some(target) = graph.get(ref_id)
            && !RelationshipRules::is_valid_relationship(item.item_type, target.item_type, rel_type)
        {
            errors.push(ValidationError::InvalidRelationship {
                from_id: item.id.clone(),
                to_id: ref_id.clone(),
                from_type: item.item_type,
                to_type: target.item_type,
                rel_type,
            });
        }
    }
}

/// Validates relationships for a single item.
fn validate_item_relationships(graph: &KnowledgeGraph, item: &Item) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // Check upstream references
    check_references(
        item,
        graph,
        item.upstream.refines.iter(),
        RelationshipType::Refines,
        &mut errors,
    );
    check_references(
        item,
        graph,
        item.upstream.derives_from.iter(),
        RelationshipType::DerivesFrom,
        &mut errors,
    );
    check_references(
        item,
        graph,
        item.upstream.satisfies.iter(),
        RelationshipType::Satisfies,
        &mut errors,
    );

    // Check downstream references
    check_references(
        item,
        graph,
        item.downstream.is_refined_by.iter(),
        RelationshipType::IsRefinedBy,
        &mut errors,
    );
    check_references(
        item,
        graph,
        item.downstream.derives.iter(),
        RelationshipType::Derives,
        &mut errors,
    );
    check_references(
        item,
        graph,
        item.downstream.is_satisfied_by.iter(),
        RelationshipType::IsSatisfiedBy,
        &mut errors,
    );

    // Check peer dependencies
    check_references(
        item,
        graph,
        item.attributes.depends_on().iter(),
        RelationshipType::DependsOn,
        &mut errors,
    );

    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::KnowledgeGraphBuilder;
    use crate::model::{DownstreamRefs, ItemId, ItemType, UpstreamRefs};
    use crate::test_utils::{
        create_test_item, create_test_item_with_refs, create_test_item_with_upstream,
    };

    #[test]
    fn test_valid_relationship() {
        let graph = KnowledgeGraphBuilder::new()
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

        let rule = RelationshipsRule;
        let errors = rule.validate(&graph, &ValidationConfig::default());
        assert!(
            errors.is_empty(),
            "Valid relationship should not produce errors"
        );
    }

    #[test]
    fn test_invalid_relationship() {
        let graph = KnowledgeGraphBuilder::new()
            .add_item(create_test_item("SOL-001", ItemType::Solution))
            // Scenario trying to refine Solution directly (should be UseCase)
            .add_item(create_test_item_with_upstream(
                "SCEN-001",
                ItemType::Scenario,
                UpstreamRefs {
                    refines: vec![ItemId::new_unchecked("SOL-001")],
                    ..Default::default()
                },
            ))
            .build()
            .unwrap();

        let rule = RelationshipsRule;
        let errors = rule.validate(&graph, &ValidationConfig::default());
        assert_eq!(errors.len(), 1, "Invalid relationship should produce error");

        if let ValidationError::InvalidRelationship {
            from_type,
            to_type,
            rel_type,
            ..
        } = &errors[0]
        {
            assert_eq!(*from_type, ItemType::Scenario);
            assert_eq!(*to_type, ItemType::Solution);
            assert_eq!(*rel_type, RelationshipType::Refines);
        } else {
            panic!("Expected InvalidRelationship error");
        }
    }

    #[test]
    fn test_valid_downstream_relationship() {
        let graph = KnowledgeGraphBuilder::new()
            .add_item(create_test_item_with_refs(
                "SOL-001",
                ItemType::Solution,
                UpstreamRefs::default(),
                DownstreamRefs {
                    is_refined_by: vec![ItemId::new_unchecked("UC-001")],
                    ..Default::default()
                },
            ))
            .add_item(create_test_item("UC-001", ItemType::UseCase))
            .build()
            .unwrap();

        let rule = RelationshipsRule;
        let errors = rule.validate(&graph, &ValidationConfig::default());
        assert!(errors.is_empty());
    }
}

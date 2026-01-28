//! Relationship type validation rule.

use crate::error::ValidationError;
use crate::graph::KnowledgeGraph;
use crate::model::{Item, RelationshipRules, RelationshipType};

/// Validates that all relationships conform to the allowed type rules.
///
/// For example:
/// - UseCase can only refine Solution
/// - Scenario can only refine UseCase
/// - SystemRequirement can only derive_from Scenario
pub fn check_relationships(graph: &KnowledgeGraph) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    for item in graph.items() {
        errors.extend(validate_item_relationships(graph, item));
    }

    errors
}

/// Validates relationships for a single item.
fn validate_item_relationships(graph: &KnowledgeGraph, item: &Item) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // Check upstream references (refines)
    for ref_id in &item.upstream.refines {
        if let Some(target) = graph.get(ref_id)
            && !RelationshipRules::is_valid_relationship(
                item.item_type,
                target.item_type,
                RelationshipType::Refines,
            )
        {
            errors.push(ValidationError::InvalidRelationship {
                from_id: item.id.clone(),
                to_id: ref_id.clone(),
                from_type: item.item_type,
                to_type: target.item_type,
                rel_type: RelationshipType::Refines,
                location: Some(item.source.clone()),
            });
        }
    }

    // Check upstream references (derives_from)
    for ref_id in &item.upstream.derives_from {
        if let Some(target) = graph.get(ref_id)
            && !RelationshipRules::is_valid_relationship(
                item.item_type,
                target.item_type,
                RelationshipType::DerivesFrom,
            )
        {
            errors.push(ValidationError::InvalidRelationship {
                from_id: item.id.clone(),
                to_id: ref_id.clone(),
                from_type: item.item_type,
                to_type: target.item_type,
                rel_type: RelationshipType::DerivesFrom,
                location: Some(item.source.clone()),
            });
        }
    }

    // Check upstream references (satisfies)
    for ref_id in &item.upstream.satisfies {
        if let Some(target) = graph.get(ref_id)
            && !RelationshipRules::is_valid_relationship(
                item.item_type,
                target.item_type,
                RelationshipType::Satisfies,
            )
        {
            errors.push(ValidationError::InvalidRelationship {
                from_id: item.id.clone(),
                to_id: ref_id.clone(),
                from_type: item.item_type,
                to_type: target.item_type,
                rel_type: RelationshipType::Satisfies,
                location: Some(item.source.clone()),
            });
        }
    }

    // Check downstream references (is_refined_by)
    for ref_id in &item.downstream.is_refined_by {
        if let Some(target) = graph.get(ref_id)
            && !RelationshipRules::is_valid_relationship(
                item.item_type,
                target.item_type,
                RelationshipType::IsRefinedBy,
            )
        {
            errors.push(ValidationError::InvalidRelationship {
                from_id: item.id.clone(),
                to_id: ref_id.clone(),
                from_type: item.item_type,
                to_type: target.item_type,
                rel_type: RelationshipType::IsRefinedBy,
                location: Some(item.source.clone()),
            });
        }
    }

    // Check downstream references (derives)
    for ref_id in &item.downstream.derives {
        if let Some(target) = graph.get(ref_id)
            && !RelationshipRules::is_valid_relationship(
                item.item_type,
                target.item_type,
                RelationshipType::Derives,
            )
        {
            errors.push(ValidationError::InvalidRelationship {
                from_id: item.id.clone(),
                to_id: ref_id.clone(),
                from_type: item.item_type,
                to_type: target.item_type,
                rel_type: RelationshipType::Derives,
                location: Some(item.source.clone()),
            });
        }
    }

    // Check downstream references (is_satisfied_by)
    for ref_id in &item.downstream.is_satisfied_by {
        if let Some(target) = graph.get(ref_id)
            && !RelationshipRules::is_valid_relationship(
                item.item_type,
                target.item_type,
                RelationshipType::IsSatisfiedBy,
            )
        {
            errors.push(ValidationError::InvalidRelationship {
                from_id: item.id.clone(),
                to_id: ref_id.clone(),
                from_type: item.item_type,
                to_type: target.item_type,
                rel_type: RelationshipType::IsSatisfiedBy,
                location: Some(item.source.clone()),
            });
        }
    }

    // Check peer dependencies (depends_on)
    for ref_id in item.attributes.depends_on() {
        if let Some(target) = graph.get(ref_id)
            && !RelationshipRules::is_valid_relationship(
                item.item_type,
                target.item_type,
                RelationshipType::DependsOn,
            )
        {
            errors.push(ValidationError::InvalidRelationship {
                from_id: item.id.clone(),
                to_id: ref_id.clone(),
                from_type: item.item_type,
                to_type: target.item_type,
                rel_type: RelationshipType::DependsOn,
                location: Some(item.source.clone()),
            });
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        DownstreamRefs, ItemBuilder, ItemId, ItemType, SourceLocation, UpstreamRefs,
    };
    use std::path::PathBuf;

    fn create_item(
        id: &str,
        item_type: ItemType,
        upstream: Option<UpstreamRefs>,
        downstream: Option<DownstreamRefs>,
    ) -> Item {
        let source = SourceLocation::new(PathBuf::from("/repo"), format!("{}.md", id));
        let mut builder = ItemBuilder::new()
            .id(ItemId::new_unchecked(id))
            .item_type(item_type)
            .name(format!("Test {}", id))
            .source(source);

        if let Some(up) = upstream {
            builder = builder.upstream(up);
        }
        if let Some(down) = downstream {
            builder = builder.downstream(down);
        }
        if item_type.requires_specification() {
            builder = builder.specification("Test spec");
        }

        builder.build().unwrap()
    }

    #[test]
    fn test_valid_relationship() {
        let mut graph = KnowledgeGraph::new(false);
        graph.add_item(create_item("SOL-001", ItemType::Solution, None, None));
        graph.add_item(create_item(
            "UC-001",
            ItemType::UseCase,
            Some(UpstreamRefs {
                refines: vec![ItemId::new_unchecked("SOL-001")],
                ..Default::default()
            }),
            None,
        ));

        let errors = check_relationships(&graph);
        assert!(
            errors.is_empty(),
            "Valid relationship should not produce errors"
        );
    }

    #[test]
    fn test_invalid_relationship() {
        let mut graph = KnowledgeGraph::new(false);
        graph.add_item(create_item("SOL-001", ItemType::Solution, None, None));
        // Scenario trying to refine Solution directly (should be UseCase)
        graph.add_item(create_item(
            "SCEN-001",
            ItemType::Scenario,
            Some(UpstreamRefs {
                refines: vec![ItemId::new_unchecked("SOL-001")],
                ..Default::default()
            }),
            None,
        ));

        let errors = check_relationships(&graph);
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
        let mut graph = KnowledgeGraph::new(false);
        graph.add_item(create_item(
            "SOL-001",
            ItemType::Solution,
            None,
            Some(DownstreamRefs {
                is_refined_by: vec![ItemId::new_unchecked("UC-001")],
                ..Default::default()
            }),
        ));
        graph.add_item(create_item("UC-001", ItemType::UseCase, None, None));

        let errors = check_relationships(&graph);
        assert!(errors.is_empty());
    }
}

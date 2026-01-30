//! Redundant relationship detection validation rule.

use std::collections::HashSet;

use crate::config::ValidationConfig;
use crate::error::ValidationError;
use crate::graph::KnowledgeGraph;
use crate::model::{Item, ItemId, RelationshipType};
use crate::validation::rule::{Severity, ValidationRule};

/// Redundant relationship detection rule (warning).
///
/// Detects redundant relationships where both items declare the same link.
/// For example, if SARCH-001 has `satisfies: [SYSREQ-00001]` and SYSREQ-00001
/// has `is_satisfied_by: [SARCH-001]`, this is redundant - only one declaration
/// is needed since the inverse is automatically inferred.
pub struct RedundantRelationshipsRule;

impl ValidationRule for RedundantRelationshipsRule {
    fn validate(&self, graph: &KnowledgeGraph, _config: &ValidationConfig) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        let mut seen_pairs: HashSet<(String, String)> = HashSet::new();

        for item in graph.items() {
            // Check downstream declarations against upstream declarations in target items

            // is_refined_by <-> refines
            check_redundant_pair(
                item,
                graph,
                &item.downstream.is_refined_by,
                |target| target.upstream.refines.contains(&item.id),
                &RelationshipPair {
                    from_rel: RelationshipType::IsRefinedBy,
                    to_rel: RelationshipType::Refines,
                },
                &mut seen_pairs,
                &mut errors,
            );

            // derives <-> derives_from
            check_redundant_pair(
                item,
                graph,
                &item.downstream.derives,
                |target| target.upstream.derives_from.contains(&item.id),
                &RelationshipPair {
                    from_rel: RelationshipType::Derives,
                    to_rel: RelationshipType::DerivesFrom,
                },
                &mut seen_pairs,
                &mut errors,
            );

            // is_satisfied_by <-> satisfies
            check_redundant_pair(
                item,
                graph,
                &item.downstream.is_satisfied_by,
                |target| target.upstream.satisfies.contains(&item.id),
                &RelationshipPair {
                    from_rel: RelationshipType::IsSatisfiedBy,
                    to_rel: RelationshipType::Satisfies,
                },
                &mut seen_pairs,
                &mut errors,
            );
        }

        errors
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }
}

/// Relationship pair configuration for redundancy checking.
struct RelationshipPair {
    from_rel: RelationshipType,
    to_rel: RelationshipType,
}

/// Checks for redundant declarations in a specific relationship pair.
fn check_redundant_pair<F>(
    item: &Item,
    graph: &KnowledgeGraph,
    downstream_refs: &[ItemId],
    has_inverse: F,
    pair: &RelationshipPair,
    seen_pairs: &mut HashSet<(String, String)>,
    errors: &mut Vec<ValidationError>,
) where
    F: Fn(&Item) -> bool,
{
    for target_id in downstream_refs {
        if let Some(target) = graph.get(target_id)
            && has_inverse(target)
        {
            let pair_key = make_pair_key(&item.id, target_id);
            if seen_pairs.insert(pair_key) {
                errors.push(ValidationError::RedundantRelationship {
                    from_id: item.id.clone(),
                    to_id: target_id.clone(),
                    from_rel: pair.from_rel,
                    to_rel: pair.to_rel,
                    from_location: Some(item.source.clone()),
                    to_location: Some(target.source.clone()),
                });
            }
        }
    }
}

/// Creates a canonical pair key for deduplication (smaller ID first).
fn make_pair_key(id1: &ItemId, id2: &ItemId) -> (String, String) {
    let s1 = id1.as_str();
    let s2 = id2.as_str();
    if s1 < s2 {
        (s1.to_string(), s2.to_string())
    } else {
        (s2.to_string(), s1.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::GraphBuilder;
    use crate::model::{DownstreamRefs, ItemType, UpstreamRefs};
    use crate::test_utils::{
        create_test_item, create_test_item_with_refs, create_test_item_with_upstream,
    };

    #[test]
    fn test_no_redundancy() {
        // SARCH satisfies SYSREQ, but SYSREQ doesn't declare is_satisfied_by
        let sysreq = create_test_item("SYSREQ-001", ItemType::SystemRequirement);
        let sarch = create_test_item_with_upstream(
            "SARCH-001",
            ItemType::SystemArchitecture,
            UpstreamRefs {
                satisfies: vec![ItemId::new_unchecked("SYSREQ-001")],
                ..Default::default()
            },
        );

        let graph = GraphBuilder::new()
            .add_item(sysreq)
            .add_item(sarch)
            .build()
            .unwrap();

        let rule = RedundantRelationshipsRule;
        let warnings = rule.validate(&graph, &ValidationConfig::default());
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_redundant_satisfies() {
        // Both declare the relationship - this is redundant
        let sysreq = create_test_item_with_refs(
            "SYSREQ-001",
            ItemType::SystemRequirement,
            UpstreamRefs::default(),
            DownstreamRefs {
                is_satisfied_by: vec![ItemId::new_unchecked("SARCH-001")],
                ..Default::default()
            },
        );
        let sarch = create_test_item_with_upstream(
            "SARCH-001",
            ItemType::SystemArchitecture,
            UpstreamRefs {
                satisfies: vec![ItemId::new_unchecked("SYSREQ-001")],
                ..Default::default()
            },
        );

        let graph = GraphBuilder::new()
            .add_item(sysreq)
            .add_item(sarch)
            .build()
            .unwrap();

        let rule = RedundantRelationshipsRule;
        let warnings = rule.validate(&graph, &ValidationConfig::default());
        assert_eq!(warnings.len(), 1);
        assert!(matches!(
            &warnings[0],
            ValidationError::RedundantRelationship { from_id, to_id, .. }
            if from_id.as_str() == "SYSREQ-001" && to_id.as_str() == "SARCH-001"
        ));
    }
}

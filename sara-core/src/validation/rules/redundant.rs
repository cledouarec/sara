//! Redundant relationship detection validation rule.

use std::collections::HashSet;

use crate::error::ValidationError;
use crate::graph::KnowledgeGraph;
use crate::model::{ItemId, RelationshipType};

/// Detects redundant relationships where both items declare the same link.
///
/// For example, if SARCH-001 has `satisfies: [SYSREQ-00001]` and SYSREQ-00001
/// has `is_satisfied_by: [SARCH-001]`, this is redundant - only one declaration
/// is needed since the inverse is automatically inferred.
pub fn check_redundant_relationships(graph: &KnowledgeGraph) -> Vec<ValidationError> {
    let mut warnings = Vec::new();
    let mut seen_pairs: HashSet<(String, String)> = HashSet::new();

    for item in graph.items() {
        // Check downstream declarations (is_refined_by, derives, is_satisfied_by)
        // against upstream declarations in target items (refines, derives_from, satisfies)

        // Check is_refined_by
        for target_id in &item.downstream.is_refined_by {
            if let Some(target) = graph.get(target_id)
                && target.upstream.refines.contains(&item.id)
            {
                let pair_key = make_pair_key(&item.id, target_id);
                if seen_pairs.insert(pair_key) {
                    warnings.push(ValidationError::RedundantRelationship {
                        from_id: item.id.clone(),
                        to_id: target_id.clone(),
                        from_rel: RelationshipType::IsRefinedBy,
                        to_rel: RelationshipType::Refines,
                        from_location: Some(item.source.clone()),
                        to_location: Some(target.source.clone()),
                    });
                }
            }
        }

        // Check derives
        for target_id in &item.downstream.derives {
            if let Some(target) = graph.get(target_id)
                && target.upstream.derives_from.contains(&item.id)
            {
                let pair_key = make_pair_key(&item.id, target_id);
                if seen_pairs.insert(pair_key) {
                    warnings.push(ValidationError::RedundantRelationship {
                        from_id: item.id.clone(),
                        to_id: target_id.clone(),
                        from_rel: RelationshipType::Derives,
                        to_rel: RelationshipType::DerivesFrom,
                        from_location: Some(item.source.clone()),
                        to_location: Some(target.source.clone()),
                    });
                }
            }
        }

        // Check is_satisfied_by
        for target_id in &item.downstream.is_satisfied_by {
            if let Some(target) = graph.get(target_id)
                && target.upstream.satisfies.contains(&item.id)
            {
                let pair_key = make_pair_key(&item.id, target_id);
                if seen_pairs.insert(pair_key) {
                    warnings.push(ValidationError::RedundantRelationship {
                        from_id: item.id.clone(),
                        to_id: target_id.clone(),
                        from_rel: RelationshipType::IsSatisfiedBy,
                        to_rel: RelationshipType::Satisfies,
                        from_location: Some(item.source.clone()),
                        to_location: Some(target.source.clone()),
                    });
                }
            }
        }
    }

    warnings
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
    use crate::model::{DownstreamRefs, ItemBuilder, ItemType, SourceLocation, UpstreamRefs};
    use std::path::PathBuf;

    fn create_item(
        id: &str,
        item_type: ItemType,
        upstream: Option<UpstreamRefs>,
        downstream: Option<DownstreamRefs>,
    ) -> crate::model::Item {
        let source = SourceLocation::new(PathBuf::from("/repo"), format!("{}.md", id), 1);
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
    fn test_no_redundancy() {
        // SARCH satisfies SYSREQ, but SYSREQ doesn't declare is_satisfied_by
        let sysreq = create_item("SYSREQ-001", ItemType::SystemRequirement, None, None);
        let sarch = create_item(
            "SARCH-001",
            ItemType::SystemArchitecture,
            Some(UpstreamRefs {
                satisfies: vec![ItemId::new_unchecked("SYSREQ-001")],
                ..Default::default()
            }),
            None,
        );

        let graph = GraphBuilder::new()
            .add_item(sysreq)
            .add_item(sarch)
            .build()
            .unwrap();

        let warnings = check_redundant_relationships(&graph);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_redundant_satisfies() {
        // Both declare the relationship - this is redundant
        let sysreq = create_item(
            "SYSREQ-001",
            ItemType::SystemRequirement,
            None,
            Some(DownstreamRefs {
                is_satisfied_by: vec![ItemId::new_unchecked("SARCH-001")],
                ..Default::default()
            }),
        );
        let sarch = create_item(
            "SARCH-001",
            ItemType::SystemArchitecture,
            Some(UpstreamRefs {
                satisfies: vec![ItemId::new_unchecked("SYSREQ-001")],
                ..Default::default()
            }),
            None,
        );

        let graph = GraphBuilder::new()
            .add_item(sysreq)
            .add_item(sarch)
            .build()
            .unwrap();

        let warnings = check_redundant_relationships(&graph);
        assert_eq!(warnings.len(), 1);
        assert!(matches!(
            &warnings[0],
            ValidationError::RedundantRelationship { from_id, to_id, .. }
            if from_id.as_str() == "SYSREQ-001" && to_id.as_str() == "SARCH-001"
        ));
    }
}

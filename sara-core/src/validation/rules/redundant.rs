//! Redundant relationship detection validation rule.

use std::collections::HashSet;

use crate::config::ValidationConfig;
use crate::error::ValidationError;
use crate::graph::KnowledgeGraph;
use crate::model::ItemId;
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
        let mut warnings = Vec::new();
        let mut seen_pairs: HashSet<(String, String)> = HashSet::new();

        for item in graph.items() {
            // Check each downstream declaration against the inverse upstream in target
            for (downstream_rel, target_id) in item.downstream_iter_with_types() {
                let inverse_rel = downstream_rel.inverse();

                if let Some(target) = graph.get(target_id)
                    && target.upstream_contains_for_type(inverse_rel, &item.id)
                {
                    let pair_key = make_pair_key(&item.id, target_id);
                    if seen_pairs.insert(pair_key) {
                        warnings.push(ValidationError::RedundantRelationship {
                            from_id: item.id.clone(),
                            to_id: target_id.clone(),
                            from_rel: downstream_rel,
                            to_rel: inverse_rel,
                        });
                    }
                }
            }
        }

        warnings
    }

    fn severity(&self) -> Severity {
        Severity::Warning
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
    use crate::model::{ItemType, RelationshipType};
    use crate::test_utils::{create_test_item, create_test_item_with_relationships};

    #[test]
    fn test_no_redundancy() {
        // SARCH satisfies SYSREQ, but SYSREQ doesn't declare is_satisfied_by
        let sysreq = create_test_item("SYSREQ-001", ItemType::SystemRequirement);
        let sarch = create_test_item_with_relationships(
            "SARCH-001",
            ItemType::SystemArchitecture,
            vec![(
                ItemId::new_unchecked("SYSREQ-001"),
                RelationshipType::Satisfies,
            )],
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
        // SYSREQ declares is_satisfied_by: SARCH
        // SARCH declares satisfies: SYSREQ
        let sysreq = create_test_item_with_relationships(
            "SYSREQ-001",
            ItemType::SystemRequirement,
            vec![(
                ItemId::new_unchecked("SARCH-001"),
                RelationshipType::IsSatisfiedBy,
            )],
        );
        let sarch = create_test_item_with_relationships(
            "SARCH-001",
            ItemType::SystemArchitecture,
            vec![(
                ItemId::new_unchecked("SYSREQ-001"),
                RelationshipType::Satisfies,
            )],
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

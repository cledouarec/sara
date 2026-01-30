//! Broken reference detection validation rule.

use crate::config::ValidationConfig;
use crate::error::ValidationError;
use crate::graph::KnowledgeGraph;
use crate::validation::rule::ValidationRule;

/// Broken reference detection rule.
///
/// Detects broken references in the knowledge graph. A broken reference
/// occurs when an item references another item that does not exist in the graph.
pub struct BrokenReferencesRule;

impl ValidationRule for BrokenReferencesRule {
    fn validate(&self, graph: &KnowledgeGraph, _config: &ValidationConfig) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for item in graph.items() {
            for ref_id in item.all_references() {
                if !graph.contains(ref_id) {
                    errors.push(ValidationError::BrokenReference {
                        from: item.id.clone(),
                        to: ref_id.clone(),
                        location: Some(item.source.clone()),
                    });
                }
            }
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::KnowledgeGraphBuilder;
    use crate::model::{ItemId, ItemType, UpstreamRefs};
    use crate::test_utils::{create_test_item, create_test_item_with_upstream};

    #[test]
    fn test_no_broken_refs() {
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

        let rule = BrokenReferencesRule;
        let errors = rule.validate(&graph, &ValidationConfig::default());
        assert!(errors.is_empty());
    }

    #[test]
    fn test_broken_ref_detected() {
        let graph = KnowledgeGraphBuilder::new()
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

        let rule = BrokenReferencesRule;
        let errors = rule.validate(&graph, &ValidationConfig::default());
        assert_eq!(errors.len(), 1);

        if let ValidationError::BrokenReference { from, to, .. } = &errors[0] {
            assert_eq!(from.as_str(), "UC-001");
            assert_eq!(to.as_str(), "SOL-MISSING");
        } else {
            panic!("Expected BrokenReference error");
        }
    }
}

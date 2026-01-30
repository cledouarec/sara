//! Orphan item detection validation rule.

use crate::config::ValidationConfig;
use crate::error::ValidationError;
use crate::graph::KnowledgeGraph;
use crate::validation::rule::{Severity, ValidationRule};

/// Orphan item detection rule.
///
/// Detects orphan items in the knowledge graph. An orphan is an item that
/// has no upstream parent, except for Solution items which are allowed to
/// be root items.
///
/// Default severity is Warning, but in strict mode all warnings become errors.
pub struct OrphansRule;

impl ValidationRule for OrphansRule {
    fn validate(&self, graph: &KnowledgeGraph, _config: &ValidationConfig) -> Vec<ValidationError> {
        graph
            .orphans()
            .into_iter()
            .map(|item| ValidationError::OrphanItem {
                id: item.id.clone(),
                item_type: item.item_type,
            })
            .collect()
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::KnowledgeGraphBuilder;
    use crate::model::{ItemId, ItemType, UpstreamRefs};
    use crate::test_utils::{create_test_item, create_test_item_with_upstream};

    #[test]
    fn test_solution_not_orphan() {
        let graph = KnowledgeGraphBuilder::new()
            .add_item(create_test_item("SOL-001", ItemType::Solution))
            .build()
            .unwrap();

        let rule = OrphansRule;
        let errors = rule.validate(&graph, &ValidationConfig::default());
        assert!(
            errors.is_empty(),
            "Solutions should not be reported as orphans"
        );
    }

    #[test]
    fn test_use_case_orphan_detected() {
        let graph = KnowledgeGraphBuilder::new()
            .add_item(create_test_item("UC-001", ItemType::UseCase))
            .build()
            .unwrap();

        let rule = OrphansRule;
        let errors = rule.validate(&graph, &ValidationConfig::default());
        assert_eq!(errors.len(), 1);

        if let ValidationError::OrphanItem { id, item_type, .. } = &errors[0] {
            assert_eq!(id.as_str(), "UC-001");
            assert_eq!(*item_type, ItemType::UseCase);
        } else {
            panic!("Expected OrphanItem error");
        }
    }

    #[test]
    fn test_linked_item_not_orphan() {
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

        let rule = OrphansRule;
        let errors = rule.validate(&graph, &ValidationConfig::default());
        assert!(errors.is_empty());
    }
}

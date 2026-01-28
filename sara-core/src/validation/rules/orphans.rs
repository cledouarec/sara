//! Orphan item detection validation rule.

use crate::error::ValidationError;
use crate::graph::KnowledgeGraph;

/// Detects orphan items in the knowledge graph.
///
/// An orphan is an item that has no upstream parent, except for Solution items
/// which are allowed to be root items.
///
/// # Arguments
/// * `graph` - The knowledge graph to check.
/// * `strict_mode` - If true, orphans are reported as errors; otherwise as warnings.
///
/// # Returns
/// A list of validation errors/warnings for orphan items.
pub fn check_orphans(graph: &KnowledgeGraph, _strict_mode: bool) -> Vec<ValidationError> {
    graph
        .orphans()
        .into_iter()
        .map(|item| ValidationError::OrphanItem {
            id: item.id.clone(),
            item_type: item.item_type,
            location: Some(item.source.clone()),
        })
        .collect()
}

/// Returns whether an orphan error should be treated as an error or warning.
pub fn is_orphan_error(strict_mode: bool) -> bool {
    strict_mode
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ItemId, ItemType, UpstreamRefs};
    use crate::test_utils::{create_test_item, create_test_item_with_upstream};

    #[test]
    fn test_solution_not_orphan() {
        let mut graph = KnowledgeGraph::new(false);
        graph.add_item(create_test_item("SOL-001", ItemType::Solution));

        let errors = check_orphans(&graph, false);
        assert!(
            errors.is_empty(),
            "Solutions should not be reported as orphans"
        );
    }

    #[test]
    fn test_use_case_orphan_detected() {
        let mut graph = KnowledgeGraph::new(false);
        graph.add_item(create_test_item("UC-001", ItemType::UseCase));

        let errors = check_orphans(&graph, false);
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
        let mut graph = KnowledgeGraph::new(false);
        graph.add_item(create_test_item("SOL-001", ItemType::Solution));
        graph.add_item(create_test_item_with_upstream(
            "UC-001",
            ItemType::UseCase,
            UpstreamRefs {
                refines: vec![ItemId::new_unchecked("SOL-001")],
                ..Default::default()
            },
        ));

        let errors = check_orphans(&graph, false);
        assert!(errors.is_empty());
    }
}

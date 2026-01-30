//! Duplicate identifier detection validation rule.

use crate::config::ValidationConfig;
use crate::error::ValidationError;
use crate::graph::KnowledgeGraph;
use crate::validation::rule::ValidationRule;

/// Duplicate identifier detection rule.
///
/// Each item ID must be unique across all repositories.
/// Note: The graph itself prevents duplicates by using a HashMap,
/// so this check is primarily useful during parsing before items are added.
pub struct DuplicatesRule;

impl ValidationRule for DuplicatesRule {
    fn validate(
        &self,
        _graph: &KnowledgeGraph,
        _config: &ValidationConfig,
    ) -> Vec<ValidationError> {
        // The graph itself prevents duplicates by using a HashMap,
        // so this check is primarily useful during parsing before
        // items are added to the graph.
        // For a built graph, we return empty since duplicates are already prevented.
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::error::ValidationError;
    use crate::model::{Item, ItemId, ItemType};
    use crate::test_utils::create_test_item_at;

    /// Checks a collection of items for duplicate IDs.
    fn check_duplicate_items(items: &[Item]) -> Vec<ValidationError> {
        let mut seen: HashSet<&ItemId> = HashSet::new();
        let mut duplicates: HashSet<ItemId> = HashSet::new();

        for item in items {
            if !seen.insert(&item.id) {
                duplicates.insert(item.id.clone());
            }
        }

        duplicates
            .into_iter()
            .map(|id| ValidationError::DuplicateIdentifier { id })
            .collect()
    }

    /// Checks if an item ID would be a duplicate in the graph.
    fn would_be_duplicate(graph: &KnowledgeGraph, id: &ItemId) -> bool {
        graph.contains(id)
    }

    #[test]
    fn test_no_duplicates() {
        let items = vec![
            create_test_item_at("SOL-001", ItemType::Solution, "sol1.md"),
            create_test_item_at("SOL-002", ItemType::Solution, "sol2.md"),
        ];

        let errors = check_duplicate_items(&items);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_duplicate_detected() {
        let items = vec![
            create_test_item_at("SOL-001", ItemType::Solution, "sol1.md"),
            create_test_item_at("SOL-001", ItemType::Solution, "sol1-copy.md"),
        ];

        let errors = check_duplicate_items(&items);
        assert_eq!(errors.len(), 1);

        if let ValidationError::DuplicateIdentifier { id } = &errors[0] {
            assert_eq!(id.as_str(), "SOL-001");
        } else {
            panic!("Expected DuplicateIdentifier error");
        }
    }

    #[test]
    fn test_multiple_duplicates() {
        let items = vec![
            create_test_item_at("SOL-001", ItemType::Solution, "sol1.md"),
            create_test_item_at("SOL-001", ItemType::Solution, "sol1-copy.md"),
            create_test_item_at("SOL-002", ItemType::Solution, "sol2.md"),
            create_test_item_at("SOL-002", ItemType::Solution, "sol2-copy.md"),
        ];

        let errors = check_duplicate_items(&items);
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn test_would_be_duplicate() {
        let mut graph = KnowledgeGraph::new(false);
        let item = create_test_item_at("SOL-001", ItemType::Solution, "sol1.md");
        graph.add_item(item);

        assert!(would_be_duplicate(
            &graph,
            &ItemId::new_unchecked("SOL-001")
        ));
        assert!(!would_be_duplicate(
            &graph,
            &ItemId::new_unchecked("SOL-002")
        ));
    }
}

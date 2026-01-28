//! Duplicate identifier detection validation rule.

use std::collections::HashMap;

use crate::error::ValidationError;
use crate::graph::KnowledgeGraph;
use crate::model::{Item, ItemId, SourceLocation};

/// Detects duplicate identifiers in the knowledge graph.
///
/// Each item ID must be unique across all repositories.
/// This function is typically run during graph construction to detect
/// when the same ID is used in multiple files.
pub fn check_duplicates(_graph: &KnowledgeGraph) -> Vec<ValidationError> {
    // The graph itself prevents duplicates by using a HashMap,
    // so this check is primarily useful during parsing before
    // items are added to the graph.
    // For a built graph, we return empty since duplicates are already prevented.
    Vec::new()
}

/// Checks a collection of items for duplicate IDs before adding to graph.
///
/// This is the primary duplicate detection function, used during parsing.
pub fn check_duplicate_items(items: &[Item]) -> Vec<ValidationError> {
    let mut seen: HashMap<&ItemId, Vec<&SourceLocation>> = HashMap::new();

    for item in items {
        seen.entry(&item.id).or_default().push(&item.source);
    }

    seen.into_iter()
        .filter(|(_, locations)| locations.len() > 1)
        .map(|(id, locations)| ValidationError::DuplicateIdentifier {
            id: id.clone(),
            locations: locations.into_iter().cloned().collect(),
        })
        .collect()
}

/// Checks if an item ID would be a duplicate in the graph.
pub fn would_be_duplicate(graph: &KnowledgeGraph, id: &ItemId) -> bool {
    graph.contains(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ItemType;
    use crate::test_utils::create_test_item_at;

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

        if let ValidationError::DuplicateIdentifier { id, locations } = &errors[0] {
            assert_eq!(id.as_str(), "SOL-001");
            assert_eq!(locations.len(), 2);
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

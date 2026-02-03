//! Item lookup and traceability query operations.

use strsim::levenshtein;

use crate::graph::{
    KnowledgeGraph, TraversalOptions, TraversalResult, traverse_downstream, traverse_upstream,
};
use crate::model::{Item, ItemId, ItemType};

/// Result of looking up an item.
#[derive(Debug)]
pub enum LookupResult<'a> {
    /// Item found.
    Found(&'a Item),
    /// Item not found, but similar items exist.
    NotFound {
        /// Suggestions for similar item IDs.
        suggestions: Vec<&'a ItemId>,
    },
}

/// Query engine for traceability operations.
#[derive(Debug)]
pub struct QueryEngine<'a> {
    graph: &'a KnowledgeGraph,
}

impl<'a> QueryEngine<'a> {
    /// Creates a new query engine.
    pub fn new(graph: &'a KnowledgeGraph) -> Self {
        Self { graph }
    }

    /// Looks up an item by ID.
    ///
    /// If the item is not found, returns suggestions for similar IDs.
    pub fn lookup(&self, id: &str) -> LookupResult<'a> {
        let item_id = ItemId::new_unchecked(id);

        if let Some(item) = self.graph.get(&item_id) {
            return LookupResult::Found(item);
        }

        // Item not found, find similar IDs
        let suggestions = self.find_similar_ids(id, 5);
        LookupResult::NotFound { suggestions }
    }

    /// Finds item IDs similar to the given string using Levenshtein distance.
    fn find_similar_ids(&self, query: &str, max_suggestions: usize) -> Vec<&'a ItemId> {
        find_similar_ids_scored(self.graph, query, max_suggestions)
            .into_iter()
            .map(|(id, _)| id)
            .collect()
    }

    /// Queries the upstream traceability chain for an item.
    pub fn trace_upstream(
        &self,
        id: &ItemId,
        options: &TraversalOptions,
    ) -> Option<TraversalResult> {
        traverse_upstream(self.graph, id, options)
    }

    /// Queries the downstream traceability chain for an item.
    pub fn trace_downstream(
        &self,
        id: &ItemId,
        options: &TraversalOptions,
    ) -> Option<TraversalResult> {
        traverse_downstream(self.graph, id, options)
    }

    /// Gets an item by ID.
    pub fn get(&self, id: &ItemId) -> Option<&'a Item> {
        self.graph.get(id)
    }

    /// Gets all items by type.
    pub fn items_by_type(&self, item_type: ItemType) -> Vec<&'a Item> {
        self.graph.items_by_type(item_type)
    }

    /// Returns the graph reference.
    pub fn graph(&self) -> &'a KnowledgeGraph {
        self.graph
    }
}

/// Gets direct parents of an item.
pub fn get_parents<'a>(graph: &'a KnowledgeGraph, id: &ItemId) -> Vec<&'a Item> {
    graph.parents(id)
}

/// Gets direct children of an item.
pub fn get_children<'a>(graph: &'a KnowledgeGraph, id: &ItemId) -> Vec<&'a Item> {
    graph.children(id)
}

/// Finds item IDs similar to the given query string using Levenshtein distance (FR-061).
///
/// Returns up to `max_suggestions` similar item IDs, sorted by distance.
/// Only includes suggestions with a reasonable edit distance.
pub fn find_similar_ids(
    graph: &KnowledgeGraph,
    query: &str,
    max_suggestions: usize,
) -> Vec<String> {
    find_similar_ids_scored(graph, query, max_suggestions)
        .into_iter()
        .map(|(id, _)| id.as_str().to_string())
        .collect()
}

/// Core implementation for finding similar IDs with Levenshtein distance scoring.
///
/// Returns item IDs with their edit distances, sorted by distance (ascending).
/// Filters to only include suggestions with reasonable edit distance.
fn find_similar_ids_scored<'a>(
    graph: &'a KnowledgeGraph,
    query: &str,
    max_suggestions: usize,
) -> Vec<(&'a ItemId, usize)> {
    let query_lower = query.to_lowercase();

    let mut scored: Vec<_> = graph
        .item_ids()
        .map(|id| {
            let id_lower = id.as_str().to_lowercase();
            let distance = levenshtein(&query_lower, &id_lower);
            (id, distance)
        })
        .collect();

    // Sort by distance (ascending)
    scored.sort_by_key(|(_, distance)| *distance);

    // Take top suggestions with reasonable distance
    scored
        .into_iter()
        .filter(|(_, distance)| {
            // Only suggest if distance is reasonable (less than half the query length)
            *distance <= query.len().max(3)
        })
        .take(max_suggestions)
        .collect()
}

/// Looks up an item by ID, returning suggestions if not found (FR-054, FR-061).
///
/// This is a convenience function for edit command lookups.
pub fn lookup_item_or_suggest<'a>(
    graph: &'a KnowledgeGraph,
    id: &str,
) -> Result<&'a Item, crate::error::EditError> {
    let item_id = ItemId::new_unchecked(id);

    if let Some(item) = graph.get(&item_id) {
        return Ok(item);
    }

    // Item not found, find similar IDs for suggestions
    let suggestions = find_similar_ids(graph, id, 3);
    Err(crate::error::EditError::ItemNotFound {
        id: id.to_string(),
        suggestions,
    })
}

/// Error when parent items are missing for a given item type (FR-052).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingParentError {
    /// The item type that requires a parent.
    pub item_type: String,
    /// The parent type that is missing.
    pub parent_type: String,
}

impl std::fmt::Display for MissingParentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cannot create {}: no {} items exist. Create a {} first.",
            self.item_type, self.parent_type, self.parent_type
        )
    }
}

impl std::error::Error for MissingParentError {}

/// Checks if parent items exist for the given item type (FR-052).
///
/// Solution has no parent requirement and always returns Ok.
/// If no graph is available, allows creation (cannot validate).
pub fn check_parent_exists(
    item_type: ItemType,
    graph: Option<&KnowledgeGraph>,
) -> Result<(), MissingParentError> {
    let Some(parent_type) = item_type.required_parent_type() else {
        return Ok(());
    };

    let Some(graph) = graph else {
        return Ok(());
    };

    let has_parents = graph.items().any(|item| item.item_type == parent_type);

    if has_parents {
        Ok(())
    } else {
        Err(MissingParentError {
            item_type: item_type.display_name().to_string(),
            parent_type: parent_type.display_name().to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::KnowledgeGraphBuilder;
    use crate::model::UpstreamRefs;
    use crate::test_utils::{create_test_item, create_test_item_with_upstream};

    #[test]
    fn test_lookup_found() {
        let graph = KnowledgeGraphBuilder::new()
            .add_item(create_test_item("SOL-001", ItemType::Solution))
            .build()
            .unwrap();

        let engine = QueryEngine::new(&graph);
        let result = engine.lookup("SOL-001");

        match result {
            LookupResult::Found(item) => {
                assert_eq!(item.id.as_str(), "SOL-001");
            }
            LookupResult::NotFound { .. } => panic!("Expected to find item"),
        }
    }

    #[test]
    fn test_lookup_not_found_with_suggestions() {
        let graph = KnowledgeGraphBuilder::new()
            .add_item(create_test_item("SOL-001", ItemType::Solution))
            .add_item(create_test_item("SOL-002", ItemType::Solution))
            .add_item(create_test_item("UC-001", ItemType::UseCase))
            .build()
            .unwrap();

        let engine = QueryEngine::new(&graph);
        let result = engine.lookup("SOL-003");

        match result {
            LookupResult::Found(_) => panic!("Should not find item"),
            LookupResult::NotFound { suggestions } => {
                // Should suggest similar IDs
                assert!(!suggestions.is_empty());
                // SOL-001 and SOL-002 should be suggested before UC-001
                let suggestion_strs: Vec<_> = suggestions.iter().map(|id| id.as_str()).collect();
                assert!(
                    suggestion_strs.contains(&"SOL-001") || suggestion_strs.contains(&"SOL-002")
                );
            }
        }
    }

    #[test]
    fn test_trace_upstream() {
        let sol = create_test_item("SOL-001", ItemType::Solution);
        let uc = create_test_item_with_upstream(
            "UC-001",
            ItemType::UseCase,
            UpstreamRefs {
                refines: vec![ItemId::new_unchecked("SOL-001")],
                ..Default::default()
            },
        );

        let graph = KnowledgeGraphBuilder::new()
            .add_item(sol)
            .add_item(uc)
            .build()
            .unwrap();

        let engine = QueryEngine::new(&graph);
        let result =
            engine.trace_upstream(&ItemId::new_unchecked("UC-001"), &TraversalOptions::new());

        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.items.len(), 2);
    }

    #[test]
    fn test_trace_downstream() {
        let sol = create_test_item("SOL-001", ItemType::Solution);
        let uc = create_test_item_with_upstream(
            "UC-001",
            ItemType::UseCase,
            UpstreamRefs {
                refines: vec![ItemId::new_unchecked("SOL-001")],
                ..Default::default()
            },
        );

        let graph = KnowledgeGraphBuilder::new()
            .add_item(sol)
            .add_item(uc)
            .build()
            .unwrap();

        let engine = QueryEngine::new(&graph);
        let result =
            engine.trace_downstream(&ItemId::new_unchecked("SOL-001"), &TraversalOptions::new());

        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.items.len(), 2);
    }
}

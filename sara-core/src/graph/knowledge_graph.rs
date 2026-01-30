//! Knowledge graph implementation using petgraph.

use petgraph::Direction;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::HashMap;

use crate::model::{Item, ItemId, ItemType, RelationshipType};

/// The main knowledge graph container.
#[derive(Debug)]
pub struct KnowledgeGraph {
    /// The underlying directed graph.
    graph: DiGraph<Item, RelationshipType>,

    /// Index for O(1) lookup by ItemId.
    index: HashMap<ItemId, NodeIndex>,
}

impl KnowledgeGraph {
    /// Creates a new empty knowledge graph.
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            index: HashMap::new(),
        }
    }

    /// Returns the number of items in the graph.
    pub fn item_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Returns the number of relationships in the graph.
    pub fn relationship_count(&self) -> usize {
        self.graph.edge_count()
    }

    /// Adds an item to the graph.
    pub fn add_item(&mut self, item: Item) -> NodeIndex {
        let id = item.id.clone();
        let idx = self.graph.add_node(item);
        self.index.insert(id, idx);
        idx
    }

    /// Adds a relationship between two items.
    pub fn add_relationship(
        &mut self,
        from: &ItemId,
        to: &ItemId,
        rel_type: RelationshipType,
    ) -> Option<()> {
        let from_idx = self.index.get(from)?;
        let to_idx = self.index.get(to)?;
        self.graph.add_edge(*from_idx, *to_idx, rel_type);
        Some(())
    }

    /// Gets an item by ID.
    pub fn get(&self, id: &ItemId) -> Option<&Item> {
        let idx = self.index.get(id)?;
        self.graph.node_weight(*idx)
    }

    /// Gets a mutable reference to an item by ID.
    pub fn get_mut(&mut self, id: &ItemId) -> Option<&mut Item> {
        let idx = self.index.get(id)?;
        self.graph.node_weight_mut(*idx)
    }

    /// Checks if an item exists in the graph.
    pub fn contains(&self, id: &ItemId) -> bool {
        self.index.contains_key(id)
    }

    /// Returns all items in the graph.
    pub fn items(&self) -> impl Iterator<Item = &Item> {
        self.graph.node_weights()
    }

    /// Returns all item IDs in the graph.
    pub fn item_ids(&self) -> impl Iterator<Item = &ItemId> {
        self.index.keys()
    }

    /// Returns all items of a specific type.
    pub fn items_by_type(&self, item_type: ItemType) -> Vec<&Item> {
        self.graph
            .node_weights()
            .filter(|item| item.item_type == item_type)
            .collect()
    }

    /// Returns the count of items by type.
    pub fn count_by_type(&self) -> HashMap<ItemType, usize> {
        let mut counts = HashMap::new();
        for item in self.graph.node_weights() {
            *counts.entry(item.item_type).or_insert(0) += 1;
        }
        counts
    }

    /// Returns direct parents of an item (items that this item relates to upstream).
    pub fn parents(&self, id: &ItemId) -> Vec<&Item> {
        let Some(idx) = self.index.get(id) else {
            return Vec::new();
        };

        self.graph
            .edges_directed(*idx, Direction::Outgoing)
            .filter(|edge| edge.weight().is_upstream())
            .filter_map(|edge| self.graph.node_weight(edge.target()))
            .collect()
    }

    /// Returns direct children of an item (items that relate to this item downstream).
    pub fn children(&self, id: &ItemId) -> Vec<&Item> {
        let Some(idx) = self.index.get(id) else {
            return Vec::new();
        };

        self.graph
            .edges_directed(*idx, Direction::Incoming)
            .filter(|edge| edge.weight().is_upstream())
            .filter_map(|edge| self.graph.node_weight(edge.source()))
            .collect()
    }

    /// Returns all items with no upstream parents (potential orphans).
    pub fn orphans(&self) -> Vec<&Item> {
        self.graph
            .node_weights()
            .filter(|item| {
                // Solutions are allowed to have no parents (root of hierarchy)
                if item.item_type.is_root() {
                    return false;
                }
                // Check if item has any upstream references
                item.upstream.is_empty()
            })
            .collect()
    }

    /// Returns the underlying petgraph for advanced operations.
    pub fn inner(&self) -> &DiGraph<Item, RelationshipType> {
        &self.graph
    }

    /// Returns a mutable reference to the underlying petgraph.
    pub fn inner_mut(&mut self) -> &mut DiGraph<Item, RelationshipType> {
        &mut self.graph
    }

    /// Returns the node index for an item ID.
    pub fn node_index(&self, id: &ItemId) -> Option<NodeIndex> {
        self.index.get(id).copied()
    }

    /// Checks if the graph has cycles.
    pub fn has_cycles(&self) -> bool {
        petgraph::algo::is_cyclic_directed(&self.graph)
    }

    /// Returns all relationships in the graph.
    pub fn relationships(&self) -> Vec<(ItemId, ItemId, RelationshipType)> {
        self.graph
            .edge_references()
            .filter_map(|edge| {
                let from = self.graph.node_weight(edge.source())?;
                let to = self.graph.node_weight(edge.target())?;
                Some((from.id.clone(), to.id.clone(), *edge.weight()))
            })
            .collect()
    }
}

impl Default for KnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::create_test_item;

    #[test]
    fn test_add_and_get_item() {
        let mut graph = KnowledgeGraph::new();
        let item = create_test_item("SOL-001", ItemType::Solution);
        graph.add_item(item);

        let id = ItemId::new_unchecked("SOL-001");
        assert!(graph.contains(&id));
        assert_eq!(graph.get(&id).unwrap().name, "Test SOL-001");
    }

    #[test]
    fn test_items_by_type() {
        let mut graph = KnowledgeGraph::new();
        graph.add_item(create_test_item("SOL-001", ItemType::Solution));
        graph.add_item(create_test_item("UC-001", ItemType::UseCase));
        graph.add_item(create_test_item("UC-002", ItemType::UseCase));

        let solutions = graph.items_by_type(ItemType::Solution);
        assert_eq!(solutions.len(), 1);

        let use_cases = graph.items_by_type(ItemType::UseCase);
        assert_eq!(use_cases.len(), 2);
    }

    #[test]
    fn test_item_count() {
        let mut graph = KnowledgeGraph::new();
        assert_eq!(graph.item_count(), 0);

        graph.add_item(create_test_item("SOL-001", ItemType::Solution));
        assert_eq!(graph.item_count(), 1);

        graph.add_item(create_test_item("UC-001", ItemType::UseCase));
        assert_eq!(graph.item_count(), 2);
    }
}

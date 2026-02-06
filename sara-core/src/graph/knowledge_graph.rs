//! Knowledge graph implementation using petgraph.

use petgraph::Direction;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::SaraError;
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
    fn add_item(&mut self, item: Item) -> NodeIndex {
        let id = item.id.clone();
        let idx = self.graph.add_node(item);
        self.index.insert(id, idx);
        idx
    }

    /// Adds a relationship between two items.
    fn add_relationship(&mut self, from: &ItemId, to: &ItemId, rel_type: RelationshipType) {
        if let (Some(from_idx), Some(to_idx)) = (self.index.get(from), self.index.get(to)) {
            self.graph.add_edge(*from_idx, *to_idx, rel_type);
        }
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
                // Check if item has any upstream relationships
                !item.has_upstream()
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

/// Builder for constructing knowledge graphs.
#[derive(Debug, Default)]
pub struct KnowledgeGraphBuilder {
    items: Vec<Item>,
    repositories: Vec<PathBuf>,
}

impl KnowledgeGraphBuilder {
    /// Creates a new graph builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a repository path.
    pub fn add_repository(mut self, path: impl Into<PathBuf>) -> Self {
        self.repositories.push(path.into());
        self
    }

    /// Adds an item to the graph.
    pub fn add_item(mut self, item: Item) -> Self {
        self.items.push(item);
        self
    }

    /// Adds multiple items to the graph.
    pub fn add_items(mut self, items: impl IntoIterator<Item = Item>) -> Self {
        self.items.extend(items);
        self
    }

    /// Builds the knowledge graph.
    pub fn build(self) -> Result<KnowledgeGraph, SaraError> {
        let mut graph = KnowledgeGraph::new();

        // First pass: add all items
        for item in &self.items {
            graph.add_item(item.clone());
        }

        // Second pass: add relationships based on item references
        for item in &self.items {
            self.add_relationships_for_item(&mut graph, item);
        }

        Ok(graph)
    }

    /// Adds relationships for an item based on its references.
    fn add_relationships_for_item(&self, graph: &mut KnowledgeGraph, item: &Item) {
        // Add all relationships from the item's relationships vec
        for rel in &item.relationships {
            graph.add_relationship(&item.id, &rel.to, rel.relationship_type);

            // For certain relationship types, add the inverse edge for bidirectional traversal
            match rel.relationship_type {
                RelationshipType::Justifies => {
                    graph.add_relationship(&rel.to, &item.id, RelationshipType::IsJustifiedBy);
                }
                RelationshipType::IsRefinedBy => {
                    graph.add_relationship(&rel.to, &item.id, RelationshipType::Refines);
                }
                RelationshipType::Derives => {
                    graph.add_relationship(&rel.to, &item.id, RelationshipType::DerivesFrom);
                }
                RelationshipType::IsSatisfiedBy => {
                    graph.add_relationship(&rel.to, &item.id, RelationshipType::Satisfies);
                }
                RelationshipType::IsJustifiedBy => {
                    graph.add_relationship(&rel.to, &item.id, RelationshipType::Justifies);
                }
                _ => {}
            }
        }

        // Add peer dependencies (for requirement types, stored in attributes)
        for target_id in item.attributes.depends_on() {
            graph.add_relationship(&item.id, target_id, RelationshipType::DependsOn);
        }

        // ADR supersession (peer relationships between ADRs, stored in attributes)
        for target_id in item.attributes.supersedes() {
            graph.add_relationship(&item.id, target_id, RelationshipType::Supersedes);
            // Add inverse: target is superseded by this ADR
            graph.add_relationship(target_id, &item.id, RelationshipType::IsSupersededBy);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Relationship;
    use crate::test_utils::{create_test_adr, create_test_item, create_test_item_with_relationships};

    #[test]
    fn test_add_and_get_item() {
        let graph = KnowledgeGraphBuilder::new()
            .add_item(create_test_item("SOL-001", ItemType::Solution))
            .build()
            .unwrap();

        let id = ItemId::new_unchecked("SOL-001");
        assert!(graph.contains(&id));
        assert_eq!(graph.get(&id).unwrap().name, "Test SOL-001");
    }

    #[test]
    fn test_items_by_type() {
        let graph = KnowledgeGraphBuilder::new()
            .add_item(create_test_item("SOL-001", ItemType::Solution))
            .add_item(create_test_item("UC-001", ItemType::UseCase))
            .add_item(create_test_item("UC-002", ItemType::UseCase))
            .build()
            .unwrap();

        let solutions = graph.items_by_type(ItemType::Solution);
        assert_eq!(solutions.len(), 1);

        let use_cases = graph.items_by_type(ItemType::UseCase);
        assert_eq!(use_cases.len(), 2);
    }

    #[test]
    fn test_item_count() {
        let graph = KnowledgeGraphBuilder::new().build().unwrap();
        assert_eq!(graph.item_count(), 0);

        let graph = KnowledgeGraphBuilder::new()
            .add_item(create_test_item("SOL-001", ItemType::Solution))
            .build()
            .unwrap();
        assert_eq!(graph.item_count(), 1);

        let graph = KnowledgeGraphBuilder::new()
            .add_item(create_test_item("SOL-001", ItemType::Solution))
            .add_item(create_test_item("UC-001", ItemType::UseCase))
            .build()
            .unwrap();
        assert_eq!(graph.item_count(), 2);
    }

    #[test]
    fn test_build_simple_graph() {
        let graph = KnowledgeGraphBuilder::new()
            .add_item(create_test_item("SOL-001", ItemType::Solution))
            .build()
            .unwrap();

        assert_eq!(graph.item_count(), 1);
    }

    #[test]
    fn test_build_graph_with_relationships() {
        let sol = create_test_item("SOL-001", ItemType::Solution);
        let uc = create_test_item_with_relationships(
            "UC-001",
            ItemType::UseCase,
            vec![Relationship::new(
                ItemId::new_unchecked("SOL-001"),
                RelationshipType::Refines,
            )],
        );

        let graph = KnowledgeGraphBuilder::new()
            .add_item(sol)
            .add_item(uc)
            .build()
            .unwrap();

        assert_eq!(graph.item_count(), 2);
        assert_eq!(graph.relationship_count(), 1);
    }

    #[test]
    fn test_adr_justifies_relationship() {
        // Create a system architecture item
        let sysarch = create_test_item("SYSARCH-001", ItemType::SystemArchitecture);
        // Create an ADR that justifies it
        let adr = create_test_adr("ADR-001", &["SYSARCH-001"], &[]);

        let graph = KnowledgeGraphBuilder::new()
            .add_item(sysarch)
            .add_item(adr)
            .build()
            .unwrap();

        assert_eq!(graph.item_count(), 2);
        // ADR-001 -> Justifies -> SYSARCH-001
        // SYSARCH-001 -> IsJustifiedBy -> ADR-001
        assert_eq!(graph.relationship_count(), 2);
    }

    #[test]
    fn test_adr_supersession_relationship() {
        // Create two ADRs where the newer one supersedes the older
        let adr_old = create_test_adr("ADR-001", &[], &[]);
        let adr_new = create_test_adr("ADR-002", &[], &["ADR-001"]);

        let graph = KnowledgeGraphBuilder::new()
            .add_item(adr_old)
            .add_item(adr_new)
            .build()
            .unwrap();

        assert_eq!(graph.item_count(), 2);
        // ADR-002 -> Supersedes -> ADR-001
        // ADR-001 -> IsSupersededBy -> ADR-002
        assert_eq!(graph.relationship_count(), 2);
    }
}

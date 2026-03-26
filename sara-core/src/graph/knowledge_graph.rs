//! Knowledge graph implementation using petgraph.

use std::collections::HashMap;

use petgraph::Direction;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use strsim::levenshtein;

use crate::error::SaraError;
use crate::model::{Item, ItemId, ItemType, RelationshipType};

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

    /// Looks up an item by ID.
    ///
    /// If the item is not found, returns suggestions for similar IDs.
    pub fn lookup(&self, id: &str) -> LookupResult<'_> {
        let item_id = ItemId::new_unchecked(id);

        if let Some(item) = self.get(&item_id) {
            return LookupResult::Found(item);
        }

        let suggestions = self
            .find_similar_ids_scored(id, 5)
            .into_iter()
            .map(|(id, _)| id)
            .collect();
        LookupResult::NotFound { suggestions }
    }

    /// Looks up an item by ID, returning suggestions if not found.
    ///
    /// Returns a `SaraError::ItemNotFound` with similar ID suggestions
    /// when the item is missing.
    pub fn lookup_or_suggest(&self, id: &str) -> Result<&Item, SaraError> {
        let item_id = ItemId::new_unchecked(id);

        if let Some(item) = self.get(&item_id) {
            return Ok(item);
        }

        let suggestions = self.find_similar_ids(id, 3);
        Err(SaraError::ItemNotFound {
            id: id.to_string(),
            suggestions,
        })
    }

    /// Finds item IDs similar to the given query string using Levenshtein distance.
    ///
    /// Returns up to `max_suggestions` similar item IDs, sorted by distance.
    /// Only includes suggestions with a reasonable edit distance.
    pub fn find_similar_ids(&self, query: &str, max_suggestions: usize) -> Vec<String> {
        self.find_similar_ids_scored(query, max_suggestions)
            .into_iter()
            .map(|(id, _)| id.as_str().to_string())
            .collect()
    }

    /// Checks if parent items exist for the given item type.
    ///
    /// Solution has no parent requirement and always returns Ok.
    pub fn check_parent_exists(&self, item_type: ItemType) -> Result<(), SaraError> {
        let Some(parent_type) = item_type.required_parent_type() else {
            return Ok(());
        };

        let has_parents = self.items().any(|item| item.item_type == parent_type);

        if has_parents {
            Ok(())
        } else {
            Err(SaraError::MissingParent {
                item_type: item_type.display_name().to_string(),
                parent_type: parent_type.display_name().to_string(),
            })
        }
    }

    /// Core implementation for finding similar IDs with Levenshtein distance scoring.
    fn find_similar_ids_scored(
        &self,
        query: &str,
        max_suggestions: usize,
    ) -> Vec<(&ItemId, usize)> {
        let query_lower = query.to_lowercase();

        let mut scored: Vec<_> = self
            .item_ids()
            .map(|id| {
                let id_lower = id.as_str().to_lowercase();
                let distance = levenshtein(&query_lower, &id_lower);
                (id, distance)
            })
            .collect();

        scored.sort_by_key(|(_, distance)| *distance);

        scored
            .into_iter()
            .filter(|(_, distance)| *distance <= query.len().max(3))
            .take(max_suggestions)
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
}

impl KnowledgeGraphBuilder {
    /// Creates a new graph builder.
    pub fn new() -> Self {
        Self::default()
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

        // First pass: collect all relationship edges from items (before moving them)
        let edges: Vec<_> = self.items.iter().flat_map(Self::collect_edges).collect();

        // Second pass: move items into the graph (no clone needed)
        for item in self.items {
            graph.add_item(item);
        }

        // Third pass: add relationship edges
        for (from, to, rel_type) in edges {
            graph.add_relationship(&from, &to, rel_type);
        }

        Ok(graph)
    }

    /// Collects all relationship edges for an item as (from, to, type) tuples.
    fn collect_edges(item: &Item) -> Vec<(ItemId, ItemId, RelationshipType)> {
        let mut edges = Vec::new();

        for rel in &item.relationships {
            edges.push((item.id.clone(), rel.to.clone(), rel.relationship_type));

            // For certain relationship types, add the inverse edge for bidirectional traversal
            let inverse = match rel.relationship_type {
                RelationshipType::Justifies => Some(RelationshipType::IsJustifiedBy),
                RelationshipType::IsRefinedBy => Some(RelationshipType::Refines),
                RelationshipType::Derives => Some(RelationshipType::DerivesFrom),
                RelationshipType::IsSatisfiedBy => Some(RelationshipType::Satisfies),
                RelationshipType::IsJustifiedBy => Some(RelationshipType::Justifies),
                _ => None,
            };
            if let Some(inv_type) = inverse {
                edges.push((rel.to.clone(), item.id.clone(), inv_type));
            }
        }

        // Peer dependencies (for requirement types, stored in attributes)
        for target_id in item.attributes.depends_on() {
            edges.push((
                item.id.clone(),
                target_id.clone(),
                RelationshipType::DependsOn,
            ));
        }

        // ADR supersession (peer relationships between ADRs, stored in attributes)
        for target_id in item.attributes.supersedes() {
            edges.push((
                item.id.clone(),
                target_id.clone(),
                RelationshipType::Supersedes,
            ));
            edges.push((
                target_id.clone(),
                item.id.clone(),
                RelationshipType::IsSupersededBy,
            ));
        }

        edges
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Relationship;
    use crate::test_utils::{
        create_test_adr, create_test_item, create_test_item_with_relationships,
    };

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

    #[test]
    fn test_lookup_found() {
        let graph = KnowledgeGraphBuilder::new()
            .add_item(create_test_item("SOL-001", ItemType::Solution))
            .build()
            .unwrap();

        match graph.lookup("SOL-001") {
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

        match graph.lookup("SOL-003") {
            LookupResult::Found(_) => panic!("Should not find item"),
            LookupResult::NotFound { suggestions } => {
                assert!(!suggestions.is_empty());
                let suggestion_strs: Vec<_> = suggestions.iter().map(|id| id.as_str()).collect();
                assert!(
                    suggestion_strs.contains(&"SOL-001") || suggestion_strs.contains(&"SOL-002")
                );
            }
        }
    }
}

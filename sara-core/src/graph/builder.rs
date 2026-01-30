//! Graph builder for constructing knowledge graphs.

use std::path::PathBuf;

use crate::error::SaraError;
use crate::graph::KnowledgeGraph;
use crate::model::{Item, ItemId, RelationshipType};

/// Builder for constructing knowledge graphs.
#[derive(Debug, Default)]
pub struct GraphBuilder {
    items: Vec<Item>,
    strict_mode: bool,
    repositories: Vec<PathBuf>,
}

impl GraphBuilder {
    /// Creates a new graph builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets strict orphan checking mode.
    pub fn with_strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
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
        let mut graph = KnowledgeGraph::new(self.strict_mode);

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
    ///
    /// All relationships are bidirectional: adding one direction automatically
    /// creates the inverse edge. This ensures the graph can be traversed in
    /// both directions regardless of which side declared the relationship.
    fn add_relationships_for_item(&self, graph: &mut KnowledgeGraph, item: &Item) {
        // All relationships from the item's relationships field
        for rel in &item.relationships {
            graph.add_relationship(&item.id, &rel.to, rel.relationship_type);
            // Add inverse for bidirectional traversal
            graph.add_relationship(&rel.to, &item.id, rel.relationship_type.inverse());
        }

        // Peer/attribute relationships (stored in attributes, not relationships)
        for target_id in item.attributes.depends_on() {
            graph.add_relationship(&item.id, target_id, RelationshipType::DependsOn);
            graph.add_relationship(target_id, &item.id, RelationshipType::IsRequiredBy);
        }
        for target_id in item.attributes.supersedes() {
            graph.add_relationship(&item.id, target_id, RelationshipType::Supersedes);
            graph.add_relationship(target_id, &item.id, RelationshipType::IsSupersededBy);
        }
    }
}

/// Resolves cross-repository references in the graph.
pub fn resolve_cross_repository_refs(graph: &mut KnowledgeGraph) -> Vec<(ItemId, ItemId)> {
    let mut unresolved = Vec::new();

    let referenced_ids: Vec<ItemId> = graph
        .items()
        .flat_map(|item| item.all_references())
        .cloned()
        .collect();

    for ref_id in referenced_ids {
        if !graph.contains(&ref_id) {
            for item in graph.items() {
                if item.all_references().any(|id| *id == ref_id) {
                    unresolved.push((item.id.clone(), ref_id.clone()));
                    break;
                }
            }
        }
    }

    unresolved
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ItemType;
    use crate::test_utils::{create_test_adr, create_test_item, create_test_item_with_relationships};

    #[test]
    fn test_build_simple_graph() {
        let graph = GraphBuilder::new()
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
            vec![(ItemId::new_unchecked("SOL-001"), RelationshipType::Refines)],
        );

        let graph = GraphBuilder::new()
            .add_item(sol)
            .add_item(uc)
            .build()
            .unwrap();

        assert_eq!(graph.item_count(), 2);
        assert_eq!(graph.relationship_count(), 2);
    }

    #[test]
    fn test_strict_mode() {
        let graph = GraphBuilder::new()
            .with_strict_mode(true)
            .add_item(create_test_item("SOL-001", ItemType::Solution))
            .build()
            .unwrap();

        assert!(graph.is_strict_mode());
    }

    #[test]
    fn test_adr_justifies_relationship() {
        // Create a system architecture item
        let sysarch = create_test_item("SYSARCH-001", ItemType::SystemArchitecture);
        // Create an ADR that justifies it
        let adr = create_test_adr("ADR-001", &["SYSARCH-001"], &[]);

        let graph = GraphBuilder::new()
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

        let graph = GraphBuilder::new()
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

//! Graph builder for constructing knowledge graphs.

#![allow(clippy::result_large_err)]

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
    fn add_relationships_for_item(&self, graph: &mut KnowledgeGraph, item: &Item) {
        // Add upstream relationships
        for target_id in &item.upstream.refines {
            graph.add_relationship(&item.id, target_id, RelationshipType::Refines);
        }
        for target_id in &item.upstream.derives_from {
            graph.add_relationship(&item.id, target_id, RelationshipType::DerivesFrom);
        }
        for target_id in &item.upstream.satisfies {
            graph.add_relationship(&item.id, target_id, RelationshipType::Satisfies);
        }

        // Add downstream relationships (and their inverse for bidirectional graph queries)
        for target_id in &item.downstream.is_refined_by {
            graph.add_relationship(&item.id, target_id, RelationshipType::IsRefinedBy);
            // Add inverse: target refines this item
            graph.add_relationship(target_id, &item.id, RelationshipType::Refines);
        }
        for target_id in &item.downstream.derives {
            graph.add_relationship(&item.id, target_id, RelationshipType::Derives);
            // Add inverse: target derives_from this item
            graph.add_relationship(target_id, &item.id, RelationshipType::DerivesFrom);
        }
        for target_id in &item.downstream.is_satisfied_by {
            graph.add_relationship(&item.id, target_id, RelationshipType::IsSatisfiedBy);
            // Add inverse: target satisfies this item
            graph.add_relationship(target_id, &item.id, RelationshipType::Satisfies);
        }

        // Add peer dependencies
        for target_id in &item.attributes.depends_on {
            graph.add_relationship(&item.id, target_id, RelationshipType::DependsOn);
        }
    }
}

/// Resolves cross-repository references in the graph.
pub fn resolve_cross_repository_refs(graph: &mut KnowledgeGraph) -> Vec<(ItemId, ItemId)> {
    let mut unresolved = Vec::new();

    // Collect all referenced IDs
    let referenced_ids: Vec<ItemId> = graph
        .items()
        .flat_map(|item| item.all_references())
        .cloned()
        .collect();

    // Check which ones are missing
    for ref_id in referenced_ids {
        if !graph.contains(&ref_id) {
            // Find the item that references this missing ID
            for item in graph.items() {
                if item.all_references().iter().any(|id| **id == ref_id) {
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
    use crate::model::{ItemBuilder, ItemType, SourceLocation, UpstreamRefs};
    use std::path::PathBuf;

    fn create_test_item(id: &str, item_type: ItemType) -> Item {
        let source = SourceLocation::new(PathBuf::from("/repo"), format!("{}.md", id), 1);
        let mut builder = ItemBuilder::new()
            .id(ItemId::new_unchecked(id))
            .item_type(item_type)
            .name(format!("Test {}", id))
            .source(source);

        if item_type.requires_specification() {
            builder = builder.specification("Test specification");
        }

        builder.build().unwrap()
    }

    fn create_test_item_with_upstream(
        id: &str,
        item_type: ItemType,
        upstream: UpstreamRefs,
    ) -> Item {
        let source = SourceLocation::new(PathBuf::from("/repo"), format!("{}.md", id), 1);
        let mut builder = ItemBuilder::new()
            .id(ItemId::new_unchecked(id))
            .item_type(item_type)
            .name(format!("Test {}", id))
            .source(source)
            .upstream(upstream);

        if item_type.requires_specification() {
            builder = builder.specification("Test specification");
        }

        builder.build().unwrap()
    }

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
        let uc = create_test_item_with_upstream(
            "UC-001",
            ItemType::UseCase,
            UpstreamRefs {
                refines: vec![ItemId::new_unchecked("SOL-001")],
                ..Default::default()
            },
        );

        let graph = GraphBuilder::new()
            .add_item(sol)
            .add_item(uc)
            .build()
            .unwrap();

        assert_eq!(graph.item_count(), 2);
        assert_eq!(graph.relationship_count(), 1);
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
}

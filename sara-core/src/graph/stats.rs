//! Graph statistics collection.

use std::collections::HashMap;

use serde::Serialize;

use crate::model::ItemType;

use super::KnowledgeGraph;

/// Statistics about a knowledge graph.
#[derive(Debug, Default, Clone, Serialize)]
pub struct GraphStats {
    /// Total number of items in the graph.
    pub item_count: usize,
    /// Total number of relationships in the graph.
    pub relationship_count: usize,
    /// Count of items by their type.
    pub items_by_type: HashMap<ItemType, usize>,
}

impl GraphStats {
    /// Collects statistics from a knowledge graph.
    pub fn from_graph(graph: &KnowledgeGraph) -> Self {
        Self {
            item_count: graph.item_count(),
            relationship_count: graph.relationship_count(),
            items_by_type: graph.count_by_type(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Item, ItemBuilder, ItemId, SourceLocation};
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

    #[test]
    fn test_graph_stats() {
        let mut graph = KnowledgeGraph::new(false);
        graph.add_item(create_test_item("SOL-001", ItemType::Solution));
        graph.add_item(create_test_item("UC-001", ItemType::UseCase));
        graph.add_item(create_test_item("UC-002", ItemType::UseCase));

        let stats = GraphStats::from_graph(&graph);

        assert_eq!(stats.item_count, 3);
        assert_eq!(stats.relationship_count, 0);
        assert_eq!(stats.items_by_type.get(&ItemType::Solution), Some(&1));
        assert_eq!(stats.items_by_type.get(&ItemType::UseCase), Some(&2));
    }
}

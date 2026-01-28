//! Graph diffing between two graph states.

use std::collections::HashSet;

use serde::Serialize;

use crate::graph::KnowledgeGraph;
use crate::model::{Item, ItemId};

/// A diff between two knowledge graphs.
#[derive(Debug, Clone, Serialize)]
pub struct GraphDiff {
    /// Items added (present in new, not in old).
    pub added_items: Vec<ItemDiff>,
    /// Items removed (present in old, not in new).
    pub removed_items: Vec<ItemDiff>,
    /// Items modified (present in both, but changed).
    pub modified_items: Vec<ItemModification>,
    /// Relationships added.
    pub added_relationships: Vec<RelationshipDiff>,
    /// Relationships removed.
    pub removed_relationships: Vec<RelationshipDiff>,
    /// Summary statistics.
    pub stats: DiffStats,
}

/// Representation of an item in a diff.
#[derive(Debug, Clone, Serialize)]
pub struct ItemDiff {
    pub id: String,
    pub name: String,
    pub item_type: String,
    pub file_path: String,
}

impl From<&Item> for ItemDiff {
    fn from(item: &Item) -> Self {
        Self {
            id: item.id.as_str().to_string(),
            name: item.name.clone(),
            item_type: item.item_type.display_name().to_string(),
            file_path: item.source.file_path.display().to_string(),
        }
    }
}

/// A modification to an item.
#[derive(Debug, Clone, Serialize)]
pub struct ItemModification {
    pub id: String,
    pub name: String,
    pub item_type: String,
    pub changes: Vec<FieldChange>,
}

/// A change to a specific field.
#[derive(Debug, Clone, Serialize)]
pub struct FieldChange {
    pub field: String,
    pub old_value: String,
    pub new_value: String,
}

/// A relationship in a diff.
#[derive(Debug, Clone, Serialize)]
pub struct RelationshipDiff {
    pub from_id: String,
    pub to_id: String,
    pub relationship_type: String,
}

/// Summary statistics for a diff.
#[derive(Debug, Clone, Default, Serialize)]
pub struct DiffStats {
    pub items_added: usize,
    pub items_removed: usize,
    pub items_modified: usize,
    pub relationships_added: usize,
    pub relationships_removed: usize,
}

impl GraphDiff {
    /// Computes the diff between two graphs.
    ///
    /// `old_graph` is the baseline (e.g., main branch).
    /// `new_graph` is the target (e.g., current HEAD).
    pub fn compute(old_graph: &KnowledgeGraph, new_graph: &KnowledgeGraph) -> Self {
        let mut added_items = Vec::new();
        let mut removed_items = Vec::new();
        let mut modified_items = Vec::new();

        // Collect item IDs from both graphs
        let old_ids: HashSet<_> = old_graph.item_ids().cloned().collect();
        let new_ids: HashSet<_> = new_graph.item_ids().cloned().collect();

        // Find added items (in new but not in old)
        for id in new_ids.difference(&old_ids) {
            if let Some(item) = new_graph.get(id) {
                added_items.push(ItemDiff::from(item));
            }
        }

        // Find removed items (in old but not in new)
        for id in old_ids.difference(&new_ids) {
            if let Some(item) = old_graph.get(id) {
                removed_items.push(ItemDiff::from(item));
            }
        }

        // Find modified items (in both, check for changes)
        for id in old_ids.intersection(&new_ids) {
            if let (Some(old_item), Some(new_item)) = (old_graph.get(id), new_graph.get(id)) {
                let changes = Self::compute_item_changes(old_item, new_item);
                if !changes.is_empty() {
                    modified_items.push(ItemModification {
                        id: id.as_str().to_string(),
                        name: new_item.name.clone(),
                        item_type: new_item.item_type.display_name().to_string(),
                        changes,
                    });
                }
            }
        }

        // Compute relationship diffs
        let old_rels: HashSet<_> = old_graph
            .relationships()
            .into_iter()
            .map(|(from, to, rel)| (from.as_str().to_string(), to.as_str().to_string(), rel))
            .collect();
        let new_rels: HashSet<_> = new_graph
            .relationships()
            .into_iter()
            .map(|(from, to, rel)| (from.as_str().to_string(), to.as_str().to_string(), rel))
            .collect();

        let added_relationships: Vec<_> = new_rels
            .difference(&old_rels)
            .map(|(from, to, rel)| RelationshipDiff {
                from_id: from.clone(),
                to_id: to.clone(),
                relationship_type: format!("{:?}", rel),
            })
            .collect();

        let removed_relationships: Vec<_> = old_rels
            .difference(&new_rels)
            .map(|(from, to, rel)| RelationshipDiff {
                from_id: from.clone(),
                to_id: to.clone(),
                relationship_type: format!("{:?}", rel),
            })
            .collect();

        let stats = DiffStats {
            items_added: added_items.len(),
            items_removed: removed_items.len(),
            items_modified: modified_items.len(),
            relationships_added: added_relationships.len(),
            relationships_removed: removed_relationships.len(),
        };

        Self {
            added_items,
            removed_items,
            modified_items,
            added_relationships,
            removed_relationships,
            stats,
        }
    }

    /// Computes changes between two versions of the same item.
    fn compute_item_changes(old: &Item, new: &Item) -> Vec<FieldChange> {
        let mut changes = Vec::new();

        // Check name change
        if old.name != new.name {
            changes.push(FieldChange {
                field: "name".to_string(),
                old_value: old.name.clone(),
                new_value: new.name.clone(),
            });
        }

        // Check description change
        if old.description != new.description {
            changes.push(FieldChange {
                field: "description".to_string(),
                old_value: old.description.clone().unwrap_or_default(),
                new_value: new.description.clone().unwrap_or_default(),
            });
        }

        // Check specification change (for requirement types)
        if old.attributes.specification() != new.attributes.specification() {
            changes.push(FieldChange {
                field: "specification".to_string(),
                old_value: old.attributes.specification().cloned().unwrap_or_default(),
                new_value: new.attributes.specification().cloned().unwrap_or_default(),
            });
        }

        // Check file path change
        if old.source.file_path != new.source.file_path {
            changes.push(FieldChange {
                field: "file_path".to_string(),
                old_value: old.source.file_path.display().to_string(),
                new_value: new.source.file_path.display().to_string(),
            });
        }

        // Check upstream refs change
        let old_upstream = Self::refs_to_string(old.upstream.all_ids());
        let new_upstream = Self::refs_to_string(new.upstream.all_ids());
        if old_upstream != new_upstream {
            changes.push(FieldChange {
                field: "upstream".to_string(),
                old_value: old_upstream,
                new_value: new_upstream,
            });
        }

        // Check downstream refs change
        let old_downstream = Self::refs_to_string(old.downstream.all_ids());
        let new_downstream = Self::refs_to_string(new.downstream.all_ids());
        if old_downstream != new_downstream {
            changes.push(FieldChange {
                field: "downstream".to_string(),
                old_value: old_downstream,
                new_value: new_downstream,
            });
        }

        changes
    }

    fn refs_to_string<'a>(refs: impl Iterator<Item = &'a ItemId>) -> String {
        let ids: Vec<_> = refs.map(|id| id.as_str()).collect();
        ids.join(", ")
    }

    /// Returns true if there are no changes.
    pub fn is_empty(&self) -> bool {
        self.added_items.is_empty()
            && self.removed_items.is_empty()
            && self.modified_items.is_empty()
            && self.added_relationships.is_empty()
            && self.removed_relationships.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::GraphBuilder;
    use crate::model::{ItemBuilder, ItemType, SourceLocation};
    use std::path::PathBuf;

    fn create_test_item(id: &str, item_type: ItemType, name: &str) -> Item {
        let source = SourceLocation::new(PathBuf::from("/repo"), format!("{}.md", id));
        let mut builder = ItemBuilder::new()
            .id(ItemId::new_unchecked(id))
            .item_type(item_type)
            .name(name)
            .source(source);

        if item_type.requires_specification() {
            builder = builder.specification("Test specification");
        }

        builder.build().unwrap()
    }

    #[test]
    fn test_no_changes() {
        let item = create_test_item("SOL-001", ItemType::Solution, "Solution");

        let old_graph = GraphBuilder::new().add_item(item.clone()).build().unwrap();
        let new_graph = GraphBuilder::new().add_item(item).build().unwrap();

        let diff = GraphDiff::compute(&old_graph, &new_graph);
        assert!(diff.is_empty());
    }

    #[test]
    fn test_added_item() {
        let old_graph = GraphBuilder::new().build().unwrap();
        let new_graph = GraphBuilder::new()
            .add_item(create_test_item("SOL-001", ItemType::Solution, "Solution"))
            .build()
            .unwrap();

        let diff = GraphDiff::compute(&old_graph, &new_graph);
        assert_eq!(diff.stats.items_added, 1);
        assert_eq!(diff.added_items[0].id, "SOL-001");
    }

    #[test]
    fn test_removed_item() {
        let old_graph = GraphBuilder::new()
            .add_item(create_test_item("SOL-001", ItemType::Solution, "Solution"))
            .build()
            .unwrap();
        let new_graph = GraphBuilder::new().build().unwrap();

        let diff = GraphDiff::compute(&old_graph, &new_graph);
        assert_eq!(diff.stats.items_removed, 1);
        assert_eq!(diff.removed_items[0].id, "SOL-001");
    }

    #[test]
    fn test_modified_item() {
        let old_item = create_test_item("SOL-001", ItemType::Solution, "Old Name");
        let new_item = create_test_item("SOL-001", ItemType::Solution, "New Name");

        let old_graph = GraphBuilder::new().add_item(old_item).build().unwrap();
        let new_graph = GraphBuilder::new().add_item(new_item).build().unwrap();

        let diff = GraphDiff::compute(&old_graph, &new_graph);
        assert_eq!(diff.stats.items_modified, 1);
        assert_eq!(diff.modified_items[0].id, "SOL-001");
        assert!(
            diff.modified_items[0]
                .changes
                .iter()
                .any(|c| c.field == "name")
        );
    }
}

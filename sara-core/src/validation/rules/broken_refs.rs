//! Broken reference detection validation rule.

use crate::error::ValidationError;
use crate::graph::KnowledgeGraph;
use crate::model::ItemId;

/// Detects broken references in the knowledge graph.
///
/// A broken reference occurs when an item references another item
/// that does not exist in the graph.
pub fn check_broken_references(graph: &KnowledgeGraph) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    for item in graph.items() {
        // Check all references from this item
        for ref_id in item.all_references() {
            if !graph.contains(ref_id) {
                errors.push(ValidationError::BrokenReference {
                    from: item.id.clone(),
                    to: ref_id.clone(),
                    location: Some(item.source.clone()),
                });
            }
        }
    }

    errors
}

/// Finds items that reference a given ID.
pub fn find_referencing_items(graph: &KnowledgeGraph, target_id: &ItemId) -> Vec<ItemId> {
    graph
        .items()
        .filter(|item| item.all_references().contains(&target_id))
        .map(|item| item.id.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ItemBuilder, ItemType, SourceLocation, UpstreamRefs};
    use std::path::PathBuf;

    fn create_item(
        id: &str,
        item_type: ItemType,
        upstream: Option<UpstreamRefs>,
    ) -> crate::model::Item {
        let source = SourceLocation::new(PathBuf::from("/repo"), format!("{}.md", id));
        let mut builder = ItemBuilder::new()
            .id(ItemId::new_unchecked(id))
            .item_type(item_type)
            .name(format!("Test {}", id))
            .source(source);

        if let Some(up) = upstream {
            builder = builder.upstream(up);
        }

        if item_type.requires_specification() {
            builder = builder.specification("Test spec");
        }

        builder.build().unwrap()
    }

    #[test]
    fn test_no_broken_refs() {
        let mut graph = KnowledgeGraph::new(false);
        graph.add_item(create_item("SOL-001", ItemType::Solution, None));
        graph.add_item(create_item(
            "UC-001",
            ItemType::UseCase,
            Some(UpstreamRefs {
                refines: vec![ItemId::new_unchecked("SOL-001")],
                ..Default::default()
            }),
        ));

        let errors = check_broken_references(&graph);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_broken_ref_detected() {
        let mut graph = KnowledgeGraph::new(false);
        graph.add_item(create_item(
            "UC-001",
            ItemType::UseCase,
            Some(UpstreamRefs {
                refines: vec![ItemId::new_unchecked("SOL-MISSING")],
                ..Default::default()
            }),
        ));

        let errors = check_broken_references(&graph);
        assert_eq!(errors.len(), 1);

        if let ValidationError::BrokenReference { from, to, .. } = &errors[0] {
            assert_eq!(from.as_str(), "UC-001");
            assert_eq!(to.as_str(), "SOL-MISSING");
        } else {
            panic!("Expected BrokenReference error");
        }
    }
}

//! Coverage report generation.

use serde::Serialize;

use crate::graph::KnowledgeGraph;
use crate::model::ItemType;

/// Coverage statistics for a single item type.
#[derive(Debug, Clone, Serialize)]
pub struct TypeCoverage {
    /// The item type.
    pub item_type: ItemType,
    /// Display name for the item type.
    pub type_name: String,
    /// Total number of items of this type.
    pub total: usize,
    /// Number of items with complete traceability.
    pub complete: usize,
    /// Number of items with incomplete traceability.
    pub incomplete: usize,
    /// Coverage percentage (0.0 - 100.0).
    pub coverage_percent: f64,
}

/// An item that is missing upstream or downstream traceability.
#[derive(Debug, Clone, Serialize)]
pub struct IncompleteItem {
    /// Item ID.
    pub id: String,
    /// Item name.
    pub name: String,
    /// Item type.
    pub item_type: String,
    /// Reason for incompleteness.
    pub reason: String,
}

/// Coverage report for the entire graph.
#[derive(Debug, Clone, Serialize)]
pub struct CoverageReport {
    /// Overall coverage percentage.
    pub overall_coverage: f64,
    /// Coverage breakdown by item type.
    pub by_type: Vec<TypeCoverage>,
    /// List of incomplete items.
    pub incomplete_items: Vec<IncompleteItem>,
    /// Total number of items.
    pub total_items: usize,
    /// Number of items with complete traceability.
    pub complete_items: usize,
}

impl CoverageReport {
    /// Generates a coverage report from a knowledge graph.
    pub fn generate(graph: &KnowledgeGraph) -> Self {
        let mut by_type = Vec::new();
        let mut incomplete_items = Vec::new();
        let mut total_items = 0;
        let mut complete_items = 0;

        // Calculate coverage for each item type
        for item_type in ItemType::all() {
            let items = graph.items_by_type(*item_type);
            let total = items.len();

            if total == 0 {
                continue;
            }

            let mut type_complete = 0;
            let mut type_incomplete = 0;

            for item in items {
                let is_complete = Self::check_item_complete(item, graph);

                if is_complete {
                    type_complete += 1;
                } else {
                    type_incomplete += 1;
                    incomplete_items.push(Self::create_incomplete_item(item, graph));
                }
            }

            let coverage_percent = if total > 0 {
                (type_complete as f64 / total as f64) * 100.0
            } else {
                100.0
            };

            by_type.push(TypeCoverage {
                item_type: *item_type,
                type_name: item_type.display_name().to_string(),
                total,
                complete: type_complete,
                incomplete: type_incomplete,
                coverage_percent,
            });

            total_items += total;
            complete_items += type_complete;
        }

        let overall_coverage = if total_items > 0 {
            (complete_items as f64 / total_items as f64) * 100.0
        } else {
            100.0
        };

        Self {
            overall_coverage,
            by_type,
            incomplete_items,
            total_items,
            complete_items,
        }
    }

    /// Checks if an item has complete traceability.
    fn check_item_complete(item: &crate::model::Item, graph: &KnowledgeGraph) -> bool {
        // Solutions are complete if they have downstream items (use graph to find children)
        if item.item_type.is_root() {
            return !graph.children(&item.id).is_empty();
        }

        // All other items are complete if they have upstream items
        !item.upstream.is_empty()
    }

    /// Creates an IncompleteItem from an item.
    fn create_incomplete_item(item: &crate::model::Item, graph: &KnowledgeGraph) -> IncompleteItem {
        let reason = if item.item_type.is_root() && graph.children(&item.id).is_empty() {
            "No downstream items defined".to_string()
        } else if item.upstream.is_empty() {
            format!(
                "Missing parent {}",
                Self::expected_parent_type(item.item_type)
            )
        } else {
            "Incomplete traceability".to_string()
        };

        IncompleteItem {
            id: item.id.as_str().to_string(),
            name: item.name.clone(),
            item_type: item.item_type.display_name().to_string(),
            reason,
        }
    }

    /// Returns the expected parent type for an item type.
    fn expected_parent_type(item_type: ItemType) -> &'static str {
        match item_type.required_parent_type() {
            Some(parent) => parent.display_name(),
            None => "N/A (root)",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::GraphBuilder;
    use crate::model::{Item, ItemBuilder, ItemId, SourceLocation, UpstreamRefs};
    use std::path::PathBuf;

    fn create_test_item(id: &str, item_type: ItemType) -> Item {
        let source = SourceLocation::new(PathBuf::from("/repo"), format!("{}.md", id));
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
        let source = SourceLocation::new(PathBuf::from("/repo"), format!("{}.md", id));
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
    fn test_coverage_report_complete() {
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

        let report = CoverageReport::generate(&graph);
        assert!(report.overall_coverage > 0.0);
    }

    #[test]
    fn test_coverage_report_incomplete() {
        // UseCase without upstream reference
        let uc = create_test_item("UC-001", ItemType::UseCase);

        let graph = GraphBuilder::new().add_item(uc).build().unwrap();

        let report = CoverageReport::generate(&graph);
        assert!(!report.incomplete_items.is_empty());
    }
}

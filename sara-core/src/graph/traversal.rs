//! Graph traversal operations for upstream/downstream queries.

use std::collections::{HashSet, VecDeque};

use petgraph::Direction;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;

use crate::graph::KnowledgeGraph;
use crate::model::{Item, ItemId, ItemType, RelationshipType};

/// Result of a traversal operation.
#[derive(Debug, Clone)]
pub struct TraversalResult {
    /// The starting item.
    pub origin: ItemId,
    /// Items found during traversal, in order visited.
    pub items: Vec<TraversalNode>,
    /// Maximum depth reached.
    pub max_depth: usize,
}

/// A node in the traversal result.
#[derive(Debug, Clone)]
pub struct TraversalNode {
    /// The item at this node.
    pub item_id: ItemId,
    /// Depth from the origin (0 = origin itself).
    pub depth: usize,
    /// Relationship type from parent to this node (None for origin).
    pub relationship: Option<RelationshipType>,
    /// Parent item ID (None for origin).
    pub parent: Option<ItemId>,
}

/// Direction of traversal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraversalDirection {
    /// Traverse toward Solution (following upstream relationships).
    Upstream,
    /// Traverse toward Detailed Designs (following downstream relationships).
    Downstream,
}

/// Options for graph traversal.
#[derive(Debug, Clone, Default)]
pub struct TraversalOptions {
    /// Maximum depth to traverse (None = unlimited).
    pub max_depth: Option<usize>,
    /// Filter results by item types (empty = all types).
    pub type_filter: Vec<ItemType>,
}

impl TraversalOptions {
    /// Creates new traversal options with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the maximum traversal depth.
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }

    /// Sets the type filter.
    pub fn with_types(mut self, types: Vec<ItemType>) -> Self {
        self.type_filter = types;
        self
    }
}

/// Traverses the graph upstream (toward Solution).
///
/// Starting from the given item, follows all upstream relationships
/// (refines, derives_from, satisfies) to find parent items.
pub fn traverse_upstream(
    graph: &KnowledgeGraph,
    start: &ItemId,
    options: &TraversalOptions,
) -> Option<TraversalResult> {
    traverse_graph(graph, start, TraversalDirection::Upstream, options)
}

/// Traverses the graph downstream (toward Detailed Designs).
///
/// Starting from the given item, follows all downstream relationships
/// (is_refined_by, derives, is_satisfied_by) to find child items.
pub fn traverse_downstream(
    graph: &KnowledgeGraph,
    start: &ItemId,
    options: &TraversalOptions,
) -> Option<TraversalResult> {
    traverse_graph(graph, start, TraversalDirection::Downstream, options)
}

/// Internal traversal implementation using BFS.
fn traverse_graph(
    graph: &KnowledgeGraph,
    start: &ItemId,
    direction: TraversalDirection,
    options: &TraversalOptions,
) -> Option<TraversalResult> {
    let start_idx = graph.node_index(start)?;
    let inner = graph.inner();

    let mut visited: HashSet<NodeIndex> = HashSet::new();
    // Queue contains: (node_idx, depth, relationship, parent_for_display)
    // parent_for_display is the last ancestor that was included in results (for proper tree building)
    let mut queue: VecDeque<(NodeIndex, usize, Option<RelationshipType>, Option<ItemId>)> =
        VecDeque::new();
    let mut result_items: Vec<TraversalNode> = Vec::new();
    let mut max_depth = 0;

    queue.push_back((start_idx, 0, None, None));
    visited.insert(start_idx);

    while let Some((node_idx, depth, relationship, display_parent)) = queue.pop_front() {
        if let Some(max) = options.max_depth
            && depth > max
        {
            continue;
        }

        if let Some(item) = inner.node_weight(node_idx) {
            // Determine if this item matches the type filter
            let matches_filter =
                options.type_filter.is_empty() || options.type_filter.contains(&item.item_type);

            // The parent to pass to children: if this item is included, use it;
            // otherwise pass through the current display_parent
            let next_display_parent = if matches_filter {
                Some(item.id.clone())
            } else {
                display_parent.clone()
            };

            if matches_filter {
                result_items.push(TraversalNode {
                    item_id: item.id.clone(),
                    depth,
                    relationship,
                    parent: display_parent,
                });
                max_depth = max_depth.max(depth);
            }

            let edges = match direction {
                TraversalDirection::Upstream => {
                    // Follow outgoing edges with upstream relationship types
                    inner
                        .edges_directed(node_idx, Direction::Outgoing)
                        .filter(|e| e.weight().is_upstream())
                        .map(|e| (e.target(), *e.weight()))
                        .collect::<Vec<_>>()
                }
                TraversalDirection::Downstream => {
                    // Follow incoming edges (items that point to us via upstream relationships)
                    // OR outgoing edges with downstream relationship types
                    let mut edges = Vec::new();

                    // Items that refine/derive from/satisfy this item
                    for edge in inner.edges_directed(node_idx, Direction::Incoming) {
                        if edge.weight().is_upstream() {
                            edges.push((edge.source(), edge.weight().inverse()));
                        }
                    }

                    // Or explicit downstream references from this item
                    for edge in inner.edges_directed(node_idx, Direction::Outgoing) {
                        if edge.weight().is_downstream() {
                            edges.push((edge.target(), *edge.weight()));
                        }
                    }

                    edges
                }
            };

            let next_depth = depth + 1;
            if options.max_depth.is_none_or(|max| next_depth <= max) {
                for (target_idx, rel_type) in edges {
                    if !visited.contains(&target_idx) {
                        visited.insert(target_idx);
                        queue.push_back((
                            target_idx,
                            next_depth,
                            Some(rel_type),
                            next_display_parent.clone(),
                        ));
                    }
                }
            }
        }
    }

    Some(TraversalResult {
        origin: start.clone(),
        items: result_items,
        max_depth,
    })
}

/// Gets the direct upstream items (parents) for an item.
pub fn get_upstream_parents<'a>(graph: &'a KnowledgeGraph, id: &ItemId) -> Vec<&'a Item> {
    graph.parents(id)
}

/// Gets the direct downstream items (children) for an item.
pub fn get_downstream_children<'a>(graph: &'a KnowledgeGraph, id: &ItemId) -> Vec<&'a Item> {
    graph.children(id)
}

/// Builds a tree representation of the traversal for display.
#[derive(Debug, Clone)]
pub struct TraversalTree {
    /// Root item ID.
    pub root: ItemId,
    /// Children of this node.
    pub children: Vec<TraversalTreeNode>,
}

/// A node in the traversal tree.
#[derive(Debug, Clone)]
pub struct TraversalTreeNode {
    /// The item at this node.
    pub item_id: ItemId,
    /// Relationship from parent.
    pub relationship: RelationshipType,
    /// Children of this node.
    pub children: Vec<TraversalTreeNode>,
}

impl TraversalResult {
    /// Converts the traversal result to a tree structure for display.
    pub fn to_tree(&self, _graph: &KnowledgeGraph) -> Option<TraversalTree> {
        if self.items.is_empty() {
            return None;
        }

        // Build parent -> children map
        let mut children_map: std::collections::HashMap<Option<ItemId>, Vec<&TraversalNode>> =
            std::collections::HashMap::new();

        for node in &self.items {
            children_map
                .entry(node.parent.clone())
                .or_default()
                .push(node);
        }

        // Recursively build tree
        fn build_children(
            parent_id: &ItemId,
            children_map: &std::collections::HashMap<Option<ItemId>, Vec<&TraversalNode>>,
        ) -> Vec<TraversalTreeNode> {
            let Some(children) = children_map.get(&Some(parent_id.clone())) else {
                return Vec::new();
            };

            children
                .iter()
                .map(|node| TraversalTreeNode {
                    item_id: node.item_id.clone(),
                    relationship: node.relationship.unwrap_or(RelationshipType::Refines),
                    children: build_children(&node.item_id, children_map),
                })
                .collect()
        }

        let root_children = children_map
            .get(&None)
            .map(|roots| {
                if roots.is_empty() {
                    Vec::new()
                } else {
                    // The first root is the origin, get its children
                    build_children(&roots[0].item_id, &children_map)
                }
            })
            .unwrap_or_default();

        Some(TraversalTree {
            root: self.origin.clone(),
            children: root_children,
        })
    }

    /// Returns only items matching the given types.
    pub fn filter_by_type(
        &self,
        types: &[ItemType],
        graph: &KnowledgeGraph,
    ) -> Vec<&TraversalNode> {
        self.items
            .iter()
            .filter(|node| {
                graph
                    .get(&node.item_id)
                    .is_some_and(|item| types.contains(&item.item_type))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::GraphBuilder;
    use crate::model::RelationshipType;
    use crate::test_utils::{create_test_item, create_test_item_with_relationships};

    #[test]
    fn test_upstream_traversal() {
        // Build a simple hierarchy: SOL-001 <- UC-001 <- SCEN-001
        let sol = create_test_item("SOL-001", ItemType::Solution);
        let uc = create_test_item_with_relationships(
            "UC-001",
            ItemType::UseCase,
            vec![(ItemId::new_unchecked("SOL-001"), RelationshipType::Refines)],
        );
        let scen = create_test_item_with_relationships(
            "SCEN-001",
            ItemType::Scenario,
            vec![(ItemId::new_unchecked("UC-001"), RelationshipType::Refines)],
        );

        let graph = GraphBuilder::new()
            .add_item(sol)
            .add_item(uc)
            .add_item(scen)
            .build()
            .unwrap();

        let result = traverse_upstream(
            &graph,
            &ItemId::new_unchecked("SCEN-001"),
            &TraversalOptions::new(),
        );

        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.items.len(), 3); // SCEN-001, UC-001, SOL-001
        assert_eq!(result.max_depth, 2);
    }

    #[test]
    fn test_downstream_traversal() {
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

        let result = traverse_downstream(
            &graph,
            &ItemId::new_unchecked("SOL-001"),
            &TraversalOptions::new(),
        );

        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.items.len(), 2); // SOL-001, UC-001
    }

    #[test]
    fn test_depth_limited_traversal() {
        let sol = create_test_item("SOL-001", ItemType::Solution);
        let uc = create_test_item_with_relationships(
            "UC-001",
            ItemType::UseCase,
            vec![(ItemId::new_unchecked("SOL-001"), RelationshipType::Refines)],
        );
        let scen = create_test_item_with_relationships(
            "SCEN-001",
            ItemType::Scenario,
            vec![(ItemId::new_unchecked("UC-001"), RelationshipType::Refines)],
        );

        let graph = GraphBuilder::new()
            .add_item(sol)
            .add_item(uc)
            .add_item(scen)
            .build()
            .unwrap();

        let result = traverse_upstream(
            &graph,
            &ItemId::new_unchecked("SCEN-001"),
            &TraversalOptions::new().with_max_depth(1),
        );

        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.max_depth, 1);
        // Should find SCEN-001 (depth 0) and UC-001 (depth 1), but not SOL-001 (depth 2)
        assert!(result.items.len() <= 2);
    }

    #[test]
    fn test_type_filtered_traversal() {
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

        let result = traverse_upstream(
            &graph,
            &ItemId::new_unchecked("UC-001"),
            &TraversalOptions::new().with_types(vec![ItemType::Solution]),
        );

        assert!(result.is_some());
        let result = result.unwrap();
        // Filter should only include Solution type
        let filtered = result.filter_by_type(&[ItemType::Solution], &graph);
        assert_eq!(filtered.len(), 1);
    }
}

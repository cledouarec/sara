//! Graph traversal operations for upstream/downstream queries.

use std::collections::HashSet;

use petgraph::Direction;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;

use crate::graph::KnowledgeGraph;
use crate::model::{Item, ItemId, ItemType, RelationshipType};

/// Result of a traversal operation.
#[derive(Debug, Clone)]
pub struct TraversalResult {
    /// The starting item.
    pub origin: ItemId,
    /// Items found during traversal, in depth-first preorder.
    ///
    /// An item reachable through several paths appears once per path, so
    /// the list unfolds the traversed subgraph into a tree.
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
    ///
    /// In the preorder of [`TraversalResult::items`], the parent occurrence
    /// is the nearest preceding node carrying this id.
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

/// Internal traversal implementation: a depth-first walk that reports one
/// occurrence per path.
///
/// Only the ancestors on the current path are excluded from expansion, so an
/// item reachable through several parents is reported under each of them
/// while cycles still terminate.
fn traverse_graph(
    graph: &KnowledgeGraph,
    start: &ItemId,
    direction: TraversalDirection,
    options: &TraversalOptions,
) -> Option<TraversalResult> {
    let start_idx = graph.node_index(start)?;

    let mut walk = Walk {
        inner: graph.inner(),
        direction,
        options,
        on_path: HashSet::new(),
        items: Vec::new(),
    };
    walk.visit(start_idx, 0, None, None);

    let max_depth = walk.items.iter().map(|node| node.depth).max().unwrap_or(0);

    Some(TraversalResult {
        origin: start.clone(),
        items: walk.items,
        max_depth,
    })
}

/// State of a depth-first walk.
struct Walk<'a> {
    inner: &'a DiGraph<Item, RelationshipType>,
    direction: TraversalDirection,
    options: &'a TraversalOptions,
    /// Nodes on the path from the origin to the node being visited.
    on_path: HashSet<NodeIndex>,
    items: Vec<TraversalNode>,
}

impl<'a> Walk<'a> {
    /// Reports the node when it passes the type filter, then visits its
    /// neighbors in the walk direction.
    ///
    /// `display_parent` is the last ancestor that was reported, so filtered
    /// out items are skipped over rather than breaking the tree.
    fn visit(
        &mut self,
        node_idx: NodeIndex,
        depth: usize,
        relationship: Option<RelationshipType>,
        display_parent: Option<&'a ItemId>,
    ) {
        let inner = self.inner;
        let Some(item) = inner.node_weight(node_idx) else {
            return;
        };

        let matches_filter = self.options.type_filter.is_empty()
            || self.options.type_filter.contains(&item.item_type);

        let next_display_parent = if matches_filter {
            self.items.push(TraversalNode {
                item_id: item.id.clone(),
                depth,
                relationship,
                parent: display_parent.cloned(),
            });
            Some(&item.id)
        } else {
            display_parent
        };

        let next_depth = depth + 1;
        if self.options.max_depth.is_some_and(|max| next_depth > max) {
            return;
        }

        self.on_path.insert(node_idx);
        for (target_idx, rel_type) in self.neighbors(node_idx) {
            if !self.on_path.contains(&target_idx) {
                self.visit(target_idx, next_depth, Some(rel_type), next_display_parent);
            }
        }
        self.on_path.remove(&node_idx);
    }

    /// Returns the neighbors reached from a node in the walk direction, with
    /// the relationship seen from that node.
    fn neighbors(&self, node_idx: NodeIndex) -> Vec<(NodeIndex, RelationshipType)> {
        match self.direction {
            TraversalDirection::Upstream => {
                // Follow outgoing edges with upstream relationship types
                self.inner
                    .edges_directed(node_idx, Direction::Outgoing)
                    .filter(|e| e.weight().is_upstream())
                    .map(|e| (e.target(), *e.weight()))
                    .collect()
            }
            TraversalDirection::Downstream => {
                // Follow incoming edges (items that point to us via upstream relationships)
                // OR outgoing edges with downstream relationship types
                let mut edges = Vec::new();

                // Items that refine/derive from/satisfy this item
                for edge in self.inner.edges_directed(node_idx, Direction::Incoming) {
                    if edge.weight().is_upstream() {
                        edges.push((edge.source(), edge.weight().inverse()));
                    }
                }

                // Or explicit downstream references from this item
                for edge in self.inner.edges_directed(node_idx, Direction::Outgoing) {
                    if edge.weight().is_downstream() {
                        edges.push((edge.target(), *edge.weight()));
                    }
                }

                edges
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::graph::KnowledgeGraphBuilder;
    use crate::model::Relationship;
    use crate::schema::builtin;
    use crate::test_utils::{create_test_item, create_test_item_with_relationships};

    #[test]
    fn test_upstream_traversal() {
        // Build a simple hierarchy: SOL-001 <- UC-001 <- SCEN-001
        let sol = create_test_item("SOL-001", builtin::SOLUTION);
        let uc = create_test_item_with_relationships(
            "UC-001",
            builtin::USE_CASE,
            vec![Relationship::new(
                ItemId::new_unchecked("SOL-001"),
                builtin::REFINES,
            )],
        );
        let scen = create_test_item_with_relationships(
            "SCEN-001",
            builtin::SCENARIO,
            vec![Relationship::new(
                ItemId::new_unchecked("UC-001"),
                builtin::REFINES,
            )],
        );

        let graph = KnowledgeGraphBuilder::new()
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
        let sol = create_test_item("SOL-001", builtin::SOLUTION);
        let uc = create_test_item_with_relationships(
            "UC-001",
            builtin::USE_CASE,
            vec![Relationship::new(
                ItemId::new_unchecked("SOL-001"),
                builtin::REFINES,
            )],
        );

        let graph = KnowledgeGraphBuilder::new()
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
        let sol = create_test_item("SOL-001", builtin::SOLUTION);
        let uc = create_test_item_with_relationships(
            "UC-001",
            builtin::USE_CASE,
            vec![Relationship::new(
                ItemId::new_unchecked("SOL-001"),
                builtin::REFINES,
            )],
        );
        let scen = create_test_item_with_relationships(
            "SCEN-001",
            builtin::SCENARIO,
            vec![Relationship::new(
                ItemId::new_unchecked("UC-001"),
                builtin::REFINES,
            )],
        );

        let graph = KnowledgeGraphBuilder::new()
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
    fn test_shared_ancestor_is_reported_under_each_parent() {
        // SYSARCH-001 satisfies two requirements that both derive from
        // SCEN-001, which refines UC-001, which refines SOL-001.
        let scen = create_test_item("SCEN-001", builtin::SCENARIO);
        let req1 = create_test_item_with_relationships(
            "SYSREQ-001",
            builtin::SYSTEM_REQUIREMENT,
            vec![Relationship::new(
                ItemId::new_unchecked("SCEN-001"),
                builtin::DERIVES_FROM,
            )],
        );
        let req2 = create_test_item_with_relationships(
            "SYSREQ-002",
            builtin::SYSTEM_REQUIREMENT,
            vec![Relationship::new(
                ItemId::new_unchecked("SCEN-001"),
                builtin::DERIVES_FROM,
            )],
        );
        let arch = create_test_item_with_relationships(
            "SYSARCH-001",
            builtin::SYSTEM_ARCHITECTURE,
            vec![
                Relationship::new(ItemId::new_unchecked("SYSREQ-001"), builtin::SATISFIES),
                Relationship::new(ItemId::new_unchecked("SYSREQ-002"), builtin::SATISFIES),
            ],
        );

        let graph = KnowledgeGraphBuilder::new()
            .add_item(scen)
            .add_item(req1)
            .add_item(req2)
            .add_item(arch)
            .build()
            .unwrap();

        let result = traverse_upstream(
            &graph,
            &ItemId::new_unchecked("SYSARCH-001"),
            &TraversalOptions::new(),
        )
        .unwrap();

        // Depth-first preorder: the origin, then each requirement immediately
        // followed by its own occurrence of the shared scenario.
        let visited: Vec<(&str, usize, Option<&str>)> = result
            .items
            .iter()
            .map(|n| {
                (
                    n.item_id.as_str(),
                    n.depth,
                    n.parent.as_ref().map(ItemId::as_str),
                )
            })
            .collect();
        assert_eq!(visited.len(), 5, "{visited:?}");
        assert_eq!(visited[0], ("SYSARCH-001", 0, None));
        for block in [1, 3] {
            let (req, depth, parent) = visited[block];
            assert!(req.starts_with("SYSREQ-"), "{visited:?}");
            assert_eq!((depth, parent), (1, Some("SYSARCH-001")));
            assert_eq!(visited[block + 1], ("SCEN-001", 2, Some(req)));
        }
        assert_ne!(visited[1].0, visited[3].0);
        assert_eq!(result.max_depth, 2);
    }

    #[test]
    fn test_cycle_terminates_without_revisiting_path_ancestors() {
        let a = create_test_item_with_relationships(
            "UC-001",
            builtin::USE_CASE,
            vec![Relationship::new(
                ItemId::new_unchecked("UC-002"),
                builtin::REFINES,
            )],
        );
        let b = create_test_item_with_relationships(
            "UC-002",
            builtin::USE_CASE,
            vec![Relationship::new(
                ItemId::new_unchecked("UC-001"),
                builtin::REFINES,
            )],
        );

        let graph = KnowledgeGraphBuilder::new()
            .add_item(a)
            .add_item(b)
            .build()
            .unwrap();

        let result = traverse_upstream(
            &graph,
            &ItemId::new_unchecked("UC-001"),
            &TraversalOptions::new(),
        )
        .unwrap();

        let ids: Vec<&str> = result.items.iter().map(|n| n.item_id.as_str()).collect();
        assert_eq!(ids, ["UC-001", "UC-002"]);
    }

    #[test]
    fn test_type_filtered_traversal() {
        let sol = create_test_item("SOL-001", builtin::SOLUTION);
        let uc = create_test_item_with_relationships(
            "UC-001",
            builtin::USE_CASE,
            vec![Relationship::new(
                ItemId::new_unchecked("SOL-001"),
                builtin::REFINES,
            )],
        );

        let graph = KnowledgeGraphBuilder::new()
            .add_item(sol)
            .add_item(uc)
            .build()
            .unwrap();

        let result = traverse_upstream(
            &graph,
            &ItemId::new_unchecked("UC-001"),
            &TraversalOptions::new().with_types(vec![builtin::SOLUTION]),
        );

        assert!(result.is_some());
        let result = result.unwrap();
        // Only Solution items should be included
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].item_id, ItemId::new_unchecked("SOL-001"));
    }
}

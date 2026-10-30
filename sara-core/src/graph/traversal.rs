//! Graph traversal operations for upstream/downstream queries.

use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet, VecDeque};

use petgraph::Direction;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use serde::Serialize;

use crate::graph::KnowledgeGraph;
use crate::model::{Item, ItemId, ItemSummary, ItemType, RelationshipType};

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

    /// Whether an item passes the type filter; an empty filter accepts
    /// every type.
    #[must_use]
    pub fn accepts(&self, item: &Item) -> bool {
        self.type_filter.is_empty() || self.type_filter.contains(&item.item_type)
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

        let matches_filter = self.options.accepts(item);

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
        for (target_idx, rel_type) in neighbors(inner, node_idx, self.direction) {
            if !self.on_path.contains(&target_idx) {
                self.visit(target_idx, next_depth, Some(rel_type), next_display_parent);
            }
        }
        self.on_path.remove(&node_idx);
    }
}

/// Returns the neighbors reached from a node in the given direction, with
/// the relationship seen from that node.
///
/// A link declared from both sides, or from the side that also gets a
/// materialized inverse edge, reaches the same neighbor through several
/// edges; each neighbor is returned once, with its first relationship.
fn neighbors(
    inner: &DiGraph<Item, RelationshipType>,
    node_idx: NodeIndex,
    direction: TraversalDirection,
) -> Vec<(NodeIndex, RelationshipType)> {
    let mut edges = Vec::new();
    match direction {
        TraversalDirection::Upstream => {
            // Follow outgoing edges with upstream relationship types
            for edge in inner.edges_directed(node_idx, Direction::Outgoing) {
                if edge.weight().is_upstream() {
                    edges.push((edge.target(), *edge.weight()));
                }
            }
        }
        TraversalDirection::Downstream => {
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
        }
    }

    first_per_target(edges)
}

/// Returns the items that depend on a node: its downstream neighbors, then
/// the items it is a peer prerequisite of, each with the relationship seen
/// from the node.
///
/// A peer dependency may be written on either side: the dependent item
/// declares a primary relation toward the node (`depends_on`, `supersedes`),
/// or the node declares the inverse toward the dependent (`is_required_by`,
/// `superseded_by`).
fn dependents(
    inner: &DiGraph<Item, RelationshipType>,
    node_idx: NodeIndex,
) -> Vec<(NodeIndex, RelationshipType)> {
    let mut edges = neighbors(inner, node_idx, TraversalDirection::Downstream);
    for edge in inner.edges_directed(node_idx, Direction::Incoming) {
        let relationship = *edge.weight();
        if relationship.is_peer() && relationship.is_primary() {
            edges.push((edge.source(), relationship.inverse()));
        }
    }
    for edge in inner.edges_directed(node_idx, Direction::Outgoing) {
        let relationship = *edge.weight();
        if relationship.is_peer() && !relationship.is_primary() {
            edges.push((edge.target(), relationship));
        }
    }

    first_per_target(edges)
}

/// Keeps the first edge leading to each target node.
fn first_per_target(
    mut edges: Vec<(NodeIndex, RelationshipType)>,
) -> Vec<(NodeIndex, RelationshipType)> {
    let mut seen = HashSet::new();
    edges.retain(|(target_idx, _)| seen.insert(*target_idx));
    edges
}

/// Items reached by a breadth-first traversal, each reported once.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct Reach {
    /// Reached items ordered by depth, then by id, each at its shortest
    /// distance from the origins. The origins themselves are not included.
    pub items: Vec<ReachedItem>,
    /// Links followed during the walk, each once, ordered by the depth of
    /// the item they start from, then by source and target id.
    pub links: Vec<ReachLink>,
    /// Whether the depth limit or the item limit left reachable items out.
    pub truncated: bool,
}

impl Reach {
    /// Keeps the `limit` nearest items and the links between them and the
    /// origins; marks the reach truncated when it drops items.
    pub fn truncate(&mut self, limit: usize) {
        if self.items.len() <= limit {
            return;
        }
        let dropped: HashSet<ItemId> = self
            .items
            .drain(limit..)
            .map(|reached| reached.item.id)
            .collect();
        self.links
            .retain(|link| !dropped.contains(&link.from) && !dropped.contains(&link.to));
        self.truncated = true;
    }
}

/// An item reached by a breadth-first traversal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct ReachedItem {
    /// The reached item.
    #[serde(flatten)]
    pub item: ItemSummary,
    /// Number of links on the shortest path from the origins (at least 1).
    pub depth: usize,
}

/// A link followed by a breadth-first traversal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct ReachLink {
    /// Item the link was followed from.
    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    pub from: ItemId,
    /// The relationship as seen from `from`.
    #[serde(rename = "relation")]
    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    pub relationship: RelationshipType,
    /// Item the link leads to.
    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    pub to: ItemId,
}

/// Collects the items reachable from `origins` in `direction`, each once.
///
/// [`traverse_upstream`] and [`traverse_downstream`] unfold every path into
/// a tree, so an item reachable through several paths appears several
/// times. This walk visits each item once, at its shortest distance, and
/// reports every link it followed: shared ancestors and descendants appear a
/// single time with all their links. Neighbors follow the same rules as
/// those traversals. Origins absent from the graph are ignored, and
/// `max_depth` bounds the distance of the reported items.
#[must_use]
pub fn reach(
    graph: &KnowledgeGraph,
    origins: &[ItemId],
    direction: TraversalDirection,
    max_depth: Option<usize>,
) -> Reach {
    let inner = graph.inner();
    breadth_first(graph, origins, max_depth, |node_idx| {
        neighbors(inner, node_idx, direction)
    })
}

/// Collects the items a change to `origins` may impact, each once.
///
/// An item depends on another when it traces down from it, as in a
/// downstream [`reach`], or when it declares a primary peer relation toward
/// it, such as `depends_on` or `supersedes`. The walk repeats from every item
/// found, so the result holds everything that directly or transitively
/// depends on the origins; items the origins themselves depend on are not
/// included. Origins absent from the graph are ignored, and `max_depth`
/// bounds the distance of the reported items.
#[must_use]
pub fn impact(graph: &KnowledgeGraph, origins: &[ItemId], max_depth: Option<usize>) -> Reach {
    let inner = graph.inner();
    breadth_first(graph, origins, max_depth, |node_idx| {
        dependents(inner, node_idx)
    })
}

/// Breadth-first walk from `origins`, expanding each node through `next`.
fn breadth_first(
    graph: &KnowledgeGraph,
    origins: &[ItemId],
    max_depth: Option<usize>,
    next: impl Fn(NodeIndex) -> Vec<(NodeIndex, RelationshipType)>,
) -> Reach {
    let mut depths: HashMap<NodeIndex, usize> = HashMap::new();
    let mut queue = VecDeque::new();
    for origin in origins {
        if let Some(node_idx) = graph.node_index(origin)
            && let Entry::Vacant(entry) = depths.entry(node_idx)
        {
            entry.insert(0);
            queue.push_back(node_idx);
        }
    }

    let mut followed = Vec::new();
    let mut truncated = false;
    while let Some(node_idx) = queue.pop_front() {
        let depth = depths[&node_idx];
        let targets = next(node_idx);
        if max_depth.is_some_and(|max| depth >= max) {
            truncated |= !targets.is_empty();
            continue;
        }

        for (target_idx, relationship) in targets {
            followed.push((node_idx, relationship, target_idx));
            if let Entry::Vacant(entry) = depths.entry(target_idx) {
                entry.insert(depth + 1);
                queue.push_back(target_idx);
            }
        }
    }

    let inner = graph.inner();
    let id_of = |node_idx: NodeIndex| inner[node_idx].id.clone();

    // Every node but the origins, at depth 0, was reached.
    let mut items: Vec<ReachedItem> = depths
        .iter()
        .filter(|&(_, &depth)| depth > 0)
        .map(|(&node_idx, &depth)| ReachedItem {
            item: ItemSummary::from(&inner[node_idx]),
            depth,
        })
        .collect();
    items.sort_by(|a, b| (a.depth, a.item.id.as_str()).cmp(&(b.depth, b.item.id.as_str())));

    followed.sort_by_key(|(from, _, to)| {
        (
            depths[from],
            inner[*from].id.as_str(),
            inner[*to].id.as_str(),
        )
    });
    let links = followed
        .into_iter()
        .map(|(from, relationship, to)| ReachLink {
            from: id_of(from),
            relationship,
            to: id_of(to),
        })
        .collect();

    Reach {
        items,
        links,
        truncated,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::graph::KnowledgeGraphBuilder;
    use crate::model::Relationship;
    use crate::schema::builtin;
    use crate::test_utils::{
        create_test_adr, create_test_item, create_test_item_with_relationships,
    };

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

    /// Returns the ids of the reported items, in report order.
    fn reported_ids(result: &TraversalResult) -> Vec<&str> {
        result
            .items
            .iter()
            .map(|node| node.item_id.as_str())
            .collect()
    }

    /// Builds the graph where SOL-001 and UC-001 both declare their link.
    fn link_declared_on_both_sides() -> KnowledgeGraph {
        let sol = create_test_item_with_relationships(
            "SOL-001",
            builtin::SOLUTION,
            vec![Relationship::new(
                ItemId::new_unchecked("UC-001"),
                builtin::IS_REFINED_BY,
            )],
        );
        let uc = create_test_item_with_relationships(
            "UC-001",
            builtin::USE_CASE,
            vec![Relationship::new(
                ItemId::new_unchecked("SOL-001"),
                builtin::REFINES,
            )],
        );

        KnowledgeGraphBuilder::new()
            .add_item(sol)
            .add_item(uc)
            .build()
            .unwrap()
    }

    #[test]
    fn test_child_declared_downstream_is_reported_once() {
        // The parent declares the link, so the builder also adds the
        // inverse `refines` edge: the child is reachable through two edges.
        let sol = create_test_item_with_relationships(
            "SOL-001",
            builtin::SOLUTION,
            vec![Relationship::new(
                ItemId::new_unchecked("UC-001"),
                builtin::IS_REFINED_BY,
            )],
        );
        let uc = create_test_item("UC-001", builtin::USE_CASE);

        let graph = KnowledgeGraphBuilder::new()
            .add_item(sol)
            .add_item(uc)
            .build()
            .unwrap();

        let result = traverse_downstream(
            &graph,
            &ItemId::new_unchecked("SOL-001"),
            &TraversalOptions::new(),
        )
        .unwrap();

        assert_eq!(reported_ids(&result), ["SOL-001", "UC-001"]);
    }

    #[test]
    fn test_justifying_adr_is_reported_once_downstream() {
        // An ADR is a detached type: its upstream `justifies` link also gets
        // an inverse edge, so the ADR is reachable through two edges.
        let arch = create_test_item("SYSARCH-001", builtin::SYSTEM_ARCHITECTURE);
        let adr = create_test_adr("ADR-001", &["SYSARCH-001"], &[]);

        let graph = KnowledgeGraphBuilder::new()
            .add_item(arch)
            .add_item(adr)
            .build()
            .unwrap();

        let result = traverse_downstream(
            &graph,
            &ItemId::new_unchecked("SYSARCH-001"),
            &TraversalOptions::new(),
        )
        .unwrap();

        assert_eq!(reported_ids(&result), ["SYSARCH-001", "ADR-001"]);
    }

    #[test]
    fn test_link_declared_on_both_sides_is_reported_once_upstream() {
        let graph = link_declared_on_both_sides();

        let result = traverse_upstream(
            &graph,
            &ItemId::new_unchecked("UC-001"),
            &TraversalOptions::new(),
        )
        .unwrap();

        assert_eq!(reported_ids(&result), ["UC-001", "SOL-001"]);
    }

    #[test]
    fn test_link_declared_on_both_sides_is_reported_once_downstream() {
        let graph = link_declared_on_both_sides();

        let result = traverse_downstream(
            &graph,
            &ItemId::new_unchecked("SOL-001"),
            &TraversalOptions::new(),
        )
        .unwrap();

        assert_eq!(reported_ids(&result), ["SOL-001", "UC-001"]);
    }

    /// Returns `(id, depth)` pairs of the reached items, in report order.
    fn reached(reach: &Reach) -> Vec<(&str, usize)> {
        reach
            .items
            .iter()
            .map(|reached| (reached.item.id.as_str(), reached.depth))
            .collect()
    }

    /// Returns `from -> to` pairs of the followed links, in report order.
    fn followed(reach: &Reach) -> Vec<(&str, &str)> {
        reach
            .links
            .iter()
            .map(|link| (link.from.as_str(), link.to.as_str()))
            .collect()
    }

    /// Builds a diamond: SCEN-001 refines UC-001 and UC-002, which both
    /// refine SOL-001.
    fn diamond() -> KnowledgeGraph {
        let refines = |id: &str| {
            vec![Relationship::new(
                ItemId::new_unchecked(id),
                builtin::REFINES,
            )]
        };
        let mut scen_refines = refines("UC-001");
        scen_refines.extend(refines("UC-002"));

        KnowledgeGraphBuilder::new()
            .add_item(create_test_item("SOL-001", builtin::SOLUTION))
            .add_item(create_test_item_with_relationships(
                "UC-001",
                builtin::USE_CASE,
                refines("SOL-001"),
            ))
            .add_item(create_test_item_with_relationships(
                "UC-002",
                builtin::USE_CASE,
                refines("SOL-001"),
            ))
            .add_item(create_test_item_with_relationships(
                "SCEN-001",
                builtin::SCENARIO,
                scen_refines,
            ))
            .build()
            .unwrap()
    }

    #[test]
    fn test_reach_downstream_reports_each_item_once() {
        let graph = diamond();

        let reach = reach(
            &graph,
            &[ItemId::new_unchecked("SOL-001")],
            TraversalDirection::Downstream,
            None,
        );

        assert_eq!(
            reached(&reach),
            [("UC-001", 1), ("UC-002", 1), ("SCEN-001", 2)]
        );
        assert_eq!(
            followed(&reach),
            [
                ("SOL-001", "UC-001"),
                ("SOL-001", "UC-002"),
                ("UC-001", "SCEN-001"),
                ("UC-002", "SCEN-001"),
            ]
        );
        assert!(
            reach
                .links
                .iter()
                .all(|link| link.relationship == builtin::IS_REFINED_BY)
        );
        assert!(!reach.truncated);
    }

    #[test]
    fn test_reach_upstream_reports_each_item_once() {
        let graph = diamond();

        let reach = reach(
            &graph,
            &[ItemId::new_unchecked("SCEN-001")],
            TraversalDirection::Upstream,
            None,
        );

        assert_eq!(
            reached(&reach),
            [("UC-001", 1), ("UC-002", 1), ("SOL-001", 2)]
        );
        assert_eq!(
            followed(&reach),
            [
                ("SCEN-001", "UC-001"),
                ("SCEN-001", "UC-002"),
                ("UC-001", "SOL-001"),
                ("UC-002", "SOL-001"),
            ]
        );
    }

    #[test]
    fn test_reach_stops_at_max_depth() {
        let graph = diamond();

        let reach = reach(
            &graph,
            &[ItemId::new_unchecked("SOL-001")],
            TraversalDirection::Downstream,
            Some(1),
        );

        assert_eq!(reached(&reach), [("UC-001", 1), ("UC-002", 1)]);
        assert_eq!(
            followed(&reach),
            [("SOL-001", "UC-001"), ("SOL-001", "UC-002")]
        );
        assert!(reach.truncated);
    }

    #[test]
    fn test_reach_truncate_keeps_the_nearest_items_and_their_links() {
        let graph = diamond();
        let mut reach = reach(
            &graph,
            &[ItemId::new_unchecked("SOL-001")],
            TraversalDirection::Downstream,
            None,
        );

        reach.truncate(2);

        assert_eq!(reached(&reach), [("UC-001", 1), ("UC-002", 1)]);
        assert_eq!(
            followed(&reach),
            [("SOL-001", "UC-001"), ("SOL-001", "UC-002")]
        );
        assert!(reach.truncated);
    }

    #[test]
    fn test_reach_serializes_items_links_and_truncation() {
        let graph = diamond();
        let reach = reach(
            &graph,
            &[ItemId::new_unchecked("SOL-001")],
            TraversalDirection::Downstream,
            Some(1),
        );

        let value = serde_yaml::to_value(&reach).unwrap();

        assert_eq!(value["items"][0]["id"], "UC-001");
        assert_eq!(value["items"][0]["type"], "use_case");
        assert_eq!(value["items"][0]["depth"], 1);
        assert_eq!(value["links"][0]["from"], "SOL-001");
        assert_eq!(value["links"][0]["relation"], "is_refined_by");
        assert_eq!(value["truncated"], true);
    }

    #[test]
    fn test_reach_truncate_within_the_limit_changes_nothing() {
        let graph = diamond();
        let full = reach(
            &graph,
            &[ItemId::new_unchecked("SOL-001")],
            TraversalDirection::Downstream,
            None,
        );
        let mut truncated = full.clone();

        truncated.truncate(3);

        assert_eq!(truncated, full);
    }

    #[test]
    fn test_reach_merges_origins_and_excludes_them() {
        let graph = diamond();

        let reach = reach(
            &graph,
            &[
                ItemId::new_unchecked("UC-001"),
                ItemId::new_unchecked("UC-002"),
            ],
            TraversalDirection::Upstream,
            None,
        );

        assert_eq!(reached(&reach), [("SOL-001", 1)]);
        assert_eq!(
            followed(&reach),
            [("UC-001", "SOL-001"), ("UC-002", "SOL-001")]
        );
    }

    /// Returns a system requirement declaring `depends_on` toward `targets`.
    fn requirement(id: &str, depends_on: &[&str]) -> Item {
        create_test_item_with_relationships(
            id,
            builtin::SYSTEM_REQUIREMENT,
            depends_on
                .iter()
                .map(|target| {
                    Relationship::new(ItemId::new_unchecked(*target), builtin::DEPENDS_ON)
                })
                .collect(),
        )
    }

    #[test]
    fn test_impact_follows_downstream_links_and_dependent_peers() {
        let graph = KnowledgeGraphBuilder::new()
            .add_item(requirement("SYSREQ-001", &[]))
            .add_item(requirement("SYSREQ-002", &["SYSREQ-001"]))
            .add_item(requirement("SYSREQ-003", &["SYSREQ-002"]))
            .add_item(create_test_item_with_relationships(
                "SYSARCH-001",
                builtin::SYSTEM_ARCHITECTURE,
                vec![Relationship::new(
                    ItemId::new_unchecked("SYSREQ-001"),
                    builtin::SATISFIES,
                )],
            ))
            .build()
            .unwrap();

        let impact = impact(&graph, &[ItemId::new_unchecked("SYSREQ-001")], None);

        assert_eq!(
            reached(&impact),
            [("SYSARCH-001", 1), ("SYSREQ-002", 1), ("SYSREQ-003", 2)]
        );
        let relationships: Vec<RelationshipType> =
            impact.links.iter().map(|link| link.relationship).collect();
        assert_eq!(
            relationships,
            [
                builtin::IS_SATISFIED_BY,
                builtin::IS_REQUIRED_BY,
                builtin::IS_REQUIRED_BY,
            ]
        );
    }

    #[test]
    fn test_impact_follows_dependencies_declared_from_the_other_side() {
        // SYSREQ-001 declares that SYSREQ-002 requires it, so SYSREQ-002
        // depends on it although SYSREQ-002 declares nothing.
        let graph = KnowledgeGraphBuilder::new()
            .add_item(create_test_item_with_relationships(
                "SYSREQ-001",
                builtin::SYSTEM_REQUIREMENT,
                vec![Relationship::new(
                    ItemId::new_unchecked("SYSREQ-002"),
                    builtin::IS_REQUIRED_BY,
                )],
            ))
            .add_item(requirement("SYSREQ-002", &[]))
            .build()
            .unwrap();

        let impact = impact(&graph, &[ItemId::new_unchecked("SYSREQ-001")], None);

        assert_eq!(reached(&impact), [("SYSREQ-002", 1)]);
        assert_eq!(impact.links[0].relationship, builtin::IS_REQUIRED_BY);
    }

    #[test]
    fn test_impact_ignores_items_the_origin_depends_on() {
        let graph = KnowledgeGraphBuilder::new()
            .add_item(requirement("SYSREQ-001", &["SYSREQ-002"]))
            .add_item(requirement("SYSREQ-002", &[]))
            .build()
            .unwrap();

        let impact = impact(&graph, &[ItemId::new_unchecked("SYSREQ-001")], None);

        assert!(impact.items.is_empty());
    }

    #[test]
    fn test_reach_link_serializes_its_relationship_as_relation() {
        let link = ReachLink {
            from: ItemId::new_unchecked("SYSREQ-001"),
            relationship: builtin::IS_SATISFIED_BY,
            to: ItemId::new_unchecked("SYSARCH-001"),
        };

        assert_eq!(
            serde_yaml::to_string(&link).unwrap(),
            "from: SYSREQ-001\nrelation: is_satisfied_by\nto: SYSARCH-001\n"
        );
    }
}

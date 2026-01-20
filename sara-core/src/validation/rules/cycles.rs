//! Circular reference detection validation rule.

use petgraph::algo::tarjan_scc;

use crate::error::ValidationError;
use crate::graph::KnowledgeGraph;
use crate::model::ItemId;

/// Checks if two nodes form only an inverse relationship pair (not a true cycle).
///
/// Inverse pairs are bidirectional edges like:
/// - A --Satisfies--> B and B --IsSatisfiedBy--> A
/// - A --Refines--> B and B --IsRefinedBy--> A
/// - A --DerivesFrom--> B and B --Derives--> A
///
/// These represent the same logical relationship from two perspectives and
/// should not be considered cycles.
fn is_inverse_pair_only(
    graph: &KnowledgeGraph,
    idx_a: petgraph::graph::NodeIndex,
    idx_b: petgraph::graph::NodeIndex,
) -> bool {
    let inner = graph.inner();

    // Get edges from A to B and from B to A
    let edges_a_to_b: Vec<_> = inner
        .edges_connecting(idx_a, idx_b)
        .map(|e| *e.weight())
        .collect();
    let edges_b_to_a: Vec<_> = inner
        .edges_connecting(idx_b, idx_a)
        .map(|e| *e.weight())
        .collect();

    // If there are no edges in either direction, not an inverse pair
    if edges_a_to_b.is_empty() || edges_b_to_a.is_empty() {
        return false;
    }

    // Check if all edges form inverse pairs
    for rel_a_to_b in &edges_a_to_b {
        let inverse = rel_a_to_b.inverse();
        if !edges_b_to_a.contains(&inverse) {
            // Found an edge that doesn't have its inverse - this is a real cycle component
            return false;
        }
    }

    for rel_b_to_a in &edges_b_to_a {
        let inverse = rel_b_to_a.inverse();
        if !edges_a_to_b.contains(&inverse) {
            return false;
        }
    }

    true
}

/// Detects circular references in the knowledge graph.
///
/// Uses Tarjan's strongly connected components algorithm to find cycles.
/// Any SCC with more than one node indicates a cycle, except for inverse
/// relationship pairs which represent bidirectional traceability.
pub fn check_cycles(graph: &KnowledgeGraph) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    let inner = graph.inner();

    // Find strongly connected components
    let sccs = tarjan_scc(inner);

    for scc in sccs {
        if scc.len() == 2 {
            // Special case: check if this is just an inverse relationship pair
            if is_inverse_pair_only(graph, scc[0], scc[1]) {
                continue; // Skip this - it's not a real cycle
            }

            // It's a real cycle between two nodes
            let cycle_ids: Vec<String> = scc
                .iter()
                .filter_map(|idx| inner.node_weight(*idx))
                .map(|item| item.id.as_str().to_string())
                .collect();

            let cycle_str = cycle_ids.join(" -> ");
            let first_item = scc.first().and_then(|idx| inner.node_weight(*idx));
            let location = first_item.map(|item| item.source.clone());

            errors.push(ValidationError::CircularReference {
                cycle: cycle_str,
                location,
            });
        } else if scc.len() > 2 {
            // This SCC contains a cycle with more than 2 nodes
            let cycle_ids: Vec<String> = scc
                .iter()
                .filter_map(|idx| inner.node_weight(*idx))
                .map(|item| item.id.as_str().to_string())
                .collect();

            let cycle_str = cycle_ids.join(" -> ");

            // Get the first item's location for error reporting
            let first_item = scc.first().and_then(|idx| inner.node_weight(*idx));
            let location = first_item.map(|item| item.source.clone());

            errors.push(ValidationError::CircularReference {
                cycle: cycle_str,
                location,
            });
        } else if scc.len() == 1 {
            // Check for self-loop
            let idx = scc[0];
            if inner.find_edge(idx, idx).is_some()
                && let Some(item) = inner.node_weight(idx)
            {
                errors.push(ValidationError::CircularReference {
                    cycle: format!("{} -> {}", item.id.as_str(), item.id.as_str()),
                    location: Some(item.source.clone()),
                });
            }
        }
    }

    errors
}

/// Checks if adding an edge would create a cycle.
pub fn would_create_cycle(graph: &KnowledgeGraph, from: &ItemId, to: &ItemId) -> bool {
    // If to can reach from, adding from->to would create a cycle
    // This is a simple reachability check
    let inner = graph.inner();

    let from_idx = match graph.node_index(from) {
        Some(idx) => idx,
        None => return false,
    };

    let to_idx = match graph.node_index(to) {
        Some(idx) => idx,
        None => return false,
    };

    // Check if there's a path from 'to' back to 'from'
    petgraph::algo::has_path_connecting(inner, to_idx, from_idx, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::GraphBuilder;
    use crate::model::{ItemBuilder, ItemType, RelationshipType, SourceLocation, UpstreamRefs};
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
    fn test_no_cycles() {
        let graph = GraphBuilder::new()
            .add_item(create_item("SOL-001", ItemType::Solution, None))
            .add_item(create_item(
                "UC-001",
                ItemType::UseCase,
                Some(UpstreamRefs {
                    refines: vec![ItemId::new_unchecked("SOL-001")],
                    ..Default::default()
                }),
            ))
            .build()
            .unwrap();

        let errors = check_cycles(&graph);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_cycle_detected() {
        // Create a cycle: SCEN-001 -> SCEN-002 -> SCEN-001
        let mut graph = KnowledgeGraph::new(false);

        let scen1 = create_item(
            "SCEN-001",
            ItemType::Scenario,
            Some(UpstreamRefs {
                refines: vec![ItemId::new_unchecked("SCEN-002")],
                ..Default::default()
            }),
        );
        let scen2 = create_item(
            "SCEN-002",
            ItemType::Scenario,
            Some(UpstreamRefs {
                refines: vec![ItemId::new_unchecked("SCEN-001")],
                ..Default::default()
            }),
        );

        graph.add_item(scen1);
        graph.add_item(scen2);

        // Add the edges manually
        graph.add_relationship(
            &ItemId::new_unchecked("SCEN-001"),
            &ItemId::new_unchecked("SCEN-002"),
            RelationshipType::Refines,
        );
        graph.add_relationship(
            &ItemId::new_unchecked("SCEN-002"),
            &ItemId::new_unchecked("SCEN-001"),
            RelationshipType::Refines,
        );

        let errors = check_cycles(&graph);
        assert!(!errors.is_empty(), "Cycle should be detected");
    }

    #[test]
    fn test_would_create_cycle() {
        let mut graph = KnowledgeGraph::new(false);

        let sol = create_item("SOL-001", ItemType::Solution, None);
        let uc = create_item(
            "UC-001",
            ItemType::UseCase,
            Some(UpstreamRefs {
                refines: vec![ItemId::new_unchecked("SOL-001")],
                ..Default::default()
            }),
        );

        graph.add_item(sol);
        graph.add_item(uc);
        graph.add_relationship(
            &ItemId::new_unchecked("UC-001"),
            &ItemId::new_unchecked("SOL-001"),
            RelationshipType::Refines,
        );

        // Adding SOL-001 -> UC-001 would create a cycle
        assert!(would_create_cycle(
            &graph,
            &ItemId::new_unchecked("SOL-001"),
            &ItemId::new_unchecked("UC-001"),
        ));

        // Adding UC-001 -> new item would not create a cycle
        assert!(!would_create_cycle(
            &graph,
            &ItemId::new_unchecked("UC-001"),
            &ItemId::new_unchecked("SCEN-001"),
        ));
    }
}

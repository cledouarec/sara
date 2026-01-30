//! Circular reference detection validation rule.

use petgraph::algo::tarjan_scc;
use petgraph::visit::EdgeFiltered;

use crate::config::ValidationConfig;
use crate::error::ValidationError;
use crate::graph::KnowledgeGraph;
use crate::validation::rule::ValidationRule;

/// Circular reference detection rule.
///
/// Uses Tarjan's strongly connected components algorithm to find cycles.
/// Only considers primary relationships (not inverse edges) when detecting
/// cycles, since inverse edges are just for graph traversal and don't
/// represent logical cycles.
pub struct CyclesRule;

impl ValidationRule for CyclesRule {
    fn validate(&self, graph: &KnowledgeGraph, _config: &ValidationConfig) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        let inner = graph.inner();

        // Filter the graph to only include primary relationships for cycle detection.
        // Inverse relationships (IsRefinedBy, Derives, IsSatisfiedBy, etc.) are excluded
        // because they're just for traversal and would cause false positives.
        let filtered = EdgeFiltered::from_fn(inner, |edge| edge.weight().is_primary());

        // Find strongly connected components on the filtered graph
        let sccs = tarjan_scc(&filtered);

        for scc in sccs {
            if scc.len() >= 2 {
                // SCC with 2+ nodes indicates a cycle
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
            } else if scc.len() == 1 {
                // Check for self-loop (only with primary relationships)
                let idx = scc[0];
                let has_self_loop = inner
                    .edges_connecting(idx, idx)
                    .any(|e| e.weight().is_primary());

                if has_self_loop && let Some(item) = inner.node_weight(idx) {
                    errors.push(ValidationError::CircularReference {
                        cycle: format!("{} -> {}", item.id.as_str(), item.id.as_str()),
                        location: Some(item.source.clone()),
                    });
                }
            }
        }

        errors
    }
}

/// Checks if adding an edge would create a cycle.
#[cfg(test)]
fn would_create_cycle(
    graph: &KnowledgeGraph,
    from: &crate::model::ItemId,
    to: &crate::model::ItemId,
) -> bool {
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
    use crate::model::{ItemId, ItemType, RelationshipType, UpstreamRefs};
    use crate::test_utils::{create_test_item, create_test_item_with_upstream};

    #[test]
    fn test_no_cycles() {
        let graph = GraphBuilder::new()
            .add_item(create_test_item("SOL-001", ItemType::Solution))
            .add_item(create_test_item_with_upstream(
                "UC-001",
                ItemType::UseCase,
                UpstreamRefs {
                    refines: vec![ItemId::new_unchecked("SOL-001")],
                    ..Default::default()
                },
            ))
            .build()
            .unwrap();

        let rule = CyclesRule;
        let errors = rule.validate(&graph, &ValidationConfig::default());
        assert!(errors.is_empty());
    }

    #[test]
    fn test_cycle_detected() {
        // Create a cycle: SCEN-001 -> SCEN-002 -> SCEN-001
        let mut graph = KnowledgeGraph::new();

        let scen1 = create_test_item_with_upstream(
            "SCEN-001",
            ItemType::Scenario,
            UpstreamRefs {
                refines: vec![ItemId::new_unchecked("SCEN-002")],
                ..Default::default()
            },
        );
        let scen2 = create_test_item_with_upstream(
            "SCEN-002",
            ItemType::Scenario,
            UpstreamRefs {
                refines: vec![ItemId::new_unchecked("SCEN-001")],
                ..Default::default()
            },
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

        let rule = CyclesRule;
        let errors = rule.validate(&graph, &ValidationConfig::default());
        assert!(!errors.is_empty(), "Cycle should be detected");
    }

    #[test]
    fn test_would_create_cycle() {
        let mut graph = KnowledgeGraph::new();

        let sol = create_test_item("SOL-001", ItemType::Solution);
        let uc = create_test_item_with_upstream(
            "UC-001",
            ItemType::UseCase,
            UpstreamRefs {
                refines: vec![ItemId::new_unchecked("SOL-001")],
                ..Default::default()
            },
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

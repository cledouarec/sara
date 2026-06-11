//! Circular reference detection validation rule.

use petgraph::algo::tarjan_scc;
use petgraph::visit::EdgeFiltered;

use crate::config::ValidationConfig;
use crate::error::SaraError;
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
    fn validate(&self, graph: &KnowledgeGraph, _config: &ValidationConfig) -> Vec<SaraError> {
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

                errors.push(SaraError::CircularReference { cycle: cycle_str });
            } else if scc.len() == 1 {
                // Check for self-loop (only with primary relationships)
                let idx = scc[0];
                let has_self_loop = inner
                    .edges_connecting(idx, idx)
                    .any(|e| e.weight().is_primary());

                if has_self_loop && let Some(item) = inner.node_weight(idx) {
                    errors.push(SaraError::CircularReference {
                        cycle: format!("{} -> {}", item.id.as_str(), item.id.as_str()),
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

    use crate::graph::KnowledgeGraphBuilder;
    use crate::model::{ItemId, Relationship};
    use crate::schema::builtin;
    use crate::test_utils::{create_test_item, create_test_item_with_relationships};

    #[test]
    fn test_no_cycles() {
        let graph = KnowledgeGraphBuilder::new()
            .add_item(create_test_item("SOL-001", builtin::SOLUTION))
            .add_item(create_test_item_with_relationships(
                "UC-001",
                builtin::USE_CASE,
                vec![Relationship::new(
                    ItemId::new_unchecked("SOL-001"),
                    builtin::REFINES,
                )],
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
        let scen1 = create_test_item_with_relationships(
            "SCEN-001",
            builtin::SCENARIO,
            vec![Relationship::new(
                ItemId::new_unchecked("SCEN-002"),
                builtin::REFINES,
            )],
        );
        let scen2 = create_test_item_with_relationships(
            "SCEN-002",
            builtin::SCENARIO,
            vec![Relationship::new(
                ItemId::new_unchecked("SCEN-001"),
                builtin::REFINES,
            )],
        );

        let graph = KnowledgeGraphBuilder::new()
            .add_item(scen1)
            .add_item(scen2)
            .build()
            .unwrap();

        let rule = CyclesRule;
        let errors = rule.validate(&graph, &ValidationConfig::default());
        assert!(!errors.is_empty(), "Cycle should be detected");
    }

    #[test]
    fn test_peer_dependency_cycle_detected() {
        let req1 = create_test_item_with_relationships(
            "SYSREQ-001",
            builtin::SYSTEM_REQUIREMENT,
            vec![Relationship::new(
                ItemId::new_unchecked("SYSREQ-002"),
                builtin::DEPENDS_ON,
            )],
        );
        let req2 = create_test_item_with_relationships(
            "SYSREQ-002",
            builtin::SYSTEM_REQUIREMENT,
            vec![Relationship::new(
                ItemId::new_unchecked("SYSREQ-001"),
                builtin::DEPENDS_ON,
            )],
        );

        let graph = KnowledgeGraphBuilder::new()
            .add_item(req1)
            .add_item(req2)
            .build()
            .unwrap();

        let rule = CyclesRule;
        let errors = rule.validate(&graph, &ValidationConfig::default());
        assert!(!errors.is_empty(), "depends_on cycle should be detected");
    }

    #[test]
    fn test_peer_dependency_without_cycle_is_valid() {
        let req1 = create_test_item_with_relationships(
            "SYSREQ-001",
            builtin::SYSTEM_REQUIREMENT,
            vec![Relationship::new(
                ItemId::new_unchecked("SYSREQ-002"),
                builtin::DEPENDS_ON,
            )],
        );
        let req2 = create_test_item("SYSREQ-002", builtin::SYSTEM_REQUIREMENT);

        let graph = KnowledgeGraphBuilder::new()
            .add_item(req1)
            .add_item(req2)
            .build()
            .unwrap();

        let rule = CyclesRule;
        let errors = rule.validate(&graph, &ValidationConfig::default());
        assert!(
            errors.is_empty(),
            "a one-way dependency must not be reported as a cycle: {errors:?}"
        );
    }

    #[test]
    fn test_supersession_cycle_detected() {
        let adr1 = create_test_item_with_relationships(
            "ADR-001",
            builtin::ARCHITECTURE_DECISION_RECORD,
            vec![Relationship::new(
                ItemId::new_unchecked("ADR-002"),
                builtin::SUPERSEDES,
            )],
        );
        let adr2 = create_test_item_with_relationships(
            "ADR-002",
            builtin::ARCHITECTURE_DECISION_RECORD,
            vec![Relationship::new(
                ItemId::new_unchecked("ADR-001"),
                builtin::SUPERSEDES,
            )],
        );

        let graph = KnowledgeGraphBuilder::new()
            .add_item(adr1)
            .add_item(adr2)
            .build()
            .unwrap();

        let rule = CyclesRule;
        let errors = rule.validate(&graph, &ValidationConfig::default());
        assert!(!errors.is_empty(), "supersedes cycle should be detected");
    }

    #[test]
    fn test_peer_self_reference_detected() {
        let req = create_test_item_with_relationships(
            "SYSREQ-001",
            builtin::SYSTEM_REQUIREMENT,
            vec![Relationship::new(
                ItemId::new_unchecked("SYSREQ-001"),
                builtin::DEPENDS_ON,
            )],
        );

        let graph = KnowledgeGraphBuilder::new().add_item(req).build().unwrap();

        let rule = CyclesRule;
        let errors = rule.validate(&graph, &ValidationConfig::default());
        assert!(
            !errors.is_empty(),
            "a self-referencing dependency should be detected"
        );
    }

    #[test]
    fn test_would_create_cycle() {
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

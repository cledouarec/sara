//! Mermaid flowchart rendering of a part of the knowledge graph.
//!
//! Renders the subgraph induced by a set of items: every item of the set is
//! a node, and every relationship of the knowledge graph whose two endpoints
//! are both in the set is an edge. Drawing the induced subgraph, rather than
//! the parent links a traversal recorded, keeps the cross edges an item has
//! toward several parents inside the same picture.
//!
//! The origin node is assigned the `origin` class and every node a class
//! named after its item type id. None of these classes carries a style, so a
//! caller styles them by appending its own `classDef` statements to the
//! diagram.

use std::collections::{BTreeMap, BTreeSet};

use crate::graph::{KnowledgeGraph, TraversalOptions, TraversalResult};
use crate::model::{ItemId, RelationshipType};

/// Diagram header line: a bottom-to-top layout, so that an edge drawn from
/// its declaring side points upward and upstream items sit above the items
/// that refine, derive from or satisfy them.
const DIAGRAM_HEADER: &str = "flowchart BT";

/// Indentation of every statement below the header.
const INDENT: &str = "    ";

/// Line break inside a node label, between the id and the name.
const LABEL_BREAK: &str = "<br>";

/// Name of the class marking the origin node.
const ORIGIN_CLASS: &str = "origin";

/// Separator between the node ids of a single `class` statement.
const CLASS_MEMBER_SEPARATOR: &str = ",";

/// Mermaid entity codes for characters that would break a quoted label.
const LABEL_ESCAPES: [(char, &str); 3] = [('"', "#quot;"), ('<', "#lt;"), ('>', "#gt;")];

/// Renders a traversal result as a raw Mermaid `flowchart BT`.
///
/// The picture contains every item the traversal reported and the
/// relationships between them; see [`render`] for the layout.
#[must_use]
pub fn traversal_to_mermaid(result: &TraversalResult, graph: &KnowledgeGraph) -> String {
    render(
        graph,
        &result.origin,
        result.items.iter().map(|node| &node.item_id),
    )
}

/// Renders an item and its direct relationships as a raw Mermaid
/// `flowchart BT`.
///
/// The picture contains the item and every item it is directly related to,
/// whichever side declared the relation; see [`render`] for the layout.
/// The related items are limited to the type filter of `options`; the
/// origin is always drawn.
#[must_use]
pub fn neighborhood_to_mermaid(
    graph: &KnowledgeGraph,
    origin: &ItemId,
    options: &TraversalOptions,
) -> String {
    let related = graph.direct_relationships(origin, options);
    let members = related
        .iter()
        .flat_map(|(_, items)| items)
        .map(|item| &item.id)
        .chain([origin]);
    render(graph, origin, members)
}

/// Renders the subgraph induced by `members` as a raw Mermaid `flowchart BT`.
///
/// Nodes are labeled with the item id and name, and edges carry the relation
/// id declared by the source item (downstream and inverse peer relations are
/// rendered from their declaring side). The origin gets the `origin` class
/// and every node a class named after its item type id, all left unstyled so
/// the caller can define them. No code fence is emitted, so the caller
/// decides how to wrap the diagram. Members unknown to the graph are ignored.
fn render<'a>(
    graph: &KnowledgeGraph,
    origin: &ItemId,
    members: impl IntoIterator<Item = &'a ItemId>,
) -> String {
    let nodes: BTreeMap<&str, (&str, &str)> = members
        .into_iter()
        .filter_map(|id| graph.get(id))
        .map(|item| {
            (
                item.id.as_str(),
                (item.name.as_str(), item.item_type.as_str()),
            )
        })
        .collect();

    let mut type_classes: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (id, (_, item_type)) in &nodes {
        type_classes.entry(item_type).or_default().push(id);
    }

    let edges: BTreeSet<(&str, &str, &str)> = graph
        .relationships()
        .filter(|(from, to, _)| {
            nodes.contains_key(from.as_str()) && nodes.contains_key(to.as_str())
        })
        .map(|(from, to, rel)| declaring_side(from, to, rel))
        .map(|(from, to, rel)| (from.as_str(), to.as_str(), rel.as_str()))
        .collect();

    let mut lines = vec![DIAGRAM_HEADER.to_string()];
    lines.extend(nodes.iter().map(|(id, (name, _))| {
        format!(
            "{INDENT}{id}[\"{id}{LABEL_BREAK}{name}\"]",
            name = escape_label(name)
        )
    }));
    lines.extend(
        edges
            .iter()
            .map(|(from, to, rel)| format!("{INDENT}{from} -->|{rel}| {to}")),
    );
    lines.push(format!("{INDENT}class {origin} {ORIGIN_CLASS}"));
    lines.extend(type_classes.iter().map(|(item_type, ids)| {
        format!(
            "{INDENT}class {ids} {item_type}",
            ids = ids.join(CLASS_MEMBER_SEPARATOR)
        )
    }));
    lines.push(String::new());

    lines.join("\n")
}

/// Orients an edge from the side that declares it.
///
/// Downstream relations and inverse peers exist in the graph as the mirror
/// of a declaration made by the other endpoint; rendering them from that
/// endpoint gives every relationship a single canonical edge, labeled with
/// the upstream name when one exists and with the peer's own name otherwise.
fn declaring_side<'a>(
    from: &'a ItemId,
    to: &'a ItemId,
    rel: RelationshipType,
) -> (&'a ItemId, &'a ItemId, RelationshipType) {
    if rel.is_downstream() || (rel.is_peer() && !rel.is_primary()) {
        (to, from, rel.inverse())
    } else {
        (from, to, rel)
    }
}

/// Escapes the characters Mermaid cannot carry inside a quoted label.
fn escape_label(name: &str) -> String {
    LABEL_ESCAPES
        .iter()
        .fold(name.to_string(), |acc, (ch, code)| acc.replace(*ch, code))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::graph::{KnowledgeGraphBuilder, TraversalOptions, traverse_upstream};
    use crate::model::{Item, ItemId, Relationship};
    use crate::schema::builtin;
    use crate::test_utils::{
        create_test_item, create_test_item_with_name, create_test_item_with_relationships,
    };

    fn id(value: &str) -> ItemId {
        ItemId::new_unchecked(value)
    }

    fn render_upstream(items: Vec<Item>, origin: &str) -> String {
        let graph = KnowledgeGraphBuilder::new()
            .add_items(items)
            .build()
            .unwrap();
        let result = traverse_upstream(&graph, &id(origin), &TraversalOptions::new()).unwrap();
        traversal_to_mermaid(&result, &graph)
    }

    #[test]
    fn renders_chain_with_upstream_relation_name() {
        let items = vec![
            create_test_item_with_name("SOL-001", builtin::SOLUTION, "Payment platform"),
            create_test_item_with_relationships(
                "UC-001",
                builtin::USE_CASE,
                vec![Relationship::new(id("SOL-001"), builtin::REFINES)],
            ),
        ];

        let expected = "\
flowchart BT
    SOL-001[\"SOL-001<br>Payment platform\"]
    UC-001[\"UC-001<br>Test UC-001\"]
    UC-001 -->|refines| SOL-001
    class UC-001 origin
    class SOL-001 solution
    class UC-001 use_case
";
        assert_eq!(render_upstream(items, "UC-001"), expected);
    }

    #[test]
    fn renders_cross_edges_of_a_diamond() {
        // SWDD-001 satisfies two requirements that both derive from the same
        // architecture: the traversal records one parent for SYSARCH-001,
        // the diagram must still show both derives_from edges.
        let items = vec![
            create_test_item("SYSARCH-001", builtin::SYSTEM_ARCHITECTURE),
            create_test_item_with_relationships(
                "SWREQ-001",
                builtin::SOFTWARE_REQUIREMENT,
                vec![Relationship::new(id("SYSARCH-001"), builtin::DERIVES_FROM)],
            ),
            create_test_item_with_relationships(
                "SWREQ-002",
                builtin::SOFTWARE_REQUIREMENT,
                vec![Relationship::new(id("SYSARCH-001"), builtin::DERIVES_FROM)],
            ),
            create_test_item_with_relationships(
                "SWDD-001",
                builtin::SOFTWARE_DETAILED_DESIGN,
                vec![
                    Relationship::new(id("SWREQ-001"), builtin::SATISFIES),
                    Relationship::new(id("SWREQ-002"), builtin::SATISFIES),
                ],
            ),
        ];

        let output = render_upstream(items, "SWDD-001");

        assert!(output.contains("    SWREQ-001 -->|derives_from| SYSARCH-001\n"));
        assert!(output.contains("    SWREQ-002 -->|derives_from| SYSARCH-001\n"));
        assert!(output.contains("    SWDD-001 -->|satisfies| SWREQ-001\n"));
        assert!(output.contains("    SWDD-001 -->|satisfies| SWREQ-002\n"));
        assert_eq!(output.matches("-->").count(), 4);
    }

    #[test]
    fn renders_downstream_declaration_from_declaring_side() {
        // The solution declares `is_refined_by`; the diagram draws the edge
        // as UC-001 refines SOL-001, once.
        let items = vec![
            create_test_item_with_relationships(
                "SOL-001",
                builtin::SOLUTION,
                vec![Relationship::new(id("UC-001"), builtin::IS_REFINED_BY)],
            ),
            create_test_item("UC-001", builtin::USE_CASE),
        ];

        let output = render_upstream(items, "UC-001");

        assert!(output.contains("    UC-001 -->|refines| SOL-001\n"));
        assert_eq!(output.matches("-->").count(), 1);
    }

    #[test]
    fn renders_peer_relation_once_with_its_own_name() {
        // A primary peer gets an inverse edge in the graph; only the declared
        // `depends_on` edge is drawn.
        let items = vec![
            create_test_item("SYSARCH-001", builtin::SYSTEM_ARCHITECTURE),
            create_test_item_with_relationships(
                "SWREQ-001",
                builtin::SOFTWARE_REQUIREMENT,
                vec![Relationship::new(id("SYSARCH-001"), builtin::DERIVES_FROM)],
            ),
            create_test_item_with_relationships(
                "SWREQ-002",
                builtin::SOFTWARE_REQUIREMENT,
                vec![
                    Relationship::new(id("SYSARCH-001"), builtin::DERIVES_FROM),
                    Relationship::new(id("SWREQ-001"), builtin::DEPENDS_ON),
                ],
            ),
        ];
        let graph = KnowledgeGraphBuilder::new()
            .add_items(items)
            .build()
            .unwrap();
        let result =
            crate::graph::traverse_downstream(&graph, &id("SYSARCH-001"), &TraversalOptions::new())
                .unwrap();

        let output = traversal_to_mermaid(&result, &graph);

        assert!(output.contains("    SWREQ-002 -->|depends_on| SWREQ-001\n"));
        assert!(!output.contains("is_required_by"));
        assert_eq!(output.matches("-->").count(), 3);
    }

    /// A requirement deriving from an architecture, required by a peer
    /// requirement and satisfied by a design.
    fn neighborhood_graph() -> KnowledgeGraph {
        let items = vec![
            create_test_item("SYSARCH-001", builtin::SYSTEM_ARCHITECTURE),
            create_test_item_with_relationships(
                "SWREQ-001",
                builtin::SOFTWARE_REQUIREMENT,
                vec![Relationship::new(id("SYSARCH-001"), builtin::DERIVES_FROM)],
            ),
            create_test_item_with_relationships(
                "SWREQ-002",
                builtin::SOFTWARE_REQUIREMENT,
                vec![Relationship::new(id("SWREQ-001"), builtin::DEPENDS_ON)],
            ),
            create_test_item_with_relationships(
                "SWDD-001",
                builtin::SOFTWARE_DETAILED_DESIGN,
                vec![Relationship::new(id("SWREQ-001"), builtin::SATISFIES)],
            ),
        ];
        KnowledgeGraphBuilder::new()
            .add_items(items)
            .build()
            .unwrap()
    }

    #[test]
    fn renders_direct_neighborhood_including_peers() {
        // Without a traversal the picture is the item and every item it is
        // directly related to, whichever side declared the relation.
        let graph = neighborhood_graph();

        let expected = "\
flowchart BT
    SWDD-001[\"SWDD-001<br>Test SWDD-001\"]
    SWREQ-001[\"SWREQ-001<br>Test SWREQ-001\"]
    SWREQ-002[\"SWREQ-002<br>Test SWREQ-002\"]
    SYSARCH-001[\"SYSARCH-001<br>Test SYSARCH-001\"]
    SWDD-001 -->|satisfies| SWREQ-001
    SWREQ-001 -->|derives_from| SYSARCH-001
    SWREQ-002 -->|depends_on| SWREQ-001
    class SWREQ-001 origin
    class SWDD-001 software_detailed_design
    class SWREQ-001,SWREQ-002 software_requirement
    class SYSARCH-001 system_architecture
";
        assert_eq!(
            neighborhood_to_mermaid(&graph, &id("SWREQ-001"), &TraversalOptions::new()),
            expected
        );
    }

    #[test]
    fn neighborhood_keeps_only_requested_types() {
        let graph = neighborhood_graph();

        // The origin is always drawn; the related items are limited to the
        // requested types, so the peer requirement and the architecture drop.
        let expected = "\
flowchart BT
    SWDD-001[\"SWDD-001<br>Test SWDD-001\"]
    SWREQ-001[\"SWREQ-001<br>Test SWREQ-001\"]
    SWDD-001 -->|satisfies| SWREQ-001
    class SWREQ-001 origin
    class SWDD-001 software_detailed_design
    class SWREQ-001 software_requirement
";
        let options = TraversalOptions::new().with_types(vec![builtin::SOFTWARE_DETAILED_DESIGN]);
        assert_eq!(
            neighborhood_to_mermaid(&graph, &id("SWREQ-001"), &options),
            expected
        );
    }

    #[test]
    fn escapes_quotes_and_angle_brackets_in_names() {
        let items = vec![create_test_item_with_name(
            "SOL-001",
            builtin::SOLUTION,
            "Say \"hi\" <now>",
        )];

        let output = render_upstream(items, "SOL-001");

        assert!(output.contains("    SOL-001[\"SOL-001<br>Say #quot;hi#quot; #lt;now#gt;\"]\n"));
    }

    #[test]
    fn omits_edges_toward_items_outside_the_result() {
        // Depth 1 keeps UC-001 and SOL-001 only; SCEN-001's link to UC-001
        // is not drawn because SCEN-001 is not in the picture.
        let items = vec![
            create_test_item("SOL-001", builtin::SOLUTION),
            create_test_item_with_relationships(
                "UC-001",
                builtin::USE_CASE,
                vec![Relationship::new(id("SOL-001"), builtin::REFINES)],
            ),
            create_test_item_with_relationships(
                "SCEN-001",
                builtin::SCENARIO,
                vec![Relationship::new(id("UC-001"), builtin::REFINES)],
            ),
        ];
        let graph = KnowledgeGraphBuilder::new()
            .add_items(items)
            .build()
            .unwrap();
        let result = traverse_upstream(
            &graph,
            &id("UC-001"),
            &TraversalOptions::new().with_max_depth(1),
        )
        .unwrap();

        let output = traversal_to_mermaid(&result, &graph);

        assert!(!output.contains("SCEN-001"));
        assert_eq!(output.matches("-->").count(), 1);
    }
}

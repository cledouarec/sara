//! Tests of the `sara_trace_item` tool.

mod common;

use serde_json::{Value, json};

/// Returns `(id, depth)` pairs of the items of a chain.
fn items(chain: &Value) -> Vec<(String, u64)> {
    chain["items"]
        .as_array()
        .expect("items")
        .iter()
        .map(|item| {
            (
                item["id"].as_str().unwrap().to_string(),
                item["depth"].as_u64().unwrap(),
            )
        })
        .collect()
}

/// Returns `(id, depth)` pairs from string literals.
fn expected(pairs: &[(&str, u64)]) -> Vec<(String, u64)> {
    pairs
        .iter()
        .map(|(id, depth)| ((*id).to_string(), *depth))
        .collect()
}

#[tokio::test]
async fn test_trace_reports_shared_ancestor_once() {
    let client = common::connect_to_fixture("diamond").await;

    let trace = common::structured(
        &client,
        "sara_trace_item",
        json!({ "id": "SYSARCH-001", "direction": "upstream" }),
    )
    .await;

    assert_eq!(trace["origin"]["id"], "SYSARCH-001");
    assert!(trace.get("downstream").is_none(), "{trace}");
    assert_eq!(
        items(&trace["upstream"]),
        expected(&[
            ("SYSREQ-001", 1),
            ("SYSREQ-002", 1),
            ("SCEN-001", 2),
            ("UC-001", 3),
            ("SOL-001", 4),
        ])
    );
    assert_eq!(trace["upstream"]["links"].as_array().unwrap().len(), 6);
    assert_eq!(trace["upstream"]["truncated"], false);

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_trace_follows_both_directions_by_default() {
    let client = common::connect_to_fixture("valid_graph").await;

    let trace = common::structured(&client, "sara_trace_item", json!({ "id": "SYSREQ-001" })).await;

    assert_eq!(
        items(&trace["upstream"]),
        expected(&[("SCEN-001", 1), ("UC-001", 2), ("SOL-001", 3)])
    );
    assert_eq!(
        items(&trace["downstream"]),
        expected(&[("SYSARCH-001", 1), ("SWREQ-001", 2), ("SWDD-001", 3)])
    );
    assert_eq!(
        trace["downstream"]["links"][0],
        json!({ "from": "SYSREQ-001", "relation": "is_satisfied_by", "to": "SYSARCH-001" })
    );

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_trace_limit_truncates_the_chain() {
    let client = common::connect_to_fixture("valid_graph").await;

    let trace = common::structured(
        &client,
        "sara_trace_item",
        json!({ "id": "SYSREQ-001", "direction": "downstream", "limit": 1 }),
    )
    .await;

    assert_eq!(items(&trace["downstream"]), expected(&[("SYSARCH-001", 1)]));
    assert_eq!(trace["downstream"]["links"].as_array().unwrap().len(), 1);
    assert_eq!(trace["downstream"]["truncated"], true);

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_trace_max_depth_truncates_the_chain() {
    let client = common::connect_to_fixture("valid_graph").await;

    let trace = common::structured(
        &client,
        "sara_trace_item",
        json!({ "id": "SYSREQ-001", "direction": "upstream", "max_depth": 1 }),
    )
    .await;

    assert_eq!(items(&trace["upstream"]), expected(&[("SCEN-001", 1)]));
    assert_eq!(trace["upstream"]["truncated"], true);

    client.cancel().await.unwrap();
}

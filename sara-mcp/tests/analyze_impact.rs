//! Tests of the `sara_analyze_impact` tool.

mod common;

use serde_json::json;

#[tokio::test]
async fn test_impact_follows_links_and_dependencies() {
    let client = common::connect_to_fixture("query_relations").await;

    let impact = common::structured(
        &client,
        "sara_analyze_impact",
        json!({ "ids": ["SWREQ-QR-001"] }),
    )
    .await;

    assert_eq!(impact["origins"][0]["id"], "SWREQ-QR-001");
    assert_eq!(impact["total"], 3);
    assert_eq!(impact["truncated"], false);
    assert_eq!(
        impact["by_type"],
        json!({
            "architecture_decision_record": 1,
            "software_detailed_design": 1,
            "software_requirement": 1,
        })
    );

    let reached: Vec<(String, u64, String, String)> = impact["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| {
            (
                item["id"].as_str().unwrap().to_string(),
                item["depth"].as_u64().unwrap(),
                item["from"].as_str().unwrap().to_string(),
                item["relation"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    let expected = [
        ("SWDD-QR-001", 1, "SWREQ-QR-001", "is_satisfied_by"),
        ("SWREQ-QR-002", 1, "SWREQ-QR-001", "is_required_by"),
        ("ADR-QR-001", 2, "SWDD-QR-001", "justified_by"),
    ]
    .map(|(id, depth, from, relation)| {
        (
            id.to_string(),
            depth,
            from.to_string(),
            relation.to_string(),
        )
    });
    assert_eq!(reached, expected);
    assert!(
        impact["items"][0]["file"]
            .as_str()
            .unwrap()
            .ends_with("SWDD-QR-001.md")
    );

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_impact_limit_truncates_the_items() {
    let client = common::connect_to_fixture("query_relations").await;

    let impact = common::structured(
        &client,
        "sara_analyze_impact",
        json!({ "ids": ["SWREQ-QR-001"], "limit": 1 }),
    )
    .await;

    assert_eq!(impact["total"], 3);
    assert_eq!(impact["truncated"], true);
    assert_eq!(impact["items"].as_array().unwrap().len(), 1);
    assert_eq!(impact["items"][0]["id"], "SWDD-QR-001");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_impact_rejects_unknown_item() {
    let client = common::connect_to_fixture("query_relations").await;

    let result = common::call(
        &client,
        "sara_analyze_impact",
        json!({ "ids": ["SWREQ-QR-009"] }),
    )
    .await;
    let text = common::text(&result);

    assert_eq!(result.is_error, Some(true));
    assert!(text.contains("SWREQ-QR-009"), "{text}");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_impact_requires_an_item() {
    let client = common::connect_to_fixture("query_relations").await;

    let result = common::call(&client, "sara_analyze_impact", json!({ "ids": [] })).await;

    assert_eq!(result.is_error, Some(true));

    client.cancel().await.unwrap();
}

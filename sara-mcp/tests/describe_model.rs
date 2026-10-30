//! Tests of the `sara_describe_model` tool.

mod common;

use serde_json::json;

#[tokio::test]
async fn test_describe_model_lists_types_and_item_counts() {
    let client = common::connect_to_fixture("valid_graph").await;

    let result = common::call(&client, "sara_describe_model", json!({})).await;
    let text = common::text(&result);

    assert_ne!(result.is_error, Some(true), "{text}");
    assert!(text.contains("item_types:"), "{text}");
    assert!(text.contains("relations:"), "{text}");
    assert!(text.contains("item_counts:\n"), "{text}");
    assert!(text.contains("  solution: 1\n"), "{text}");
    assert!(text.contains("  hardware_requirement: 0\n"), "{text}");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_describe_model_is_read_only() {
    let client = common::connect_to_fixture("valid_graph").await;

    let tool = common::tool(&client, "sara_describe_model").await;
    let annotations = tool.annotations.expect("annotations");

    assert_eq!(annotations.read_only_hint, Some(true));
    assert_eq!(annotations.destructive_hint, Some(false));
    assert_eq!(annotations.idempotent_hint, Some(true));
    assert_eq!(annotations.open_world_hint, Some(false));

    client.cancel().await.unwrap();
}

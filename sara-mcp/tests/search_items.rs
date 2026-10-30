//! Tests of the `sara_search_items` tool.

mod common;

use serde_json::json;

#[tokio::test]
async fn test_search_ranks_best_match_first() {
    let client = common::connect_to_fixture("valid_graph").await;

    let results = common::structured(
        &client,
        "sara_search_items",
        json!({ "query": "response time" }),
    )
    .await;

    assert_eq!(results["hits"][0]["id"], "SYSREQ-001");
    assert_eq!(results["hits"][0]["type"], "system_requirement");
    assert!(
        results["hits"][0]["file"]
            .as_str()
            .unwrap()
            .ends_with("SYSREQ-001.md")
    );
    assert_eq!(results["hits"][0]["matched_in"][0], "name");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_search_snippet_comes_from_the_body() {
    let client = common::connect_to_fixture("valid_graph").await;

    let results = common::structured(
        &client,
        "sara_search_items",
        json!({ "query": "abandonment" }),
    )
    .await;

    assert_eq!(results["total"], 1);
    assert_eq!(results["hits"][0]["matched_in"], json!(["body"]));
    assert!(
        results["hits"][0]["snippet"]
            .as_str()
            .unwrap()
            .contains("abandonment")
    );

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_search_without_query_lists_items_page_by_page() {
    let client = common::connect_to_fixture("valid_graph").await;

    let first = common::structured(&client, "sara_search_items", json!({ "limit": 2 })).await;
    assert_eq!(first["total"], 7);
    assert_eq!(first["hits"].as_array().unwrap().len(), 2);
    assert_eq!(first["hits"][0]["id"], "SCEN-001");
    assert_eq!(first["has_more"], true);
    assert_eq!(first["next_offset"], 2);

    let last = common::structured(
        &client,
        "sara_search_items",
        json!({ "limit": 2, "offset": 6 }),
    )
    .await;
    assert_eq!(last["hits"].as_array().unwrap().len(), 1);
    assert_eq!(last["has_more"], false);
    assert!(last.get("next_offset").is_none(), "{last}");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_search_filters_by_type_prefix() {
    let client = common::connect_to_fixture("valid_graph").await;

    let results =
        common::structured(&client, "sara_search_items", json!({ "types": ["SYSREQ"] })).await;

    assert_eq!(results["total"], 1);
    assert_eq!(results["hits"][0]["id"], "SYSREQ-001");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_search_rejects_unknown_type() {
    let client = common::connect_to_fixture("valid_graph").await;

    let result = common::call(&client, "sara_search_items", json!({ "types": ["bogus"] })).await;
    let text = common::text(&result);

    assert_eq!(result.is_error, Some(true));
    assert!(text.contains("bogus"), "{text}");
    assert!(text.contains("system_requirement (SYSREQ)"), "{text}");

    client.cancel().await.unwrap();
}

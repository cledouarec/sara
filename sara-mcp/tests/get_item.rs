//! Tests of the `sara_get_item` tool.

mod common;

use serde_json::{Value, json};

/// Returns the link group of `relation` in an item view.
fn group<'a>(item: &'a Value, relation: &str) -> &'a Value {
    item["links"]
        .as_array()
        .expect("links")
        .iter()
        .find(|group| group["relation"] == relation)
        .unwrap_or_else(|| panic!("no {relation} group in {item}"))
}

#[tokio::test]
async fn test_get_item_returns_fields_relations_links_and_body() {
    let client = common::connect_to_fixture("valid_graph").await;

    let item = common::structured(&client, "sara_get_item", json!({ "id": "SYSREQ-001" })).await;

    assert_eq!(item["type"], "system_requirement");
    assert_eq!(item["name"], "Authentication Response Time");
    assert_eq!(
        item["fields"]["specification"],
        "The system SHALL authenticate users within 2 seconds under normal load."
    );
    assert!(item["file"].as_str().unwrap().ends_with("SYSREQ-001.md"));
    assert_eq!(item["relations"], json!({ "derives_from": ["SCEN-001"] }));
    assert!(
        item["body"]
            .as_str()
            .unwrap()
            .contains("## Acceptance Criteria")
    );

    let derives_from = group(&item, "derives_from");
    assert_eq!(derives_from["direction"], "upstream");
    assert_eq!(
        derives_from["items"],
        json!([{ "id": "SCEN-001", "type": "scenario", "name": "Successful Login" }])
    );

    let satisfied_by = group(&item, "is_satisfied_by");
    assert_eq!(satisfied_by["direction"], "downstream");
    // Declared in the architecture's file, so absent from the relations.
    assert_eq!(satisfied_by["items"][0]["id"], "SYSARCH-001");
    assert!(item["relations"].get("is_satisfied_by").is_none(), "{item}");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_get_item_without_body() {
    let client = common::connect_to_fixture("valid_graph").await;

    let item = common::structured(
        &client,
        "sara_get_item",
        json!({ "id": "SYSREQ-001", "include_body": false }),
    )
    .await;

    assert!(item.get("body").is_none(), "{item}");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_get_item_unknown_id_suggests_close_ids() {
    let client = common::connect_to_fixture("valid_graph").await;

    let result = common::call(&client, "sara_get_item", json!({ "id": "SYSREQ-01" })).await;
    let text = common::text(&result);

    assert_eq!(result.is_error, Some(true));
    assert!(text.contains("SYSREQ-001"), "{text}");
    assert!(text.contains("sara_search_items"), "{text}");

    client.cancel().await.unwrap();
}

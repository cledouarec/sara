//! Every structured result must match the output schema of its tool,
//! including results that leave optional parts out: clients built on the
//! official SDKs reject results that do not.

mod common;

use std::fs;

use serde_json::{Value, json};

/// Marks of developer documentation, which must not reach clients: rustdoc
/// links and Rust paths.
const DEVELOPER_MARKERS: [&str; 4] = ["[`", "](", "crate::", "Self::"];

/// Keys holding text meant for the reader of a schema.
const TEXT_KEYS: [&str; 2] = ["description", "title"];

/// Schema keyword marking a property clients never receive.
const WRITE_ONLY: &str = "writeOnly";

#[tokio::test]
async fn test_read_results_match_their_schemas() {
    let client = common::connect_to_fixture("valid_graph").await;

    // A solution sets no field; SYSREQ-001 sets one and has a body.
    common::call_conforming(&client, "sara_get_item", json!({ "id": "SOL-001" })).await;
    common::call_conforming(&client, "sara_get_item", json!({ "id": "SYSREQ-001" })).await;
    common::call_conforming(
        &client,
        "sara_get_item",
        json!({ "id": "SOL-001", "include_body": false }),
    )
    .await;
    // Listing reports no match locations nor snippets.
    common::call_conforming(&client, "sara_search_items", json!({})).await;
    common::call_conforming(
        &client,
        "sara_search_items",
        json!({ "query": "authentication" }),
    )
    .await;
    common::call_conforming(&client, "sara_trace_item", json!({ "id": "SYSREQ-001" })).await;
    // One direction only, cut by the item limit.
    common::call_conforming(
        &client,
        "sara_trace_item",
        json!({ "id": "SYSREQ-001", "direction": "upstream", "limit": 1 }),
    )
    .await;
    common::call_conforming(
        &client,
        "sara_analyze_impact",
        json!({ "ids": ["SOL-001"] }),
    )
    .await;
    // A healthy graph has no issues nor skipped files.
    common::call_conforming(&client, "sara_validate", json!({})).await;

    client.cancel().await.unwrap();

    let client = common::connect_to_fixture("broken_refs").await;
    common::call_conforming(&client, "sara_validate", json!({})).await;
    client.cancel().await.unwrap();

    // A file the scan skipped is reported by path.
    let client = common::connect_to_fixture("corrupt_file").await;
    common::call_conforming(&client, "sara_validate", json!({})).await;
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_write_results_match_their_schemas() {
    let (repository, client) = common::connect_to_copy("valid_graph").await;
    let file = repository.path().join("SWREQ-002.md");

    // All required fields given: no defaulted fields.
    common::call_conforming(
        &client,
        "sara_create_item",
        json!({
            "type": "SWREQ",
            "file": file.to_str().unwrap(),
            "fields": { "specification": "The software SHALL refresh expired tokens." },
        }),
    )
    .await;
    // No required field given: defaulted fields.
    let bare = repository.path().join("SWREQ-003.md");
    common::call_conforming(
        &client,
        "sara_create_item",
        json!({ "type": "SWREQ", "file": bare.to_str().unwrap() }),
    )
    .await;
    // No link left behind: no kept links.
    common::call_conforming(
        &client,
        "sara_update_item",
        json!({ "id": "SWREQ-002", "name": "Token refresh" }),
    )
    .await;

    client.cancel().await.unwrap();

    // A link written in the linked item's file: kept links.
    let repository = tempfile::TempDir::new().unwrap();
    fs::write(
        repository.path().join("SOL-001.md"),
        "---\nid: \"SOL-001\"\ntype: solution\nname: \"Hub\"\nis_refined_by:\n  - \"UC-002\"\n---\n",
    )
    .unwrap();
    fs::write(
        repository.path().join("UC-002.md"),
        "---\nid: \"UC-002\"\ntype: use_case\nname: \"Scenes\"\n---\n",
    )
    .unwrap();
    let client = common::connect_to_dir(repository.path()).await;
    let updated = common::call_conforming(
        &client,
        "sara_update_item",
        json!({ "id": "UC-002", "relations": { "refines": [] } }),
    )
    .await;
    assert!(updated.get("kept_links").is_some(), "{updated}");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_schemas_describe_results_for_clients() {
    let client = common::connect_to_fixture("valid_graph").await;
    let tools = client.list_all_tools().await.unwrap();
    client.cancel().await.unwrap();

    for tool in tools {
        let mut texts: Vec<String> = tool.description.iter().map(ToString::to_string).collect();
        collect_texts(&Value::Object((*tool.input_schema).clone()), &mut texts);
        if let Some(output) = &tool.output_schema {
            let output = Value::Object((**output).clone());
            assert!(
                !has_key(&output, WRITE_ONLY),
                "{} output schema has write-only properties",
                tool.name
            );
            collect_texts(&output, &mut texts);
        }
        for text in &texts {
            for marker in DEVELOPER_MARKERS {
                assert!(!text.contains(marker), "{}: {text}", tool.name);
            }
        }
    }
}

/// Collects the descriptions and titles found in `schema`.
fn collect_texts(schema: &Value, texts: &mut Vec<String>) {
    match schema {
        Value::Object(object) => {
            for (key, value) in object {
                match value {
                    Value::String(text) if TEXT_KEYS.contains(&key.as_str()) => {
                        texts.push(text.clone());
                    }
                    _ => collect_texts(value, texts),
                }
            }
        }
        Value::Array(values) => {
            for value in values {
                collect_texts(value, texts);
            }
        }
        _ => {}
    }
}

/// Whether `schema` or one of its parts has the key `key`.
fn has_key(schema: &Value, key: &str) -> bool {
    match schema {
        Value::Object(object) => {
            object.contains_key(key) || object.values().any(|value| has_key(value, key))
        }
        Value::Array(values) => values.iter().any(|value| has_key(value, key)),
        _ => false,
    }
}

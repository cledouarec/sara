//! Tests of the `sara_update_item` tool.

mod common;

use std::fs;
use std::path::Path;

use serde_json::json;

/// A scenario declaring a link from its downstream side, a key unknown to
/// the model and a comment.
const SCENARIO: &str = r#"---
id: "SCEN-001"
type: scenario
name: "Adjust Living Room Dimmer"
# Declared on this side on purpose
derives:
  - "SYSREQ-001"
owner: "alice"
---
# Adjust Living Room Dimmer
"#;

/// A system requirement that `SCEN-001` may derive.
const REQUIREMENT: &str = r#"---
id: "SYSREQ-002"
type: system_requirement
name: "Dimming latency"
specification: "The system SHALL dim lights within 200 ms."
---
"#;

/// Writes `files` into a new repository.
fn repository(files: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::TempDir::new().unwrap();
    for (name, content) in files {
        fs::write(dir.path().join(name), content).unwrap();
    }
    dir
}

/// Reads a file of a repository.
fn read(repository: &Path, name: &str) -> String {
    fs::read_to_string(repository.join(name)).unwrap()
}

#[tokio::test]
async fn test_update_item_rewrites_only_the_given_key() {
    let repository = common::copy_fixture("valid_graph");
    let before = read(repository.path(), "SYSREQ-001.md");
    let client = common::connect_to_dir(repository.path()).await;

    let updated = common::structured(
        &client,
        "sara_update_item",
        json!({ "id": "SYSREQ-001", "name": "Fast authentication" }),
    )
    .await;

    assert_eq!(
        updated["changes"],
        json!([{
            "field": "Name",
            "old_value": "Authentication Response Time",
            "new_value": "Fast authentication",
        }])
    );
    assert_eq!(
        read(repository.path(), "SYSREQ-001.md"),
        before.replace(
            "name: \"Authentication Response Time\"",
            "name: \"Fast authentication\""
        )
    );

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_update_item_keeps_entries_outside_the_change() {
    let repository = repository(&[("SCEN-001.md", SCENARIO)]);
    let client = common::connect_to_dir(repository.path()).await;

    let result = common::call(
        &client,
        "sara_update_item",
        json!({ "id": "SCEN-001", "name": "Dim the lights" }),
    )
    .await;

    assert_ne!(result.is_error, Some(true), "{}", common::text(&result));
    assert_eq!(
        read(repository.path(), "SCEN-001.md"),
        SCENARIO.replace(
            "name: \"Adjust Living Room Dimmer\"",
            "name: \"Dim the lights\""
        )
    );

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_update_item_replaces_relation_held_outside_the_type() {
    let repository = repository(&[("SCEN-001.md", SCENARIO), ("SYSREQ-002.md", REQUIREMENT)]);
    let client = common::connect_to_dir(repository.path()).await;

    let result = common::call(
        &client,
        "sara_update_item",
        json!({ "id": "SCEN-001", "relations": { "derives": ["SYSREQ-002"] } }),
    )
    .await;

    assert_ne!(result.is_error, Some(true), "{}", common::text(&result));
    assert_eq!(
        read(repository.path(), "SCEN-001.md"),
        SCENARIO.replace("  - \"SYSREQ-001\"", "  - \"SYSREQ-002\"")
    );

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_update_item_reports_links_declared_elsewhere() {
    let solution = "---\nid: \"SOL-001\"\ntype: solution\nname: \"Hub\"\nis_refined_by:\n  - \"UC-002\"\n---\n";
    let use_case = "---\nid: \"UC-002\"\ntype: use_case\nname: \"Scenes\"\n---\n";
    let repository = repository(&[("SOL-001.md", solution), ("UC-002.md", use_case)]);
    let client = common::connect_to_dir(repository.path()).await;

    let updated = common::structured(
        &client,
        "sara_update_item",
        json!({ "id": "UC-002", "relations": { "refines": [] } }),
    )
    .await;

    assert_eq!(
        updated["kept_links"],
        json!([{ "relation": "refines", "id": "SOL-001" }])
    );

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_update_item_requires_a_change() {
    let (_repository, client) = common::connect_to_copy("valid_graph").await;

    let result = common::call(&client, "sara_update_item", json!({ "id": "SYSREQ-001" })).await;

    assert_eq!(result.is_error, Some(true));

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_update_item_rejects_relation_the_item_cannot_hold() {
    let repository = common::copy_fixture("valid_graph");
    let before = read(repository.path(), "SYSREQ-001.md");
    let client = common::connect_to_dir(repository.path()).await;

    let result = common::call(
        &client,
        "sara_update_item",
        json!({ "id": "SYSREQ-001", "relations": { "satisfies": ["SOL-001"] } }),
    )
    .await;

    assert_eq!(result.is_error, Some(true));
    assert!(
        common::text(&result).contains("derives_from"),
        "{}",
        common::text(&result)
    );
    assert_eq!(read(repository.path(), "SYSREQ-001.md"), before);

    client.cancel().await.unwrap();
}

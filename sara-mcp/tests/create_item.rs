//! Tests of the `sara_create_item` tool.

mod common;

use std::fs;

use serde_json::{Value, json};

/// Arguments creating a software requirement in `file`.
fn software_requirement(file: &str) -> Value {
    json!({
        "type": "SWREQ",
        "file": file,
        "name": "Token refresh",
        "fields": { "specification": "The software SHALL refresh expired tokens." },
        "relations": { "derives_from": ["SYSARCH-001"] },
    })
}

#[tokio::test]
async fn test_create_item_continues_the_id_sequence() {
    let (repository, client) = common::connect_to_copy("valid_graph").await;
    let file = repository.path().join("SWREQ-002.md");

    let created = common::structured(
        &client,
        "sara_create_item",
        software_requirement(file.to_str().unwrap()),
    )
    .await;

    assert_eq!(created["id"], "SWREQ-002");
    assert_eq!(created["type"], "software_requirement");
    assert_eq!(created["created"], true);
    assert!(created.get("defaulted_fields").is_none(), "{created}");
    // Templates keep the line endings of the checkout: CRLF on Windows.
    let content = fs::read_to_string(&file).unwrap().replace("\r\n", "\n");
    assert!(
        content.contains("derives_from:\n  - \"SYSARCH-001\""),
        "{content}"
    );

    let validation = common::structured(&client, "sara_validate", json!({})).await;
    assert_eq!(validation["valid"], true, "{validation}");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_create_item_reports_defaulted_fields() {
    let (repository, client) = common::connect_to_copy("valid_graph").await;
    let file = repository.path().join("new.md");

    let created = common::structured(
        &client,
        "sara_create_item",
        json!({ "type": "software_requirement", "file": file.to_str().unwrap() }),
    )
    .await;

    assert_eq!(created["defaulted_fields"], json!(["specification"]));

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_create_item_takes_list_field_values() {
    let (repository, client) = common::connect_to_copy("valid_graph").await;
    let file = repository.path().join("ADR-001.md");

    let created = common::structured(
        &client,
        "sara_create_item",
        json!({
            "type": "ADR",
            "file": file.to_str().unwrap(),
            "fields": { "status": "proposed", "deciders": ["Alice", "Bob"] },
        }),
    )
    .await;

    assert!(created.get("defaulted_fields").is_none(), "{created}");
    let content = fs::read_to_string(&file).unwrap().replace("\r\n", "\n");
    assert!(
        content.contains("deciders:\n  - \"Alice\"\n  - \"Bob\""),
        "{content}"
    );

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_create_item_rejects_existing_item_file() {
    let (repository, client) = common::connect_to_copy("valid_graph").await;
    let file = repository.path().join("SYSREQ-001.md");
    let before = fs::read_to_string(&file).unwrap();

    let result = common::call(
        &client,
        "sara_create_item",
        software_requirement(file.to_str().unwrap()),
    )
    .await;
    let text = common::text(&result);

    assert_eq!(result.is_error, Some(true));
    assert!(text.contains("sara_update_item"), "{text}");
    assert_eq!(fs::read_to_string(&file).unwrap(), before);

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_create_item_rejects_path_outside_repositories() {
    let repository = common::copy_fixture("valid_graph");
    let elsewhere = tempfile::TempDir::new().unwrap();
    let client = common::connect_to_dir(repository.path()).await;
    let file = elsewhere.path().join("SWREQ-002.md");

    let result = common::call(
        &client,
        "sara_create_item",
        software_requirement(file.to_str().unwrap()),
    )
    .await;

    assert_eq!(result.is_error, Some(true));
    assert!(
        common::text(&result).contains("outside"),
        "{}",
        common::text(&result)
    );
    assert!(!file.exists());

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_create_item_rejects_unknown_field() {
    let (repository, client) = common::connect_to_copy("valid_graph").await;
    let file = repository.path().join("new.md");

    let result = common::call(
        &client,
        "sara_create_item",
        json!({
            "type": "SWREQ",
            "file": file.to_str().unwrap(),
            "fields": { "colour": "red" },
        }),
    )
    .await;
    let text = common::text(&result);

    assert_eq!(result.is_error, Some(true));
    assert!(
        text.contains("colour") && text.contains("specification"),
        "{text}"
    );
    assert!(!file.exists());

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_create_item_rejects_target_of_the_wrong_type() {
    let (repository, client) = common::connect_to_copy("valid_graph").await;
    let file = repository.path().join("new.md");

    let result = common::call(
        &client,
        "sara_create_item",
        json!({
            "type": "SWREQ",
            "file": file.to_str().unwrap(),
            "relations": { "derives_from": ["SOL-001"] },
        }),
    )
    .await;
    let text = common::text(&result);

    assert_eq!(result.is_error, Some(true));
    assert!(
        text.contains("SOL-001") && text.contains("system_architecture"),
        "{text}"
    );
    assert!(!file.exists());

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_create_item_rejects_unknown_target() {
    let (repository, client) = common::connect_to_copy("valid_graph").await;
    let file = repository.path().join("new.md");

    let result = common::call(
        &client,
        "sara_create_item",
        json!({
            "type": "SWREQ",
            "file": file.to_str().unwrap(),
            "relations": { "derives_from": ["SYSARCH-009"] },
        }),
    )
    .await;
    let text = common::text(&result);

    assert_eq!(result.is_error, Some(true));
    assert!(text.contains("SYSARCH-001"), "{text}");
    assert!(!file.exists());

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_concurrent_creations_get_distinct_ids() {
    let (repository, client) = common::connect_to_copy("valid_graph").await;
    let first = repository.path().join("first.md");
    let second = repository.path().join("second.md");

    let (a, b) = tokio::join!(
        common::structured(
            &client,
            "sara_create_item",
            software_requirement(first.to_str().unwrap())
        ),
        common::structured(
            &client,
            "sara_create_item",
            software_requirement(second.to_str().unwrap())
        ),
    );

    assert_ne!(a["id"], b["id"]);

    client.cancel().await.unwrap();
}

#[cfg(unix)]
#[tokio::test]
async fn test_create_item_refuses_dangling_symlink() {
    let repository = common::copy_fixture("valid_graph");
    let outside = tempfile::TempDir::new().unwrap();
    let target = outside.path().join("pwned.md");
    let link = repository.path().join("evil.md");
    std::os::unix::fs::symlink(&target, &link).unwrap();
    let client = common::connect_to_dir(repository.path()).await;

    let result = common::call(
        &client,
        "sara_create_item",
        software_requirement(link.to_str().unwrap()),
    )
    .await;

    assert_eq!(result.is_error, Some(true));
    assert!(!target.exists());

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_create_item_rejects_an_unreadable_id() {
    let (repository, client) = common::connect_to_copy("valid_graph").await;
    let file = repository.path().join("new.md");

    let result = common::call(
        &client,
        "sara_create_item",
        json!({ "type": "SOL", "file": file.to_str().unwrap(), "id": "SOL 9" }),
    )
    .await;

    assert_eq!(result.is_error, Some(true));
    assert!(
        common::text(&result).contains("SOL 9"),
        "{}",
        common::text(&result)
    );
    assert!(!file.exists());

    client.cancel().await.unwrap();
}

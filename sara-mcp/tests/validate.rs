//! Tests of the `sara_validate` tool.

mod common;

use serde_json::{Value, json};

/// Validates a fixture repository with the given arguments.
async fn validate(fixture: &str, arguments: Value) -> Value {
    let client = common::connect_to_fixture(fixture).await;
    let validation = common::structured(&client, "sara_validate", arguments).await;
    client.cancel().await.unwrap();
    validation
}

/// Returns the kinds of the issues of a validation result.
fn kinds(validation: &Value) -> Vec<&str> {
    validation["issues"]
        .as_array()
        .map(|issues| {
            issues
                .iter()
                .filter_map(|issue| issue["kind"].as_str())
                .collect()
        })
        .unwrap_or_default()
}

#[tokio::test]
async fn test_validate_accepts_valid_graph() {
    let validation = validate("valid_graph", json!({})).await;

    assert_eq!(validation["valid"], true);
    assert_eq!(validation["error_count"], 0);
    assert_eq!(validation["items_checked"], 7);
}

#[tokio::test]
async fn test_validate_reports_duplicate_ids() {
    let validation = validate("duplicates", json!({})).await;

    assert_eq!(validation["valid"], false);
    assert!(
        kinds(&validation).contains(&"DuplicateIdentifier"),
        "{validation}"
    );
}

#[tokio::test]
async fn test_validate_reports_broken_references() {
    let validation = validate("broken_refs", json!({})).await;

    assert_eq!(validation["valid"], false);
    assert!(
        kinds(&validation).contains(&"BrokenReference"),
        "{validation}"
    );
    assert_eq!(validation["issues"][0]["severity"], "error");
}

#[tokio::test]
async fn test_validate_reports_cycles() {
    let validation = validate("cycles", json!({})).await;

    assert!(
        kinds(&validation).contains(&"CircularReference"),
        "{validation}"
    );
}

#[tokio::test]
async fn test_validate_strict_turns_warnings_into_errors() {
    let lenient = validate("orphans", json!({})).await;
    assert_eq!(lenient["valid"], true);
    assert!(kinds(&lenient).contains(&"OrphanItem"), "{lenient}");
    assert_eq!(lenient["issues"][0]["severity"], "warning");

    let strict = validate("orphans", json!({ "strict": true })).await;
    assert_eq!(strict["valid"], false);
    assert_eq!(strict["strict"], true);
    assert_eq!(strict["issues"][0]["severity"], "error");
}

#[tokio::test]
async fn test_validate_reports_skipped_files() {
    let lenient = validate("corrupt_file", json!({})).await;
    assert_eq!(lenient["valid"], true);
    assert!(
        lenient["scan_warnings"][0]["path"]
            .as_str()
            .unwrap()
            .ends_with("CORRUPT-001.md"),
        "{lenient}"
    );

    let strict = validate("corrupt_file", json!({ "strict": true })).await;
    assert_eq!(strict["valid"], false);
}

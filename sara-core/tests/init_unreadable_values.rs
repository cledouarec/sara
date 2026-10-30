//! Verifies that initialization refuses to write a document sara could not
//! read back.
//!
//! Lives as a dedicated integration-test binary: it installs a custom model,
//! and the active schema is a process-wide `OnceLock`.

use sara_core::schema::builtin;
use sara_core::schema::{self, FieldDef, FieldType, Schema};
use sara_core::service::{FieldInput, InitError, InitOptions, InitService, TypeConfig};

/// Builds the built-in model where solutions declare an optional `due` date.
fn schema_with_due_date() -> Schema {
    let mut schema = Schema::builtin();
    let solution = schema
        .item_types
        .iter_mut()
        .find(|t| t.id == "solution")
        .expect("builtin schema defines solution");
    solution.fields.push(FieldDef {
        name: "due".to_string(),
        display_name: "Due".to_string(),
        field_type: FieldType::Date,
        required: false,
        placeholder: None,
    });
    schema
}

#[test]
fn test_init_refuses_a_value_that_breaks_the_frontmatter() {
    schema::install(schema_with_due_date()).expect("first installation");
    let dir = tempfile::tempdir().expect("temporary directory");
    let file = dir.path().join("SOL-001.md");
    // Dates are written unquoted, so this value ends the YAML mapping early.
    let config = TypeConfig::new(builtin::SOLUTION)
        .field("due", FieldInput::Text("2026-01-01\nbroken: [".to_string()));
    let opts = InitOptions::new(file.clone(), config)
        .with_id("SOL-001")
        .with_name("Hub");

    let result = InitService::new().init(None, &opts);

    assert!(
        matches!(result, Err(InitError::InvalidOption(_))),
        "{result:?}"
    );
    assert!(!file.exists());
}

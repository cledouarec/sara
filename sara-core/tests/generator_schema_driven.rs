//! Verifies that document generation follows the active [`Schema`] and the
//! installed template overrides.
//!
//! Lives as a dedicated integration-test binary so it owns its own process:
//! the active schema, the override store and the template registry are all
//! process-wide `OnceLock`s, so installing here cannot leak into sibling
//! tests.

use std::path::PathBuf;

use sara_core::generator::{self, OutputFormat, TemplateOverride};
use sara_core::model::{
    FieldValue, Item, ItemBuilder, ItemId, ItemType, Relationship, RelationshipType, SourceLocation,
};
use sara_core::schema::{self, FieldDef, FieldType, Schema};

/// Builds a schema based on the built-in default where the solution type
/// declares an extra optional `owner` text field.
fn schema_with_owner_field() -> Schema {
    let mut schema = Schema::builtin();
    let solution = schema
        .item_types
        .iter_mut()
        .find(|t| t.id == "solution")
        .expect("builtin schema defines solution");
    solution.fields.push(FieldDef {
        name: "owner".to_string(),
        display_name: "Owner".to_string(),
        field_type: FieldType::Text,
        required: false,
        placeholder: None,
    });
    schema
}

fn test_source() -> SourceLocation {
    SourceLocation {
        repository: PathBuf::from("/repo"),
        file_path: PathBuf::from("docs/test.md"),
        git_ref: None,
    }
}

fn build_item(id: &str, item_type: ItemType, name: &str) -> Item {
    ItemBuilder::new()
        .id(ItemId::new_unchecked(id))
        .item_type(item_type)
        .name(name)
        .source(test_source())
        .build()
        .expect("build test item")
}

/// Generation must render fields declared by the active schema and honor
/// installed `.tera` overrides, while untouched types keep their built-in
/// bodies.
///
/// Bundled into a single `#[test]` because the schema, override and registry
/// singletons accept one installation per process; splitting the assertions
/// would race against parallel test threads.
#[test]
fn generation_follows_active_schema_and_overrides() {
    schema::install(schema_with_owner_field()).expect("install schema once at start of test");
    generator::install_overrides(vec![TemplateOverride {
        type_id: "use_case".to_string(),
        source: "{% include \"frontmatter.tera\" %}\n\n# Custom Use Case: {{ name }}".to_string(),
    }])
    .expect("install overrides once at start of test");

    // A field added by the schema shows up in the generated frontmatter,
    // while the built-in document body for the type is untouched.
    let mut solution = build_item("SOL-001", ItemType::SOLUTION, "Test Solution");
    solution
        .attributes
        .insert("owner", FieldValue::Text("Alice".to_string()));
    let frontmatter = generator::generate_metadata(&solution, OutputFormat::Markdown);
    assert!(
        frontmatter.contains("owner: \"Alice\""),
        "schema-declared field must be rendered: {frontmatter}"
    );
    let document = generator::generate_document(&solution, OutputFormat::Markdown);
    assert!(document.contains("owner: \"Alice\""));
    assert!(document.contains("# Solution: Test Solution"));
    assert!(document.contains("## Goals & KPIs"));

    // The override replaces the built-in body and still renders the generic
    // frontmatter, including declared relations.
    let use_case = ItemBuilder::new()
        .id(ItemId::new_unchecked("UC-001"))
        .item_type(ItemType::USE_CASE)
        .name("Test Use Case")
        .source(test_source())
        .relationships(vec![Relationship::new(
            ItemId::new_unchecked("SOL-001"),
            RelationshipType::REFINES,
        )])
        .build()
        .expect("build use case");
    let document = generator::generate_document(&use_case, OutputFormat::Markdown);
    assert!(document.contains("# Custom Use Case: Test Use Case"));
    assert!(
        !document.contains("## Actor(s)"),
        "override must replace the built-in body"
    );
    assert!(document.contains("refines:"));
    assert!(document.contains("- \"SOL-001\""));

    // Types without overrides keep their built-in bodies.
    let scenario = build_item("SCEN-001", ItemType::SCENARIO, "Test Scenario");
    let document = generator::generate_document(&scenario, OutputFormat::Markdown);
    assert!(document.contains("# Scenario: Test Scenario"));
}

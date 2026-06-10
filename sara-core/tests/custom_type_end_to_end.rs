//! Verifies that a type defined only in a custom YAML schema behaves like a
//! built-in one across the whole pipeline: id resolution, parsing,
//! relationship validity, document generation and item initialization.
//!
//! Lives as a dedicated integration-test binary so it owns its own process:
//! the active schema and the template registry are process-wide `OnceLock`s,
//! so installing here cannot leak into sibling tests.

use std::path::Path;

use sara_core::generator::{self, OutputFormat};
use sara_core::model::{ItemType, RelationshipRules, RelationshipType};
use sara_core::parser::{InputFormat, parse_metadata};
use sara_core::schema::{self, Schema};
use sara_core::service::{InitOptions, InitService, TypeConfig, parse_item_type};

/// YAML definition of a type that has no built-in counterpart.
const CUSTOM_TYPE_YAML: &str = r#"- id: stakeholder_requirement
  display_name: Stakeholder Requirement
  prefix: STKREQ
  id_format: "{prefix}-{seq:03}"
  parent_types:
  - solution
  fields:
  - name: rationale
    display_name: Rationale
    field_type: text
    required: true
    placeholder: Explain why the stakeholder needs this.
  - name: review_date
    display_name: Review date
    field_type: date
  allowed_targets:
  - relation: refines
    targets:
    - solution
"#;

/// Builds a schema extending the built-in default with the
/// `stakeholder_requirement` type, going through YAML to exercise the public
/// loading path.
fn schema_with_custom_type() -> Schema {
    let yaml = Schema::builtin()
        .to_yaml()
        .expect("serialize builtin")
        .replace("\nrelations:", &format!("\n{CUSTOM_TYPE_YAML}relations:"));
    Schema::from_yaml_str(&yaml, Path::new("<test>")).expect("parse extended schema")
}

const CUSTOM_MD: &str = r#"---
id: "STKREQ-001"
type: stakeholder_requirement
name: "Operator overview"
refines:
  - "SOL-001"
rationale: "Operators need a single pane of glass."
review_date: "2026-06-01"
---
# Stakeholder Requirement: Operator overview
"#;

/// A custom-schema type must flow through id resolution, parsing, validity
/// checks, generation and initialization without any built-in counterpart.
///
/// Bundled into a single `#[test]` because the schema and template registry
/// singletons accept one installation per process.
#[test]
fn custom_type_flows_through_the_whole_pipeline() {
    schema::install(schema_with_custom_type()).expect("install once at start of test");

    // Id resolution and metadata.
    let custom = ItemType::from_id("stakeholder_requirement").expect("type known to schema");
    assert_eq!(custom.display_name(), "Stakeholder Requirement");
    assert_eq!(custom.prefix(), "STKREQ");
    assert_eq!(custom.generate_id(Some(4)), "STKREQ-004");
    assert!(custom.requires_refines());
    assert!(!custom.is_root());
    assert!(
        ItemType::all().contains(&custom),
        "all() must list schema-defined types"
    );

    // CLI-style type-name resolution, including the squashed and prefix
    // aliases, and the architecture_decision_record lookup.
    assert_eq!(parse_item_type("stakeholder_requirement"), Some(custom));
    assert_eq!(parse_item_type("stakeholderrequirement"), Some(custom));
    assert_eq!(parse_item_type("stkreq"), Some(custom));
    assert_eq!(
        parse_item_type("adr"),
        Some(ItemType::ARCHITECTURE_DECISION_RECORD)
    );

    // Relationship validity is derived from the schema declaration.
    assert!(RelationshipRules::is_valid_relationship(
        custom,
        ItemType::SOLUTION,
        RelationshipType::Refines,
    ));
    assert!(!RelationshipRules::is_valid_relationship(
        ItemType::SOLUTION,
        custom,
        RelationshipType::Refines,
    ));

    // Parsing fills the declared fields and the relationships.
    let item = parse_metadata(
        CUSTOM_MD,
        Path::new("docs/STKREQ-001.md"),
        Path::new("/repo"),
        InputFormat::Markdown,
    )
    .expect("parse custom-type document");
    assert_eq!(item.item_type, custom);
    let rationale = item
        .attributes
        .get("rationale")
        .and_then(|v| v.as_text())
        .expect("rationale text field");
    assert_eq!(rationale, "Operators need a single pane of glass.");
    let review_date = item
        .attributes
        .get("review_date")
        .and_then(|v| v.as_date())
        .expect("review_date date field");
    assert_eq!(review_date, "2026-06-01");
    let refines: Vec<_> = item.relationship_ids(RelationshipType::Refines).collect();
    assert_eq!(refines.len(), 1);

    // A required field missing from the frontmatter fails the build.
    let missing_rationale = CUSTOM_MD.replace(
        "rationale: \"Operators need a single pane of glass.\"\n",
        "",
    );
    assert!(
        parse_metadata(
            &missing_rationale,
            Path::new("bad.md"),
            Path::new("/repo"),
            InputFormat::Markdown,
        )
        .is_err()
    );

    // Generation falls back to the generic body and renders declared fields.
    let document = generator::generate_document(&item, OutputFormat::Markdown);
    assert!(document.contains("type: stakeholder_requirement"));
    assert!(document.contains("rationale: \"Operators need a single pane of glass.\""));
    assert!(document.contains("review_date: 2026-06-01"));
    assert!(document.contains("refines:"));
    assert!(document.contains("# Stakeholder Requirement: Operator overview"));
    assert!(document.contains("## Rationale"));

    // Initialization works with no input: the required field falls back to
    // its schema placeholder.
    let dir = tempfile::tempdir().expect("temp dir");
    let file = dir.path().join("STKREQ-002.md");
    let opts = InitOptions::new(file.clone(), TypeConfig::new(custom))
        .with_id("STKREQ-002")
        .with_name("Maintenance access");
    let result = InitService::new().init(&opts).expect("init custom type");
    assert_eq!(result.item_type, custom);
    assert!(result.needs_specification, "placeholder text was used");
    let content = std::fs::read_to_string(&file).expect("read generated file");
    assert!(content.contains("id: \"STKREQ-002\""));
    assert!(content.contains("rationale: \"Explain why the stakeholder needs this.\""));
    assert!(content.contains("# Stakeholder Requirement: Maintenance access"));
}

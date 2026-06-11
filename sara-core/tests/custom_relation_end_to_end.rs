//! Verifies that a relation defined only in a custom YAML schema behaves
//! like a built-in one: id resolution, inverse and direction, validity
//! matrix, parsing, graph traversal, generation and initialization.
//!
//! Lives as a dedicated integration-test binary so it owns its own process:
//! the active schema and the template registry are process-wide `OnceLock`s,
//! so installing here cannot leak into sibling tests.

use std::path::{Path, PathBuf};

use sara_core::generator::{self, OutputFormat};
use sara_core::graph::KnowledgeGraphBuilder;
use sara_core::model::{
    FieldValue, ItemBuilder, ItemId, ItemType, RelationshipRules, RelationshipType, SourceLocation,
};
use sara_core::parser::{InputFormat, parse_metadata};
use sara_core::schema::builtin;
use sara_core::schema::{self, Schema};
use sara_core::service::{EditOptions, EditService, InitOptions, InitService, TypeConfig};

/// YAML definition of a type whose traceability uses a custom relation.
const CUSTOM_TYPE_YAML: &str = "- id: test_case
  display_name: Test Case
  prefix: TC
  id_format: \"{prefix}-{seq:03}\"
  parent_types:
  - system_requirement
  fields: []
  allowed_targets:
  - relation: verifies
    targets:
    - system_requirement
";

/// YAML definition of the custom relation pair.
const CUSTOM_RELATIONS_YAML: &str = "- id: verifies
  display_name: Verifies
  inverse: is_verified_by
  direction: upstream
  primary: true
- id: is_verified_by
  display_name: Is verified by
  inverse: verifies
  direction: downstream
  primary: false
";

/// Builds a schema extending the built-in default with a `test_case` type
/// linked through a new `verifies`/`is_verified_by` relation pair, going
/// through YAML to exercise the public loading path.
fn schema_with_custom_relation() -> Schema {
    let mut yaml = Schema::builtin()
        .to_yaml()
        .expect("serialize builtin")
        .replace("\nrelations:", &format!("\n{CUSTOM_TYPE_YAML}relations:"));
    yaml.push_str(CUSTOM_RELATIONS_YAML);
    Schema::from_yaml_str(&yaml, Path::new("<test>")).expect("parse extended schema")
}

const CUSTOM_MD: &str = r#"---
id: "TC-001"
type: test_case
name: "Latency check"
verifies:
  - "SYSREQ-001"
---
# Test Case: Latency check
"#;

fn test_source() -> SourceLocation {
    SourceLocation {
        repository: PathBuf::from("/repo"),
        file_path: PathBuf::from("docs/test.md"),
        git_ref: None,
    }
}

/// A custom-schema relation must flow through inverse/direction resolution,
/// validity checks, parsing, traversal, generation and initialization.
///
/// Bundled into a single `#[test]` because the schema and template registry
/// singletons accept one installation per process.
#[test]
fn custom_relation_flows_through_the_whole_pipeline() {
    schema::install(schema_with_custom_relation()).expect("install once at start of test");

    // Id resolution, direction, inverse and primality come from the schema.
    let verifies = RelationshipType::from_id("verifies").expect("relation known to schema");
    let is_verified_by =
        RelationshipType::from_id("is_verified_by").expect("inverse known to schema");
    assert!(verifies.is_upstream());
    assert!(verifies.is_primary());
    assert!(is_verified_by.is_downstream());
    assert!(!is_verified_by.is_primary());
    assert_eq!(verifies.inverse(), is_verified_by);
    assert_eq!(is_verified_by.inverse(), verifies);
    assert!(
        RelationshipType::all().contains(&verifies),
        "all() must list schema-defined relations"
    );

    // The validity matrix is derived from the type's declaration.
    let test_case = ItemType::from_id("test_case").expect("type known to schema");
    assert!(RelationshipRules::is_valid_relationship(
        test_case,
        builtin::SYSTEM_REQUIREMENT,
        verifies,
    ));
    assert!(!RelationshipRules::is_valid_relationship(
        builtin::SYSTEM_REQUIREMENT,
        test_case,
        verifies,
    ));

    // Parsing reads the custom relation from the frontmatter.
    let item = parse_metadata(
        CUSTOM_MD,
        Path::new("docs/TC-001.md"),
        Path::new("/repo"),
        InputFormat::Markdown,
    )
    .expect("parse custom-relation document");
    let targets: Vec<_> = item.relationship_ids(verifies).collect();
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].as_str(), "SYSREQ-001");

    // The graph links both items through the custom relation.
    let requirement = ItemBuilder::new()
        .id(ItemId::new_unchecked("SYSREQ-001"))
        .item_type(builtin::SYSTEM_REQUIREMENT)
        .name("Latency budget")
        .source(test_source())
        .attribute(
            "specification",
            FieldValue::text("The system SHALL respond within 200ms."),
        )
        .build()
        .expect("build requirement");
    let graph = KnowledgeGraphBuilder::new()
        .add_items(vec![item.clone(), requirement])
        .build()
        .expect("build graph");
    let parents = graph.parents(&item.id);
    assert_eq!(parents.len(), 1, "verifies must act as an upstream link");
    assert_eq!(parents[0].id.as_str(), "SYSREQ-001");

    // Generation renders the custom relation in the frontmatter.
    let document = generator::generate_document(&item, OutputFormat::Markdown);
    assert!(document.contains("type: test_case"));
    assert!(document.contains("verifies:"));
    assert!(document.contains("- \"SYSREQ-001\""));
    assert!(document.contains("# Test Case: Latency check"));

    // Initialization accepts the custom relation as input.
    let dir = tempfile::tempdir().expect("temp dir");
    let file = dir.path().join("TC-002.md");
    let config = TypeConfig::new(test_case).relation("verifies", vec!["SYSREQ-001".to_string()]);
    let opts = InitOptions::new(file.clone(), config)
        .with_id("TC-002")
        .with_name("Throughput check");
    InitService::new().init(&opts).expect("init custom type");
    let content = std::fs::read_to_string(&file).expect("read generated file");
    assert!(content.contains("verifies:"));
    assert!(content.contains("- \"SYSREQ-001\""));

    // Editing replaces the custom relation's targets and rejects relations
    // the type does not declare.
    let service = EditService::new();
    let ctx = service.get_item_context(&item);
    assert_eq!(ctx.traceability.get(verifies), ["SYSREQ-001"]);

    let edit = EditOptions::new("TC-001").with_relation(verifies, vec!["SYSREQ-002".to_string()]);
    service
        .validate_options(&edit, test_case)
        .expect("verifies is declared by test_case");
    let merged = service.merge_values(edit, &ctx);
    let yaml = service.build_frontmatter_yaml("TC-001", test_case, &merged);
    assert!(yaml.contains("verifies:"));
    assert!(yaml.contains("- \"SYSREQ-002\""));
    assert!(!yaml.contains("SYSREQ-001"));

    let invalid =
        EditOptions::new("TC-001").with_relation(builtin::REFINES, vec!["SOL-001".to_string()]);
    assert!(
        service.validate_options(&invalid, test_case).is_err(),
        "test_case declares no refines relation"
    );
}

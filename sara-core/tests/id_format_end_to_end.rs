//! End-to-end coverage for custom identifier formats.
//!
//! Runs in its own process: the custom schema is installed once and stays
//! active for the whole test.

use std::path::{Path, PathBuf};

use sara_core::graph::KnowledgeGraphBuilder;
use sara_core::model::{Item, ItemBuilder, ItemId, ItemType, SourceLocation};
use sara_core::schema::{self, Schema};
use sara_core::validation;

const CUSTOM_SCHEMA: &str = r#"item_types:
- id: ticket
  display_name: Ticket
  prefix: TKT
  id_format: "{prefix}-{year}-{seq:02}"
  parent_types: []
  fields: []
  allowed_targets: []
- id: note
  display_name: Note
  prefix: NOTE
  id_format: "{prefix}-{uuid4}"
  parent_types: []
  fields: []
  allowed_targets: []
relations: []
"#;

fn make_item(id: &str, item_type: ItemType) -> Item {
    let source = SourceLocation::new(PathBuf::from("/test-repo"), format!("{id}.md"));
    ItemBuilder::new()
        .id(ItemId::new_unchecked(id))
        .item_type(item_type)
        .name(format!("Test {id}"))
        .source(source)
        .build()
        .expect("test item should build")
}

#[test]
fn custom_id_formats_drive_generation_and_suggestion() {
    let schema = Schema::from_yaml_str(CUSTOM_SCHEMA, Path::new("<test>")).unwrap();
    schema::install(schema).expect("install once at start of test");

    let ticket = ItemType::from_id("ticket").unwrap();
    let note = ItemType::from_id("note").unwrap();

    // The year is read back from the rendered id rather than recomputed,
    // keeping the test clock-free.
    let first = ticket.generate_id(None);
    let year = first
        .strip_prefix("TKT-")
        .unwrap()
        .split('-')
        .next()
        .unwrap();
    assert_eq!(first, format!("TKT-{year}-01"));
    assert_eq!(year.len(), 4);

    // The counter is scoped to the current year: the 1999 id is
    // shape-conformant but outside the scope, so it does not count.
    let graph = KnowledgeGraphBuilder::new()
        .add_item(make_item(&format!("TKT-{year}-07"), ticket))
        .add_item(make_item("TKT-1999-99", ticket))
        .build()
        .unwrap();
    assert_eq!(
        ticket.suggest_next_id(Some(&graph)),
        format!("TKT-{year}-08")
    );

    // A uuid-only format suggests a fresh unique id without scanning.
    let suggested = note.suggest_next_id(Some(&graph));
    assert!(suggested.starts_with("NOTE-"));
    assert_eq!(suggested.len(), "NOTE-".len() + 36);
    assert_ne!(suggested, note.suggest_next_id(Some(&graph)));

    // The check rule accepts other-period ids and flags shape mismatches.
    let note_id = note.suggest_next_id(None);
    let checked = KnowledgeGraphBuilder::new()
        .add_item(make_item("TKT-1999-03", ticket))
        .add_item(make_item("TKT-BAD", ticket))
        .add_item(make_item(&note_id, note))
        .build()
        .unwrap();
    let report = validation::validate(&checked, false);
    let warnings: Vec<String> = report.warnings().iter().map(|w| w.to_string()).collect();
    assert!(
        warnings
            .iter()
            .any(|w| { w.contains("TKT-BAD") && w.contains("does not match the id_format") }),
        "got: {warnings:?}"
    );
    assert!(
        !warnings.iter().any(|w| w.contains("TKT-1999-03")),
        "got: {warnings:?}"
    );
    assert!(
        !warnings.iter().any(|w| w.contains(&note_id)),
        "got: {warnings:?}"
    );
}

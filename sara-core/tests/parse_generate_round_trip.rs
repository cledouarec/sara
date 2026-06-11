//! Verifies that a generated document parses back into an equivalent item:
//! every value the generator writes into the frontmatter must survive the
//! parser unchanged.

use std::path::{Path, PathBuf};

use sara_core::generator::{self, OutputFormat};
use sara_core::model::{
    FieldValue, Item, ItemBuilder, ItemId, Relationship, RelationshipType, SourceLocation,
};
use sara_core::parser::{InputFormat, parse_metadata};
use sara_core::schema::builtin;

/// Builds an item exercising every frontmatter shape: a description, an enum
/// field, a list field, and relationships of two different relations.
fn representative_item() -> Item {
    let source = SourceLocation {
        repository: PathBuf::from("/repo"),
        file_path: PathBuf::from("docs/ADR-042.md"),
        git_ref: None,
    };

    ItemBuilder::new()
        .id(ItemId::new_unchecked("ADR-042"))
        .item_type(builtin::ARCHITECTURE_DECISION_RECORD)
        .name("Adopt an Event-Driven Architecture")
        .description("Decision to decouple services through events")
        .source(source)
        .attribute("status", FieldValue::Enum("accepted".to_string()))
        .attribute(
            "deciders",
            FieldValue::text_list(["Alice Smith", "Bob Jones"]),
        )
        .relationships(vec![
            Relationship::new(ItemId::new_unchecked("SYSARCH-001"), builtin::JUSTIFIES),
            Relationship::new(ItemId::new_unchecked("SWDD-001"), builtin::JUSTIFIES),
            Relationship::new(ItemId::new_unchecked("ADR-001"), builtin::SUPERSEDES),
        ])
        .build()
        .expect("representative item builds")
}

/// Returns the targets of one relation as plain ID strings.
fn targets(item: &Item, relation: RelationshipType) -> Vec<&str> {
    item.relationship_ids(relation)
        .map(ItemId::as_str)
        .collect()
}

#[test]
fn generated_document_parses_back_into_an_equivalent_item() {
    let original = representative_item();

    let document = generator::generate_document(&original, OutputFormat::Markdown);
    let round_tripped = parse_metadata(
        &document,
        Path::new("docs/ADR-042.md"),
        Path::new("/repo"),
        InputFormat::Markdown,
    )
    .expect("generated document parses");

    assert_eq!(round_tripped.id, original.id);
    assert_eq!(round_tripped.item_type, original.item_type);
    assert_eq!(round_tripped.name, original.name);
    assert_eq!(round_tripped.description, original.description);

    // The parser collects relationships in the relation declaration order of
    // the active schema, not in the order the item was built with, so the
    // flat relationship lists may legitimately differ in order. Comparing
    // the targets per relation (plus the total count) asserts the same
    // information survived the round trip.
    assert_eq!(
        round_tripped.relationships.len(),
        original.relationships.len()
    );
    for relation in [builtin::JUSTIFIES, builtin::SUPERSEDES] {
        assert_eq!(
            targets(&round_tripped, relation),
            targets(&original, relation),
            "targets of `{}` must survive the round trip",
            relation.as_str()
        );
    }

    assert_eq!(round_tripped.attributes.len(), original.attributes.len());
    for (name, value) in original.attributes.iter() {
        assert_eq!(
            round_tripped.attributes.get(name),
            Some(value),
            "attribute `{name}` must survive the round trip"
        );
    }
}

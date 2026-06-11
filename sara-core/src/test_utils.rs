//! Test utilities and fixtures for sara-core tests.
//!
//! This module provides common test helpers to reduce duplication
//! across test modules.

use std::path::PathBuf;

use crate::model::{FieldValue, Item, ItemBuilder, ItemId, ItemType, Relationship, SourceLocation};
use crate::schema::builtin;

/// Fills the required fields declared by the item's type with deterministic
/// test values, so fixtures build for any type of the active schema.
fn with_required_defaults(mut builder: ItemBuilder, item_type: ItemType) -> ItemBuilder {
    for field in item_type.declared_fields().iter().filter(|f| f.required) {
        builder = match field.name.as_str() {
            "specification" => builder.attribute(
                &field.name,
                FieldValue::text("The system SHALL meet this test specification"),
            ),
            "status" => builder.attribute(&field.name, FieldValue::Enum("proposed".to_string())),
            "deciders" => builder.attribute(&field.name, FieldValue::text_list(["Test Decider"])),
            _ => builder,
        };
    }
    builder
}

/// Creates a test item with the given ID and type.
///
/// Automatically sets required fields based on item type:
/// - Adds a default specification for requirement types
/// - Adds default status and deciders for ADR types
///
/// # Examples
///
/// ```ignore
/// use sara_core::test_utils::create_test_item;
/// use sara_core::model::ItemType;
///
/// let solution = create_test_item("SOL-001", builtin::SOLUTION);
/// let requirement = create_test_item("SYSREQ-001", builtin::SYSTEM_REQUIREMENT);
/// ```
#[must_use]
pub fn create_test_item(id: &str, item_type: ItemType) -> Item {
    create_test_item_with_name(id, item_type, &format!("Test {id}"))
}

/// Creates a test item with a custom name.
///
/// Use this when you need to control the item name, such as testing
/// name changes in diffs or edit operations.
///
/// # Examples
///
/// ```ignore
/// use sara_core::test_utils::create_test_item_with_name;
/// use sara_core::model::ItemType;
///
/// let solution = create_test_item_with_name("SOL-001", builtin::SOLUTION, "My Custom Name");
/// ```
#[must_use]
pub fn create_test_item_with_name(id: &str, item_type: ItemType, name: &str) -> Item {
    let source = SourceLocation::new(PathBuf::from("/test-repo"), format!("{id}.md"));
    let builder = ItemBuilder::new()
        .id(ItemId::new_unchecked(id))
        .item_type(item_type)
        .name(name)
        .source(source);

    with_required_defaults(builder, item_type)
        .build()
        .expect("Test item should build successfully")
}

/// Creates a test item with relationships.
///
/// # Examples
///
/// ```ignore
/// use sara_core::test_utils::create_test_item_with_relationships;
/// use sara_core::model::{ItemType, Relationship, RelationshipType, ItemId};
///
/// let use_case = create_test_item_with_relationships(
///     "UC-001",
///     builtin::USE_CASE,
///     vec![Relationship::new(ItemId::new_unchecked("SOL-001"), builtin::REFINES)],
/// );
/// ```
#[must_use]
pub fn create_test_item_with_relationships(
    id: &str,
    item_type: ItemType,
    relationships: Vec<Relationship>,
) -> Item {
    let source = SourceLocation::new(PathBuf::from("/test-repo"), format!("{id}.md"));
    let builder = ItemBuilder::new()
        .id(ItemId::new_unchecked(id))
        .item_type(item_type)
        .name(format!("Test {id}"))
        .source(source)
        .relationships(relationships);

    with_required_defaults(builder, item_type)
        .build()
        .expect("Test item should build successfully")
}

/// Creates a test ADR item with optional justifies and supersedes references.
///
/// # Examples
///
/// ```ignore
/// use sara_core::test_utils::create_test_adr;
///
/// let adr = create_test_adr("ADR-001", &["SYSARCH-001"], &[]);
/// let superseding_adr = create_test_adr("ADR-002", &[], &["ADR-001"]);
/// ```
#[must_use]
pub fn create_test_adr(id: &str, justifies: &[&str], supersedes: &[&str]) -> Item {
    let source = SourceLocation::new(PathBuf::from("/test-repo"), format!("{id}.md"));
    let relationships: Vec<Relationship> = justifies
        .iter()
        .map(|s| Relationship::new(ItemId::new_unchecked(*s), builtin::JUSTIFIES))
        .chain(
            supersedes
                .iter()
                .map(|s| Relationship::new(ItemId::new_unchecked(*s), builtin::SUPERSEDES)),
        )
        .collect();

    let builder = ItemBuilder::new()
        .id(ItemId::new_unchecked(id))
        .item_type(builtin::ARCHITECTURE_DECISION_RECORD)
        .name(format!("Test {id}"))
        .source(source)
        .relationships(relationships);

    with_required_defaults(builder, builtin::ARCHITECTURE_DECISION_RECORD)
        .build()
        .expect("Test ADR should build successfully")
}

/// Creates a test item with a custom file path.
///
/// Useful for testing duplicate detection where file paths matter.
#[must_use]
pub fn create_test_item_at(id: &str, item_type: ItemType, file_path: &str) -> Item {
    let source = SourceLocation::new(PathBuf::from("/test-repo"), PathBuf::from(file_path));
    let builder = ItemBuilder::new()
        .id(ItemId::new_unchecked(id))
        .item_type(item_type)
        .name(format!("Test {id}"))
        .source(source);

    with_required_defaults(builder, item_type)
        .build()
        .expect("Test item should build successfully")
}

/// Creates a simple graph fixture with Solution -> UseCase -> Scenario chain.
///
/// Returns a vector of items that can be added to a graph.
#[must_use]
pub fn create_simple_hierarchy() -> Vec<Item> {
    vec![
        create_test_item("SOL-001", builtin::SOLUTION),
        create_test_item_with_relationships(
            "UC-001",
            builtin::USE_CASE,
            vec![Relationship::new(
                ItemId::new_unchecked("SOL-001"),
                builtin::REFINES,
            )],
        ),
        create_test_item_with_relationships(
            "SCEN-001",
            builtin::SCENARIO,
            vec![Relationship::new(
                ItemId::new_unchecked("UC-001"),
                builtin::REFINES,
            )],
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_item_solution() {
        let item = create_test_item("SOL-001", builtin::SOLUTION);
        assert_eq!(item.id.as_str(), "SOL-001");
        assert_eq!(item.item_type, builtin::SOLUTION);
    }

    #[test]
    fn test_create_test_item_requirement() {
        let item = create_test_item("SYSREQ-001", builtin::SYSTEM_REQUIREMENT);
        assert_eq!(item.id.as_str(), "SYSREQ-001");
        assert!(item.attributes.get("specification").is_some());
    }

    #[test]
    fn test_create_test_adr() {
        let item = create_test_adr("ADR-001", &["SYSARCH-001"], &["ADR-000"]);
        assert_eq!(item.id.as_str(), "ADR-001");
        assert_eq!(
            item.attributes.get("status"),
            Some(&FieldValue::Enum("proposed".to_string()))
        );
        let justifies: Vec<_> = item.relationship_ids(builtin::JUSTIFIES).collect();
        assert_eq!(justifies.len(), 1);
        assert_eq!(item.relationship_ids(builtin::SUPERSEDES).count(), 1);
    }

    #[test]
    fn test_create_simple_hierarchy() {
        let items = create_simple_hierarchy();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].item_type, builtin::SOLUTION);
        assert_eq!(items[1].item_type, builtin::USE_CASE);
        assert_eq!(items[2].item_type, builtin::SCENARIO);
    }
}

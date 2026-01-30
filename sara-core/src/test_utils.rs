//! Test utilities and fixtures for sara-core tests.
//!
//! This module provides common test helpers to reduce duplication
//! across test modules.

use std::path::PathBuf;

use crate::model::{
    AdrStatus, Item, ItemAttributes, ItemBuilder, ItemId, ItemType, Relationship, RelationshipType,
    SourceLocation,
};

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
/// let solution = create_test_item("SOL-001", ItemType::Solution);
/// let requirement = create_test_item("SYSREQ-001", ItemType::SystemRequirement);
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
/// let solution = create_test_item_with_name("SOL-001", ItemType::Solution, "My Custom Name");
/// ```
#[must_use]
pub fn create_test_item_with_name(id: &str, item_type: ItemType, name: &str) -> Item {
    let source = SourceLocation::new(PathBuf::from("/test-repo"), format!("{id}.md"));
    ItemBuilder::new()
        .id(ItemId::new_unchecked(id))
        .item_type(item_type)
        .name(name)
        .source(source)
        .attributes(default_attributes_for_type(item_type))
        .build()
        .expect("Test item should build successfully")
}

/// Creates a test item with the specified relationships.
///
/// # Examples
///
/// ```ignore
/// use sara_core::test_utils::create_test_item_with_relationships;
/// use sara_core::model::{ItemType, ItemId, RelationshipType};
///
/// let use_case = create_test_item_with_relationships(
///     "UC-001",
///     ItemType::UseCase,
///     vec![(ItemId::new_unchecked("SOL-001"), RelationshipType::Refines)],
/// );
/// ```
#[must_use]
pub fn create_test_item_with_relationships(
    id: &str,
    item_type: ItemType,
    relationships: Vec<(ItemId, RelationshipType)>,
) -> Item {
    let source = SourceLocation::new(PathBuf::from("/test-repo"), format!("{id}.md"));
    let rels: Vec<Relationship> = relationships
        .into_iter()
        .map(|(target, rel_type)| Relationship::new(target, rel_type))
        .collect();

    ItemBuilder::new()
        .id(ItemId::new_unchecked(id))
        .item_type(item_type)
        .name(format!("Test {id}"))
        .source(source)
        .relationships(rels)
        .attributes(default_attributes_for_type(item_type))
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
        .map(|s| Relationship::new(ItemId::new_unchecked(*s), RelationshipType::Justifies))
        .collect();

    ItemBuilder::new()
        .id(ItemId::new_unchecked(id))
        .item_type(ItemType::ArchitectureDecisionRecord)
        .name(format!("Test {id}"))
        .source(source)
        .relationships(relationships)
        .attributes(ItemAttributes::Adr {
            status: AdrStatus::Proposed,
            deciders: vec!["Test Decider".to_string()],
            supersedes: supersedes
                .iter()
                .map(|s| ItemId::new_unchecked(*s))
                .collect(),
        })
        .build()
        .expect("Test ADR should build successfully")
}

/// Creates a test item with a custom file path.
///
/// Useful for testing duplicate detection where file paths matter.
#[must_use]
pub fn create_test_item_at(id: &str, item_type: ItemType, file_path: &str) -> Item {
    let source = SourceLocation::new(PathBuf::from("/test-repo"), PathBuf::from(file_path));
    ItemBuilder::new()
        .id(ItemId::new_unchecked(id))
        .item_type(item_type)
        .name(format!("Test {id}"))
        .source(source)
        .attributes(default_attributes_for_type(item_type))
        .build()
        .expect("Test item should build successfully")
}

/// Creates a simple graph fixture with Solution -> UseCase -> Scenario chain.
///
/// Returns a vector of items that can be added to a graph.
#[must_use]
pub fn create_simple_hierarchy() -> Vec<Item> {
    vec![
        create_test_item("SOL-001", ItemType::Solution),
        create_test_item_with_relationships(
            "UC-001",
            ItemType::UseCase,
            vec![(ItemId::new_unchecked("SOL-001"), RelationshipType::Refines)],
        ),
        create_test_item_with_relationships(
            "SCEN-001",
            ItemType::Scenario,
            vec![(ItemId::new_unchecked("UC-001"), RelationshipType::Refines)],
        ),
    ]
}

/// Creates default attributes for a given item type with test values.
fn default_attributes_for_type(item_type: ItemType) -> ItemAttributes {
    match item_type {
        ItemType::Solution => ItemAttributes::Solution,
        ItemType::UseCase => ItemAttributes::UseCase,
        ItemType::Scenario => ItemAttributes::Scenario,
        ItemType::SoftwareDetailedDesign => ItemAttributes::SoftwareDetailedDesign,
        ItemType::HardwareDetailedDesign => ItemAttributes::HardwareDetailedDesign,
        ItemType::SystemArchitecture => ItemAttributes::SystemArchitecture { platform: None },
        ItemType::SystemRequirement => ItemAttributes::SystemRequirement {
            specification: "Test specification".to_string(),
            depends_on: Vec::new(),
        },
        ItemType::SoftwareRequirement => ItemAttributes::SoftwareRequirement {
            specification: "Test specification".to_string(),
            depends_on: Vec::new(),
        },
        ItemType::HardwareRequirement => ItemAttributes::HardwareRequirement {
            specification: "Test specification".to_string(),
            depends_on: Vec::new(),
        },
        ItemType::ArchitectureDecisionRecord => ItemAttributes::Adr {
            status: AdrStatus::Proposed,
            deciders: vec!["Test Decider".to_string()],
            supersedes: Vec::new(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_item_solution() {
        let item = create_test_item("SOL-001", ItemType::Solution);
        assert_eq!(item.id.as_str(), "SOL-001");
        assert_eq!(item.item_type, ItemType::Solution);
    }

    #[test]
    fn test_create_test_item_requirement() {
        let item = create_test_item("SYSREQ-001", ItemType::SystemRequirement);
        assert_eq!(item.id.as_str(), "SYSREQ-001");
        assert!(item.attributes.specification().is_some());
    }

    #[test]
    fn test_create_test_adr() {
        let item = create_test_adr("ADR-001", &["SYSARCH-001"], &["ADR-000"]);
        assert_eq!(item.id.as_str(), "ADR-001");
        assert_eq!(item.attributes.status(), Some(AdrStatus::Proposed));
        let justifies: Vec<_> = item
            .relationship_ids(RelationshipType::Justifies)
            .collect();
        assert_eq!(justifies.len(), 1);
        assert_eq!(item.attributes.supersedes().len(), 1);
    }

    #[test]
    fn test_create_simple_hierarchy() {
        let items = create_simple_hierarchy();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].item_type, ItemType::Solution);
        assert_eq!(items[1].item_type, ItemType::UseCase);
        assert_eq!(items[2].item_type, ItemType::Scenario);
    }
}

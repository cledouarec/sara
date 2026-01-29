//! Test utilities and fixtures for sara-core tests.
//!
//! This module provides common test helpers to reduce duplication
//! across test modules.

use std::path::PathBuf;

use crate::model::{
    AdrStatus, DownstreamRefs, Item, ItemBuilder, ItemId, ItemType, SourceLocation, UpstreamRefs,
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
    let mut builder = ItemBuilder::new()
        .id(ItemId::new_unchecked(id))
        .item_type(item_type)
        .name(name)
        .source(source);

    // Add required fields based on item type
    if item_type.requires_specification() {
        builder = builder.specification("Test specification");
    }

    if item_type.requires_deciders() {
        builder = builder
            .status(AdrStatus::Proposed)
            .deciders(vec!["Test Decider".to_string()]);
    }

    builder
        .build()
        .expect("Test item should build successfully")
}

/// Creates a test item with upstream references.
///
/// # Examples
///
/// ```ignore
/// use sara_core::test_utils::create_test_item_with_upstream;
/// use sara_core::model::{ItemType, UpstreamRefs, ItemId};
///
/// let use_case = create_test_item_with_upstream(
///     "UC-001",
///     ItemType::UseCase,
///     UpstreamRefs {
///         refines: vec![ItemId::new_unchecked("SOL-001")],
///         ..Default::default()
///     },
/// );
/// ```
#[must_use]
pub fn create_test_item_with_upstream(
    id: &str,
    item_type: ItemType,
    upstream: UpstreamRefs,
) -> Item {
    let source = SourceLocation::new(PathBuf::from("/test-repo"), format!("{id}.md"));
    let mut builder = ItemBuilder::new()
        .id(ItemId::new_unchecked(id))
        .item_type(item_type)
        .name(format!("Test {id}"))
        .source(source)
        .upstream(upstream);

    // Add required fields based on item type
    if item_type.requires_specification() {
        builder = builder.specification("Test specification");
    }

    if item_type.requires_deciders() {
        builder = builder
            .status(AdrStatus::Proposed)
            .deciders(vec!["Test Decider".to_string()]);
    }

    builder
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
    let upstream = UpstreamRefs {
        justifies: justifies
            .iter()
            .map(|s| ItemId::new_unchecked(*s))
            .collect(),
        ..Default::default()
    };

    ItemBuilder::new()
        .id(ItemId::new_unchecked(id))
        .item_type(ItemType::ArchitectureDecisionRecord)
        .name(format!("Test {id}"))
        .source(source)
        .upstream(upstream)
        .status(AdrStatus::Proposed)
        .deciders(vec!["Test Decider".to_string()])
        .supersedes_all(
            supersedes
                .iter()
                .map(|s| ItemId::new_unchecked(*s))
                .collect(),
        )
        .build()
        .expect("Test ADR should build successfully")
}

/// Creates a test item with a custom file path.
///
/// Useful for testing duplicate detection where file paths matter.
#[must_use]
pub fn create_test_item_at(id: &str, item_type: ItemType, file_path: &str) -> Item {
    let source = SourceLocation::new(PathBuf::from("/test-repo"), PathBuf::from(file_path));
    let mut builder = ItemBuilder::new()
        .id(ItemId::new_unchecked(id))
        .item_type(item_type)
        .name(format!("Test {id}"))
        .source(source);

    if item_type.requires_specification() {
        builder = builder.specification("Test specification");
    }

    if item_type.requires_deciders() {
        builder = builder
            .status(AdrStatus::Proposed)
            .deciders(vec!["Test Decider".to_string()]);
    }

    builder
        .build()
        .expect("Test item should build successfully")
}

/// Creates a test item with both upstream and downstream references.
///
/// Useful for testing relationship validation.
#[must_use]
pub fn create_test_item_with_refs(
    id: &str,
    item_type: ItemType,
    upstream: UpstreamRefs,
    downstream: DownstreamRefs,
) -> Item {
    let source = SourceLocation::new(PathBuf::from("/test-repo"), format!("{id}.md"));
    let mut builder = ItemBuilder::new()
        .id(ItemId::new_unchecked(id))
        .item_type(item_type)
        .name(format!("Test {id}"))
        .source(source)
        .upstream(upstream)
        .downstream(downstream);

    if item_type.requires_specification() {
        builder = builder.specification("Test specification");
    }

    if item_type.requires_deciders() {
        builder = builder
            .status(AdrStatus::Proposed)
            .deciders(vec!["Test Decider".to_string()]);
    }

    builder
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
        create_test_item_with_upstream(
            "UC-001",
            ItemType::UseCase,
            UpstreamRefs {
                refines: vec![ItemId::new_unchecked("SOL-001")],
                ..Default::default()
            },
        ),
        create_test_item_with_upstream(
            "SCEN-001",
            ItemType::Scenario,
            UpstreamRefs {
                refines: vec![ItemId::new_unchecked("UC-001")],
                ..Default::default()
            },
        ),
    ]
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
        assert_eq!(item.upstream.justifies.len(), 1);
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

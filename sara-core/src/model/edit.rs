//! Edit command types for updating document metadata.
//!
//! Provides types for FR-054 through FR-066 (Edit Command).

use std::path::PathBuf;

use indexmap::IndexMap;

use super::{Item, RelationshipType};

/// Summary of changes made during an edit operation.
#[derive(Debug, Clone)]
pub struct EditSummary {
    /// The ID of the edited item.
    pub item_id: String,
    /// Path to the modified file.
    pub file_path: PathBuf,
    /// List of field changes applied.
    pub changes: Vec<FieldChange>,
}

impl EditSummary {
    /// Returns true if any changes were actually made.
    pub fn has_changes(&self) -> bool {
        self.changes.iter().any(|c| c.is_changed())
    }

    /// Returns only the fields that were actually changed.
    pub fn actual_changes(&self) -> Vec<&FieldChange> {
        self.changes.iter().filter(|c| c.is_changed()).collect()
    }
}

/// A single field change in an edit operation.
#[derive(Debug, Clone)]
pub struct FieldChange {
    /// Display label of the changed field or relation.
    pub field: String,
    /// Previous value (for display in diff).
    pub old_value: String,
    /// New value (for display in diff).
    pub new_value: String,
}

impl FieldChange {
    /// Creates a new field change record.
    pub fn new(
        field: impl Into<String>,
        old_value: impl Into<String>,
        new_value: impl Into<String>,
    ) -> Self {
        Self {
            field: field.into(),
            old_value: old_value.into(),
            new_value: new_value.into(),
        }
    }

    /// Returns true if the value actually changed.
    pub fn is_changed(&self) -> bool {
        self.old_value != self.new_value
    }
}

/// Relation targets keyed by relation, as plain ID strings.
///
/// Suitable for CLI input, interactive prompts, and frontmatter rebuilding.
/// Entries keep their insertion order; [`Self::from_item`] inserts them in
/// the declaration order of the active schema, so downstream output remains
/// stable.
#[derive(Debug, Default, Clone)]
pub struct TraceabilityLinks {
    links: IndexMap<RelationshipType, Vec<String>>,
}

impl TraceabilityLinks {
    /// Creates an empty set of links.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the target IDs recorded for a relation.
    #[must_use]
    pub fn get(&self, relation: RelationshipType) -> &[String] {
        self.links.get(&relation).map_or(&[], Vec::as_slice)
    }

    /// Replaces the target IDs of a relation.
    pub fn set(&mut self, relation: RelationshipType, ids: Vec<String>) {
        self.links.insert(relation, ids);
    }

    /// Appends target IDs to a relation, keeping existing ones.
    pub fn extend(&mut self, relation: RelationshipType, ids: impl IntoIterator<Item = String>) {
        self.links.entry(relation).or_default().extend(ids);
    }

    /// Returns `(relation, target IDs)` pairs in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = (RelationshipType, &[String])> {
        self.links.iter().map(|(rel, ids)| (*rel, ids.as_slice()))
    }

    /// Returns true if no relation has any target.
    pub fn is_empty(&self) -> bool {
        self.links.values().all(Vec::is_empty)
    }

    /// Collects the item's targets for every relation its type declares.
    pub fn from_item(item: &Item) -> Self {
        let mut links = Self::new();
        for relation in item.item_type.declared_relations() {
            links.set(
                relation,
                item.relationship_ids(relation)
                    .map(|id| id.as_str().to_string())
                    .collect(),
            );
        }
        links
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::schema::builtin;

    use crate::model::{ItemId, Relationship};
    use crate::test_utils::create_test_item_with_relationships;

    #[test]
    fn test_field_change_is_changed() {
        let changed = FieldChange::new("Name", "Old", "New");
        assert!(changed.is_changed());

        let unchanged = FieldChange::new("Name", "Same", "Same");
        assert!(!unchanged.is_changed());
    }

    #[test]
    fn test_edit_summary_has_changes() {
        let summary = EditSummary {
            item_id: "SREQ-001".to_string(),
            file_path: PathBuf::from("test.md"),
            changes: vec![
                FieldChange::new("Name", "Old", "New"),
                FieldChange::new("Description", "Same", "Same"),
            ],
        };
        assert!(summary.has_changes());
        assert_eq!(summary.actual_changes().len(), 1);
    }

    #[test]
    fn test_edit_summary_no_changes() {
        let summary = EditSummary {
            item_id: "SREQ-001".to_string(),
            file_path: PathBuf::from("test.md"),
            changes: vec![FieldChange::new("Name", "Same", "Same")],
        };
        assert!(!summary.has_changes());
        assert_eq!(summary.actual_changes().len(), 0);
    }

    #[test]
    fn test_traceability_links_set_get() {
        let mut links = TraceabilityLinks::new();
        assert!(links.is_empty());
        assert!(links.get(builtin::REFINES).is_empty());

        links.set(builtin::REFINES, vec!["SOL-001".to_string()]);
        links.extend(builtin::REFINES, ["SOL-002".to_string()]);
        assert!(!links.is_empty());
        assert_eq!(links.get(builtin::REFINES), ["SOL-001", "SOL-002"]);
    }

    #[test]
    fn test_traceability_links_from_item() {
        let item = create_test_item_with_relationships(
            "UC-001",
            builtin::USE_CASE,
            vec![Relationship::new(
                ItemId::new_unchecked("SOL-001"),
                builtin::REFINES,
            )],
        );

        let links = TraceabilityLinks::from_item(&item);
        assert_eq!(links.get(builtin::REFINES), ["SOL-001"]);
    }
}

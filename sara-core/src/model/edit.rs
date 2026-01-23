//! Edit command types for updating document metadata.
//!
//! Provides types for FR-054 through FR-066 (Edit Command).

use std::path::PathBuf;

use super::FieldName;

/// Fields to update via CLI flags (non-interactive mode).
///
/// Used for non-interactive editing where the user specifies which
/// fields to update via command-line flags (FR-057, FR-058).
#[derive(Debug, Default, Clone)]
pub struct EditUpdates {
    /// New name for the item.
    pub name: Option<String>,
    /// New description for the item.
    pub description: Option<String>,
    /// New refines links (for UseCase, Scenario).
    pub refines: Option<Vec<String>>,
    /// New derives_from links (for requirements).
    pub derives_from: Option<Vec<String>>,
    /// New satisfies links (for architectures, designs).
    pub satisfies: Option<Vec<String>>,
    /// New depends_on links (peer dependencies for requirements).
    pub depends_on: Option<Vec<String>>,
    /// New specification (for requirement types).
    pub specification: Option<String>,
    /// New platform (for SystemArchitecture).
    pub platform: Option<String>,
}

impl EditUpdates {
    /// Returns true if any field is set (triggers non-interactive mode).
    pub fn has_updates(&self) -> bool {
        self.name.is_some()
            || self.description.is_some()
            || self.refines.is_some()
            || self.derives_from.is_some()
            || self.satisfies.is_some()
            || self.depends_on.is_some()
            || self.specification.is_some()
            || self.platform.is_some()
    }
}

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
    /// The field that was changed.
    pub field: FieldName,
    /// Previous value (for display in diff).
    pub old_value: String,
    /// New value (for display in diff).
    pub new_value: String,
}

impl FieldChange {
    /// Creates a new field change record.
    pub fn new(
        field: FieldName,
        old_value: impl Into<String>,
        new_value: impl Into<String>,
    ) -> Self {
        Self {
            field,
            old_value: old_value.into(),
            new_value: new_value.into(),
        }
    }

    /// Returns true if the value actually changed.
    pub fn is_changed(&self) -> bool {
        self.old_value != self.new_value
    }
}

/// Traceability links as string IDs (for user input and editing).
///
/// This struct represents traceability links using plain strings,
/// suitable for CLI input, interactive prompts, and serialization.
/// Use `UpstreamRefs` for the validated graph model.
#[derive(Debug, Default, Clone)]
pub struct TraceabilityLinks {
    /// Items this item refines (for UseCase, Scenario).
    pub refines: Vec<String>,
    /// Items this item derives from (for requirements).
    pub derives_from: Vec<String>,
    /// Items this item satisfies (for architectures, designs).
    pub satisfies: Vec<String>,
    /// Peer dependencies (for requirement types).
    pub depends_on: Vec<String>,
}

impl TraceabilityLinks {
    /// Returns true if all traceability fields are empty.
    pub fn is_empty(&self) -> bool {
        self.refines.is_empty()
            && self.derives_from.is_empty()
            && self.satisfies.is_empty()
            && self.depends_on.is_empty()
    }

    /// Creates from an Item's upstream references.
    pub fn from_upstream(upstream: &super::UpstreamRefs) -> Self {
        Self {
            refines: upstream
                .refines
                .iter()
                .map(|id| id.as_str().to_string())
                .collect(),
            derives_from: upstream
                .derives_from
                .iter()
                .map(|id| id.as_str().to_string())
                .collect(),
            satisfies: upstream
                .satisfies
                .iter()
                .map(|id| id.as_str().to_string())
                .collect(),
            depends_on: Vec::new(),
        }
    }

    /// Creates from an Item's upstream references and peer dependencies.
    pub fn from_item(item: &super::Item) -> Self {
        Self {
            refines: item
                .upstream
                .refines
                .iter()
                .map(|id| id.as_str().to_string())
                .collect(),
            derives_from: item
                .upstream
                .derives_from
                .iter()
                .map(|id| id.as_str().to_string())
                .collect(),
            satisfies: item
                .upstream
                .satisfies
                .iter()
                .map(|id| id.as_str().to_string())
                .collect(),
            depends_on: item
                .attributes
                .depends_on
                .iter()
                .map(|id| id.as_str().to_string())
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edit_updates_has_updates_empty() {
        let updates = EditUpdates::default();
        assert!(!updates.has_updates());
    }

    #[test]
    fn test_edit_updates_has_updates_name() {
        let updates = EditUpdates {
            name: Some("New Name".to_string()),
            ..Default::default()
        };
        assert!(updates.has_updates());
    }

    #[test]
    fn test_edit_updates_has_updates_traceability() {
        let updates = EditUpdates {
            derives_from: Some(vec!["SCEN-001".to_string()]),
            ..Default::default()
        };
        assert!(updates.has_updates());
    }

    #[test]
    fn test_field_change_is_changed() {
        let changed = FieldChange::new(FieldName::Name, "Old", "New");
        assert!(changed.is_changed());

        let unchanged = FieldChange::new(FieldName::Name, "Same", "Same");
        assert!(!unchanged.is_changed());
    }

    #[test]
    fn test_edit_summary_has_changes() {
        let summary = EditSummary {
            item_id: "SREQ-001".to_string(),
            file_path: PathBuf::from("test.md"),
            changes: vec![
                FieldChange::new(FieldName::Name, "Old", "New"),
                FieldChange::new(FieldName::Description, "Same", "Same"),
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
            changes: vec![FieldChange::new(FieldName::Name, "Same", "Same")],
        };
        assert!(!summary.has_changes());
        assert_eq!(summary.actual_changes().len(), 0);
    }
}

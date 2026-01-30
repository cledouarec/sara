//! Edit service for modifying existing document metadata.
//!
//! Provides functionality for editing requirement items (FR-054 through FR-066).

use std::fs;
use std::path::PathBuf;

use serde::Serialize;

use crate::error::EditError;
use crate::graph::KnowledgeGraph;
use crate::model::{FieldChange, FieldName, Item, ItemType, TraceabilityLinks};
use crate::parser::update_frontmatter;
use crate::query::lookup_item_or_suggest;

// ============================================================================
// Options
// ============================================================================

/// Options for editing an item.
#[derive(Debug, Clone, Default)]
pub struct EditOptions {
    /// The item ID to edit.
    pub item_id: String,
    /// New name (if provided).
    pub name: Option<String>,
    /// New description (if provided).
    pub description: Option<String>,
    /// New refines references (if provided).
    pub refines: Option<Vec<String>>,
    /// New derives_from references (if provided).
    pub derives_from: Option<Vec<String>>,
    /// New satisfies references (if provided).
    pub satisfies: Option<Vec<String>>,
    /// New depends_on references (if provided).
    pub depends_on: Option<Vec<String>>,
    /// New justifies references (if provided, for ADRs).
    pub justifies: Option<Vec<String>>,
    /// New specification (if provided).
    pub specification: Option<String>,
    /// New platform (if provided).
    pub platform: Option<String>,
}

impl EditOptions {
    /// Creates new edit options for the given item ID.
    pub fn new(item_id: impl Into<String>) -> Self {
        Self {
            item_id: item_id.into(),
            ..Default::default()
        }
    }

    /// Sets the name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the name if provided.
    pub fn maybe_name(mut self, name: Option<String>) -> Self {
        self.name = name;
        self
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the description if provided.
    pub fn maybe_description(mut self, description: Option<String>) -> Self {
        self.description = description;
        self
    }

    /// Sets the refines references.
    pub fn with_refines(mut self, refines: Vec<String>) -> Self {
        self.refines = Some(refines);
        self
    }

    /// Sets the refines references if provided.
    pub fn maybe_refines(mut self, refines: Option<Vec<String>>) -> Self {
        self.refines = refines;
        self
    }

    /// Sets the derives_from references.
    pub fn with_derives_from(mut self, derives_from: Vec<String>) -> Self {
        self.derives_from = Some(derives_from);
        self
    }

    /// Sets the derives_from references if provided.
    pub fn maybe_derives_from(mut self, derives_from: Option<Vec<String>>) -> Self {
        self.derives_from = derives_from;
        self
    }

    /// Sets the satisfies references.
    pub fn with_satisfies(mut self, satisfies: Vec<String>) -> Self {
        self.satisfies = Some(satisfies);
        self
    }

    /// Sets the satisfies references if provided.
    pub fn maybe_satisfies(mut self, satisfies: Option<Vec<String>>) -> Self {
        self.satisfies = satisfies;
        self
    }

    /// Sets the depends_on references.
    pub fn with_depends_on(mut self, depends_on: Vec<String>) -> Self {
        self.depends_on = Some(depends_on);
        self
    }

    /// Sets the depends_on references if provided.
    pub fn maybe_depends_on(mut self, depends_on: Option<Vec<String>>) -> Self {
        self.depends_on = depends_on;
        self
    }

    /// Sets the justifies references.
    pub fn with_justifies(mut self, justifies: Vec<String>) -> Self {
        self.justifies = Some(justifies);
        self
    }

    /// Sets the justifies references if provided.
    pub fn maybe_justifies(mut self, justifies: Option<Vec<String>>) -> Self {
        self.justifies = justifies;
        self
    }

    /// Sets the specification.
    pub fn with_specification(mut self, specification: impl Into<String>) -> Self {
        self.specification = Some(specification.into());
        self
    }

    /// Sets the specification if provided.
    pub fn maybe_specification(mut self, specification: Option<String>) -> Self {
        self.specification = specification;
        self
    }

    /// Sets the platform.
    pub fn with_platform(mut self, platform: impl Into<String>) -> Self {
        self.platform = Some(platform.into());
        self
    }

    /// Sets the platform if provided.
    pub fn maybe_platform(mut self, platform: Option<String>) -> Self {
        self.platform = platform;
        self
    }

    /// Returns true if any modification was requested.
    #[must_use]
    pub fn has_updates(&self) -> bool {
        self.name.is_some()
            || self.description.is_some()
            || self.refines.is_some()
            || self.derives_from.is_some()
            || self.satisfies.is_some()
            || self.depends_on.is_some()
            || self.justifies.is_some()
            || self.specification.is_some()
            || self.platform.is_some()
    }
}

// ============================================================================
// Result and Context Types
// ============================================================================

/// Result of a successful edit operation.
#[derive(Debug)]
pub struct EditResult {
    /// The item ID that was edited.
    pub item_id: String,
    /// The file path that was modified.
    pub file_path: PathBuf,
    /// The changes that were applied.
    pub changes: Vec<FieldChange>,
}

impl EditResult {
    /// Returns true if any changes were made.
    #[must_use]
    pub fn has_changes(&self) -> bool {
        !self.changes.is_empty()
    }

    /// Returns the number of changes.
    #[must_use]
    pub fn change_count(&self) -> usize {
        self.changes.len()
    }
}

/// Context for the item being edited.
#[derive(Debug, Clone)]
pub struct ItemContext {
    /// The item ID.
    pub id: String,
    /// The item type.
    pub item_type: ItemType,
    /// The current name.
    pub name: String,
    /// The current description.
    pub description: Option<String>,
    /// The current specification.
    pub specification: Option<String>,
    /// The current platform.
    pub platform: Option<String>,
    /// The current traceability links.
    pub traceability: TraceabilityLinks,
    /// The file path.
    pub file_path: PathBuf,
}

impl ItemContext {
    /// Creates a context from an Item.
    pub fn from_item(item: &Item) -> Self {
        Self {
            id: item.id.as_str().to_string(),
            item_type: item.item_type,
            name: item.name.clone(),
            description: item.description.clone(),
            specification: item.attributes.specification().map(ToOwned::to_owned),
            platform: item.attributes.platform().map(ToOwned::to_owned),
            traceability: TraceabilityLinks::from_item(item),
            file_path: item.source.full_path(),
        }
    }
}

/// Values to apply during editing.
#[derive(Debug, Clone)]
pub struct EditedValues {
    /// The name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Optional specification.
    pub specification: Option<String>,
    /// Optional platform.
    pub platform: Option<String>,
    /// Traceability links.
    pub traceability: TraceabilityLinks,
}

impl EditedValues {
    /// Creates new edited values.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            specification: None,
            platform: None,
            traceability: TraceabilityLinks::default(),
        }
    }

    /// Sets the description.
    pub fn with_description(mut self, description: Option<String>) -> Self {
        self.description = description;
        self
    }

    /// Sets the specification.
    pub fn with_specification(mut self, specification: Option<String>) -> Self {
        self.specification = specification;
        self
    }

    /// Sets the platform.
    pub fn with_platform(mut self, platform: Option<String>) -> Self {
        self.platform = platform;
        self
    }

    /// Sets the traceability links.
    pub fn with_traceability(mut self, traceability: TraceabilityLinks) -> Self {
        self.traceability = traceability;
        self
    }
}

// ============================================================================
// Public Functions
// ============================================================================

/// Gets the item context for editing.
///
/// Looks up the item by ID and returns its context for interactive editing.
///
/// # Errors
///
/// Returns `EditError::ItemNotFound` with suggestions if the item doesn't exist.
pub fn get_item_for_edit(graph: &KnowledgeGraph, item_id: &str) -> Result<ItemContext, EditError> {
    let item = lookup_item_or_suggest(graph, item_id)?;
    Ok(ItemContext::from_item(item))
}

/// Validates edit options against the item type.
fn validate_options(opts: &EditOptions, item_type: ItemType) -> Result<(), EditError> {
    if opts.specification.is_some() && !item_type.requires_specification() {
        return Err(EditError::Validation(format!(
            "--specification is only valid for requirement types, not {}",
            item_type.display_name()
        )));
    }

    if opts.platform.is_some() && item_type != ItemType::SystemArchitecture {
        return Err(EditError::Validation(
            "--platform is only valid for System Architecture items".to_string(),
        ));
    }

    Ok(())
}

/// Merges edit options with current item values.
fn merge_values(opts: &EditOptions, current: &ItemContext) -> EditedValues {
    EditedValues {
        name: opts.name.clone().unwrap_or_else(|| current.name.clone()),
        description: opts
            .description
            .clone()
            .or_else(|| current.description.clone()),
        specification: opts
            .specification
            .clone()
            .or_else(|| current.specification.clone()),
        platform: opts.platform.clone().or_else(|| current.platform.clone()),
        traceability: TraceabilityLinks {
            refines: opts
                .refines
                .clone()
                .unwrap_or_else(|| current.traceability.refines.clone()),
            derives_from: opts
                .derives_from
                .clone()
                .unwrap_or_else(|| current.traceability.derives_from.clone()),
            satisfies: opts
                .satisfies
                .clone()
                .unwrap_or_else(|| current.traceability.satisfies.clone()),
            depends_on: opts
                .depends_on
                .clone()
                .unwrap_or_else(|| current.traceability.depends_on.clone()),
            justifies: opts
                .justifies
                .clone()
                .unwrap_or_else(|| current.traceability.justifies.clone()),
        },
    }
}

/// Builds a change summary comparing old and new values.
///
/// Only includes fields that have actually changed.
pub fn build_change_summary(old: &ItemContext, new: &EditedValues) -> Vec<FieldChange> {
    let mut changes = Vec::new();

    // Only add name change if actually different
    if old.name != new.name {
        changes.push(FieldChange::new(FieldName::Name, &old.name, &new.name));
    }

    // Only add description change if actually different
    if old.description != new.description {
        changes.push(FieldChange::new(
            FieldName::Description,
            old.description.as_deref().unwrap_or("(none)"),
            new.description.as_deref().unwrap_or("(none)"),
        ));
    }

    // Traceability changes (only if different)
    add_traceability_change_if_different(
        &mut changes,
        FieldName::Refines,
        &old.traceability.refines,
        &new.traceability.refines,
    );
    add_traceability_change_if_different(
        &mut changes,
        FieldName::DerivesFrom,
        &old.traceability.derives_from,
        &new.traceability.derives_from,
    );
    add_traceability_change_if_different(
        &mut changes,
        FieldName::Satisfies,
        &old.traceability.satisfies,
        &new.traceability.satisfies,
    );

    // Type-specific (only if different)
    if old.specification != new.specification {
        changes.push(FieldChange::new(
            FieldName::Specification,
            old.specification.as_deref().unwrap_or("(none)"),
            new.specification.as_deref().unwrap_or("(none)"),
        ));
    }

    if old.platform != new.platform {
        changes.push(FieldChange::new(
            FieldName::Platform,
            old.platform.as_deref().unwrap_or("(none)"),
            new.platform.as_deref().unwrap_or("(none)"),
        ));
    }

    changes
}

/// Applies changes to the file.
///
/// # Errors
///
/// Returns error if file read/write fails.
pub fn apply_changes(
    item_id: &str,
    item_type: ItemType,
    new_values: &EditedValues,
    file_path: &PathBuf,
) -> Result<(), EditError> {
    let content = fs::read_to_string(file_path).map_err(|e| EditError::IoError(e.to_string()))?;
    let new_yaml = build_frontmatter_yaml(item_id, item_type, new_values);
    let updated_content = update_frontmatter(&content, &new_yaml);
    fs::write(file_path, updated_content).map_err(|e| EditError::IoError(e.to_string()))?;
    Ok(())
}

/// Performs a non-interactive edit operation.
///
/// # Errors
///
/// Returns error if item lookup, validation, or file update fails.
pub fn edit_item(graph: &KnowledgeGraph, opts: &EditOptions) -> Result<EditResult, EditError> {
    let item_ctx = get_item_for_edit(graph, &opts.item_id)?;
    validate_options(opts, item_ctx.item_type)?;
    let new_values = merge_values(opts, &item_ctx);
    let changes = build_change_summary(&item_ctx, &new_values);

    apply_changes(
        &item_ctx.id,
        item_ctx.item_type,
        &new_values,
        &item_ctx.file_path,
    )?;

    Ok(EditResult {
        item_id: item_ctx.id,
        file_path: item_ctx.file_path,
        changes,
    })
}

// ============================================================================
// Private Functions
// ============================================================================

/// Adds a traceability field change only if values differ.
fn add_traceability_change_if_different(
    changes: &mut Vec<FieldChange>,
    field: FieldName,
    old: &[String],
    new: &[String],
) {
    // Skip if both are the same (including both empty)
    if old == new {
        return;
    }

    let old_str = if old.is_empty() {
        "(none)".to_string()
    } else {
        old.join(", ")
    };
    let new_str = if new.is_empty() {
        "(none)".to_string()
    } else {
        new.join(", ")
    };

    changes.push(FieldChange::new(field, &old_str, &new_str));
}

/// Frontmatter structure for YAML serialization.
#[derive(Serialize)]
struct Frontmatter<'a> {
    id: &'a str,
    #[serde(rename = "type")]
    item_type: &'a str,
    name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    refines: Vec<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    derives_from: Vec<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    satisfies: Vec<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    specification: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    platform: Option<&'a str>,
}

/// Builds YAML frontmatter string from edit values.
fn build_frontmatter_yaml(item_id: &str, item_type: ItemType, values: &EditedValues) -> String {
    let frontmatter = Frontmatter {
        id: item_id,
        item_type: item_type.as_str(),
        name: &values.name,
        description: values.description.as_deref(),
        refines: values
            .traceability
            .refines
            .iter()
            .map(String::as_str)
            .collect(),
        derives_from: values
            .traceability
            .derives_from
            .iter()
            .map(String::as_str)
            .collect(),
        satisfies: values
            .traceability
            .satisfies
            .iter()
            .map(String::as_str)
            .collect(),
        specification: values.specification.as_deref(),
        platform: values.platform.as_deref(),
    };

    serde_yaml::to_string(&frontmatter).unwrap_or_else(|e| {
        // Fallback to a minimal valid YAML if serialization fails
        tracing::warn!("Failed to serialize frontmatter: {}", e);
        format!(
            "id: \"{}\"\ntype: {}\nname: \"{}\"\n",
            item_id,
            item_type.as_str(),
            values.name
        )
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::create_test_item_with_name;

    #[test]
    fn test_edit_options_has_updates() {
        let opts = EditOptions::new("SOL-001");
        assert!(!opts.has_updates());

        let opts_with_name = EditOptions::new("SOL-001").with_name("New Name");
        assert!(opts_with_name.has_updates());
    }

    #[test]
    fn test_item_context_from_item() {
        let item = create_test_item_with_name("SOL-001", ItemType::Solution, "Test Solution");
        let ctx = ItemContext::from_item(&item);

        assert_eq!(ctx.id, "SOL-001");
        assert_eq!(ctx.name, "Test Solution");
        assert_eq!(ctx.item_type, ItemType::Solution);
    }

    #[test]
    fn test_validate_options_specification() {
        // Valid: specification on requirement type
        let opts = EditOptions::new("SYSREQ-001").with_specification("new spec");
        assert!(validate_options(&opts, ItemType::SystemRequirement).is_ok());

        // Invalid: specification on solution type
        let opts = EditOptions::new("SOL-001").with_specification("new spec");
        assert!(validate_options(&opts, ItemType::Solution).is_err());
    }

    #[test]
    fn test_validate_options_platform() {
        // Valid: platform on system architecture
        let opts = EditOptions::new("SYSARCH-001").with_platform("AWS");
        assert!(validate_options(&opts, ItemType::SystemArchitecture).is_ok());

        // Invalid: platform on solution
        let opts = EditOptions::new("SOL-001").with_platform("AWS");
        assert!(validate_options(&opts, ItemType::Solution).is_err());
    }

    #[test]
    fn test_merge_values() {
        let current = ItemContext {
            id: "SOL-001".to_string(),
            item_type: ItemType::Solution,
            name: "Old Name".to_string(),
            description: Some("Old Description".to_string()),
            specification: None,
            platform: None,
            traceability: TraceabilityLinks::default(),
            file_path: PathBuf::from("/test.md"),
        };

        let opts = EditOptions::new("SOL-001").with_name("New Name");

        let merged = merge_values(&opts, &current);

        assert_eq!(merged.name, "New Name");
        assert_eq!(merged.description, Some("Old Description".to_string()));
    }

    #[test]
    fn test_build_change_summary() {
        let old = ItemContext {
            id: "SOL-001".to_string(),
            item_type: ItemType::Solution,
            name: "Old Name".to_string(),
            description: None,
            specification: None,
            platform: None,
            traceability: TraceabilityLinks::default(),
            file_path: PathBuf::from("/test.md"),
        };

        let new = EditedValues::new("New Name");

        let changes = build_change_summary(&old, &new);

        let name_change = changes.iter().find(|c| c.field == FieldName::Name).unwrap();
        assert!(name_change.is_changed());
        assert_eq!(name_change.old_value, "Old Name");
        assert_eq!(name_change.new_value, "New Name");
    }

    #[test]
    fn test_build_frontmatter_yaml() {
        let values = EditedValues::new("Test Solution")
            .with_description(Some("A test solution".to_string()));

        let yaml = build_frontmatter_yaml("SOL-001", ItemType::Solution, &values);

        // serde_yaml may quote or not quote depending on content
        assert!(yaml.contains("id:") && yaml.contains("SOL-001"));
        assert!(yaml.contains("type: solution"));
        assert!(yaml.contains("name:") && yaml.contains("Test Solution"));
        assert!(yaml.contains("description:") && yaml.contains("A test solution"));
    }
}

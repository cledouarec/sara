//! Edit service for modifying existing document metadata.
//!
//! Provides functionality for editing requirement items (FR-054 through FR-066).

use std::fs;
use std::path::PathBuf;

use indexmap::IndexMap;

use crate::error::SaraError;
use crate::generator::{self, OutputFormat};
use crate::graph::KnowledgeGraph;
use crate::model::{
    FieldChange, Item, ItemAttributes, ItemBuilder, ItemId, ItemType, RelationshipType,
    SourceLocation, TraceabilityLinks,
};
use crate::parser::update_frontmatter;
use crate::schema::FieldType;

use super::FieldInput;
use super::init::init_field_value;

/// Options for editing an item.
#[derive(Debug, Clone, Default)]
pub struct EditOptions {
    /// The item ID to edit.
    pub item_id: String,
    /// New name (if provided).
    pub name: Option<String>,
    /// New description (if provided).
    pub description: Option<String>,
    /// New relation targets, keyed by relation; a present entry replaces the
    /// item's existing targets for that relation.
    pub relations: IndexMap<RelationshipType, Vec<String>>,
    /// New field values, keyed by field name; a present entry replaces the
    /// item's current value for that field.
    pub fields: IndexMap<String, FieldInput>,
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

    /// Replaces the targets of a relation.
    pub fn with_relation(mut self, relation: RelationshipType, ids: Vec<String>) -> Self {
        self.relations.insert(relation, ids);
        self
    }

    /// Replaces the targets of a relation if provided.
    pub fn maybe_relation(self, relation: RelationshipType, ids: Option<Vec<String>>) -> Self {
        match ids {
            Some(ids) => self.with_relation(relation, ids),
            None => self,
        }
    }

    /// Replaces the value of a field.
    pub fn with_field(mut self, name: impl Into<String>, input: FieldInput) -> Self {
        self.fields.insert(name.into(), input);
        self
    }

    /// Replaces the value of a text field if provided.
    pub fn maybe_text_field(self, name: impl Into<String>, value: Option<String>) -> Self {
        match value {
            Some(value) => self.with_field(name, FieldInput::Text(value)),
            None => self,
        }
    }

    /// Returns true if any modification was requested.
    pub fn has_updates(&self) -> bool {
        self.name.is_some()
            || self.description.is_some()
            || !self.relations.is_empty()
            || !self.fields.is_empty()
    }
}

/// Values to apply during editing.
#[derive(Debug, Clone)]
pub struct EditedValues {
    /// The name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Traceability links.
    pub traceability: TraceabilityLinks,
    /// The item's attributes after the edit: the current ones with the
    /// edited fields overlaid; fields outside the edit pass through as-is.
    pub attributes: ItemAttributes,
}

impl EditedValues {
    /// Creates new edited values.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            traceability: TraceabilityLinks::default(),
            attributes: ItemAttributes::new(),
        }
    }

    /// Sets the description.
    pub fn with_description(mut self, description: Option<String>) -> Self {
        self.description = description;
        self
    }

    /// Sets the traceability links.
    pub fn with_traceability(mut self, traceability: TraceabilityLinks) -> Self {
        self.traceability = traceability;
        self
    }

    /// Sets the attributes carried over from the edited item.
    pub fn with_attributes(mut self, attributes: ItemAttributes) -> Self {
        self.attributes = attributes;
        self
    }
}

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
    pub fn has_changes(&self) -> bool {
        !self.changes.is_empty()
    }

    /// Returns the number of changes.
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
    /// The current traceability links.
    pub traceability: TraceabilityLinks,
    /// The current attributes.
    pub attributes: ItemAttributes,
    /// The file path.
    pub file_path: PathBuf,
}

impl ItemContext {
    /// Creates a context from an `Item`.
    pub fn from_item(item: &Item) -> Self {
        Self {
            id: item.id.as_str().to_string(),
            item_type: item.item_type,
            name: item.name.clone(),
            description: item.description.clone(),
            traceability: TraceabilityLinks::from_item(item),
            attributes: item.attributes.clone(),
            file_path: item.source.full_path(),
        }
    }
}

/// Service for editing requirement items.
#[derive(Debug, Default)]
pub struct EditService;

impl EditService {
    /// Creates a new edit service.
    pub fn new() -> Self {
        Self
    }

    /// Looks up an item by ID with fuzzy suggestions on failure.
    pub fn lookup_item<'a>(
        &self,
        graph: &'a KnowledgeGraph,
        item_id: &str,
    ) -> Result<&'a Item, SaraError> {
        graph.lookup_or_suggest(item_id)
    }

    /// Gets the context for an item.
    pub fn get_item_context(&self, item: &Item) -> ItemContext {
        ItemContext::from_item(item)
    }

    /// Validates edit options against the item type.
    pub fn validate_options(
        &self,
        opts: &EditOptions,
        item_type: ItemType,
    ) -> Result<(), SaraError> {
        for (name, input) in &opts.fields {
            let Some(field) = item_type.declared_field(name) else {
                return Err(SaraError::EditFailed(format!(
                    "--{} is not a field declared by {}",
                    name.replace('_', "-"),
                    item_type.display_name()
                )));
            };
            if let FieldType::Enum { values } = &field.field_type
                && let FieldInput::Text(value) = input
                && !values.contains(value)
            {
                return Err(SaraError::EditFailed(format!(
                    "invalid value `{value}` for --{}, expected one of: {}",
                    name.replace('_', "-"),
                    values.join(", ")
                )));
            }
        }

        let declared = item_type.declared_relations();
        for relation in opts.relations.keys() {
            if !declared.contains(relation) {
                return Err(SaraError::EditFailed(format!(
                    "--{} is not a relation declared by {}",
                    relation.as_str().replace('_', "-"),
                    item_type.display_name()
                )));
            }
        }

        Ok(())
    }

    /// Merges edit options with current item values.
    pub fn merge_values(&self, opts: &EditOptions, current: &ItemContext) -> EditedValues {
        let mut traceability = current.traceability.clone();
        for (relation, ids) in &opts.relations {
            traceability.set(*relation, ids.clone());
        }

        let mut attributes = current.attributes.clone();
        for (name, input) in &opts.fields {
            if let Some(field) = current.item_type.declared_field(name)
                && let Some(value) = init_field_value(field, Some(input))
            {
                attributes.insert(name.clone(), value);
            }
        }

        EditedValues {
            name: opts.name.clone().unwrap_or_else(|| current.name.clone()),
            description: opts
                .description
                .clone()
                .or_else(|| current.description.clone()),
            traceability,
            attributes,
        }
    }

    /// Builds a change summary comparing old and new values.
    pub fn build_change_summary(&self, old: &ItemContext, new: &EditedValues) -> Vec<FieldChange> {
        let mut changes = Vec::new();

        changes.push(FieldChange::new("Name", &old.name, &new.name));
        changes.push(FieldChange::new(
            "Description",
            old.description.as_deref().unwrap_or("(none)"),
            new.description.as_deref().unwrap_or("(none)"),
        ));

        // Traceability changes
        for (relation, new_ids) in new.traceability.iter() {
            self.add_traceability_change(
                &mut changes,
                relation.display_name(),
                old.traceability.get(relation),
                new_ids,
            );
        }

        // Declared field changes
        for field in old.item_type.declared_fields() {
            let old_value = old.attributes.get(&field.name);
            let new_value = new.attributes.get(&field.name);
            if old_value.is_none() && new_value.is_none() {
                continue;
            }
            changes.push(FieldChange::new(
                &field.display_name,
                old_value.map_or("(none)".to_string(), ToString::to_string),
                new_value.map_or("(none)".to_string(), ToString::to_string),
            ));
        }

        changes
    }

    /// Adds a traceability field change if values differ.
    fn add_traceability_change(
        &self,
        changes: &mut Vec<FieldChange>,
        field: &str,
        old: &[String],
        new: &[String],
    ) {
        if old.is_empty() && new.is_empty() {
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

    /// Applies changes to the file.
    pub fn apply_changes(
        &self,
        item_id: &str,
        item_type: ItemType,
        new_values: &EditedValues,
        file_path: &PathBuf,
    ) -> Result<(), SaraError> {
        let content =
            fs::read_to_string(file_path).map_err(|e| SaraError::EditFailed(e.to_string()))?;
        let new_yaml = self.build_frontmatter_yaml(item_id, item_type, new_values);
        let updated_content = update_frontmatter(&content, &new_yaml);
        fs::write(file_path, updated_content).map_err(|e| SaraError::EditFailed(e.to_string()))?;
        Ok(())
    }

    /// Builds YAML frontmatter string from edit values.
    pub fn build_frontmatter_yaml(
        &self,
        item_id: &str,
        item_type: ItemType,
        values: &EditedValues,
    ) -> String {
        let item = self.build_item_from_values(item_id, item_type, values);
        generator::generate_metadata(&item, OutputFormat::Markdown)
    }

    /// Builds a temporary `Item` from edit values for frontmatter generation.
    ///
    /// Attributes are taken from the edited values as-is; any required field
    /// still missing falls back to its schema placeholder so the item always
    /// builds.
    fn build_item_from_values(
        &self,
        item_id: &str,
        item_type: ItemType,
        values: &EditedValues,
    ) -> Item {
        let source = SourceLocation {
            repository: PathBuf::new(),
            file_path: PathBuf::from("edit.md"),
            git_ref: None,
        };

        let mut builder = ItemBuilder::new()
            .id(ItemId::new_unchecked(item_id))
            .item_type(item_type)
            .name(&values.name)
            .source(source);

        if let Some(ref desc) = values.description {
            builder = builder.description(desc);
        }

        let mut rels = Vec::new();
        for (relation, ids) in values.traceability.iter() {
            rels.extend(super::ids_to_relationships(ids, relation));
        }
        builder = builder.relationships(rels);

        let mut attributes = values.attributes.clone();
        for field in item_type.declared_fields() {
            if field.required
                && attributes.get(&field.name).is_none()
                && let Some(value) = init_field_value(field, None)
            {
                attributes.insert(field.name.clone(), value);
            }
        }
        for (name, value) in attributes.iter() {
            builder = builder.attribute(name.clone(), value.clone());
        }

        builder.build().expect("Failed to build item for edit")
    }

    /// Performs a non-interactive edit operation.
    pub fn edit(
        &self,
        graph: &KnowledgeGraph,
        opts: &EditOptions,
    ) -> Result<EditResult, SaraError> {
        // Look up the item
        let item = self.lookup_item(graph, &opts.item_id)?;
        let item_ctx = self.get_item_context(item);

        // Validate options
        self.validate_options(opts, item_ctx.item_type)?;

        // Merge values
        let new_values = self.merge_values(opts, &item_ctx);

        // Build change summary
        let changes: Vec<FieldChange> = self
            .build_change_summary(&item_ctx, &new_values)
            .into_iter()
            .filter(|c| c.is_changed())
            .collect();

        // Apply changes
        self.apply_changes(
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
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::schema::builtin;
    use crate::test_utils::{create_test_item, create_test_item_with_name};

    #[test]
    fn test_edit_options_has_updates() {
        let opts = EditOptions::new("SOL-001");
        assert!(!opts.has_updates());

        let opts_with_name = EditOptions::new("SOL-001").with_name("New Name");
        assert!(opts_with_name.has_updates());

        let opts_with_relation =
            EditOptions::new("UC-001").with_relation(builtin::REFINES, vec!["SOL-001".to_string()]);
        assert!(opts_with_relation.has_updates());
    }

    #[test]
    fn test_item_context_from_item() {
        let item = create_test_item_with_name("SOL-001", builtin::SOLUTION, "Test Solution");
        let ctx = ItemContext::from_item(&item);

        assert_eq!(ctx.id, "SOL-001");
        assert_eq!(ctx.name, "Test Solution");
        assert_eq!(ctx.item_type, builtin::SOLUTION);
    }

    #[test]
    fn test_validate_options_undeclared_field() {
        let service = EditService::new();

        // Valid: specification on requirement type
        let opts = EditOptions::new("SYSREQ-001")
            .maybe_text_field("specification", Some("new spec".to_string()));
        assert!(
            service
                .validate_options(&opts, builtin::SYSTEM_REQUIREMENT)
                .is_ok()
        );

        // Invalid: specification on solution type (declares no fields)
        let opts = EditOptions::new("SOL-001")
            .maybe_text_field("specification", Some("new spec".to_string()));
        assert!(service.validate_options(&opts, builtin::SOLUTION).is_err());
    }

    #[test]
    fn test_validate_options_enum_value() {
        let service = EditService::new();

        let opts =
            EditOptions::new("ADR-001").maybe_text_field("status", Some("accepted".to_string()));
        assert!(
            service
                .validate_options(&opts, builtin::ARCHITECTURE_DECISION_RECORD)
                .is_ok()
        );

        let opts =
            EditOptions::new("ADR-001").maybe_text_field("status", Some("bogus".to_string()));
        assert!(
            service
                .validate_options(&opts, builtin::ARCHITECTURE_DECISION_RECORD)
                .is_err()
        );
    }

    #[test]
    fn test_validate_options_undeclared_relation() {
        let service = EditService::new();

        // Valid: refines on use case
        let opts =
            EditOptions::new("UC-001").with_relation(builtin::REFINES, vec!["SOL-001".to_string()]);
        assert!(service.validate_options(&opts, builtin::USE_CASE).is_ok());

        // Invalid: refines on solution (declares no relation)
        let opts = EditOptions::new("SOL-001")
            .with_relation(builtin::REFINES, vec!["SOL-002".to_string()]);
        assert!(service.validate_options(&opts, builtin::SOLUTION).is_err());
    }

    #[test]
    fn test_merge_values() {
        let service = EditService::new();

        let current = ItemContext {
            id: "SOL-001".to_string(),
            item_type: builtin::SOLUTION,
            name: "Old Name".to_string(),
            description: Some("Old Description".to_string()),
            traceability: TraceabilityLinks::default(),
            attributes: ItemAttributes::new(),
            file_path: PathBuf::from("/test.md"),
        };

        let opts = EditOptions::new("SOL-001").with_name("New Name");

        let merged = service.merge_values(&opts, &current);

        assert_eq!(merged.name, "New Name");
        assert_eq!(merged.description, Some("Old Description".to_string()));
    }

    #[test]
    fn test_merge_values_replaces_field() {
        let service = EditService::new();

        let item = create_test_item("SYSREQ-001", builtin::SYSTEM_REQUIREMENT);
        let current = ItemContext::from_item(&item);

        let opts = EditOptions::new("SYSREQ-001").maybe_text_field(
            "specification",
            Some("The system SHALL be edited.".to_string()),
        );
        let merged = service.merge_values(&opts, &current);

        assert_eq!(
            merged.attributes.get("specification"),
            Some(&crate::model::FieldValue::Text(
                "The system SHALL be edited.".to_string()
            ))
        );
    }

    #[test]
    fn test_merge_values_replaces_relation() {
        let service = EditService::new();

        let item = create_test_item("UC-001", builtin::USE_CASE);
        let mut current = ItemContext::from_item(&item);
        current
            .traceability
            .set(builtin::REFINES, vec!["SOL-001".to_string()]);

        let opts =
            EditOptions::new("UC-001").with_relation(builtin::REFINES, vec!["SOL-002".to_string()]);
        let merged = service.merge_values(&opts, &current);

        assert_eq!(merged.traceability.get(builtin::REFINES), ["SOL-002"]);
    }

    #[test]
    fn test_build_change_summary() {
        let service = EditService::new();

        let old = ItemContext {
            id: "SOL-001".to_string(),
            item_type: builtin::SOLUTION,
            name: "Old Name".to_string(),
            description: None,
            traceability: TraceabilityLinks::default(),
            attributes: ItemAttributes::new(),
            file_path: PathBuf::from("/test.md"),
        };

        let new = EditedValues::new("New Name");

        let changes = service.build_change_summary(&old, &new);

        let name_change = changes.iter().find(|c| c.field == "Name").unwrap();
        assert!(name_change.is_changed());
        assert_eq!(name_change.old_value, "Old Name");
        assert_eq!(name_change.new_value, "New Name");
    }

    #[test]
    fn test_build_frontmatter_yaml() {
        let service = EditService::new();

        let values = EditedValues::new("Test Solution")
            .with_description(Some("A test solution".to_string()));

        let yaml = service.build_frontmatter_yaml("SOL-001", builtin::SOLUTION, &values);

        assert!(yaml.contains("id: \"SOL-001\""));
        assert!(yaml.contains("type: solution"));
        assert!(yaml.contains("name: \"Test Solution\""));
        assert!(yaml.contains("description: \"A test solution\""));
    }

    #[test]
    fn test_build_frontmatter_yaml_preserves_attributes() {
        let service = EditService::new();

        let item =
            crate::test_utils::create_test_item("ADR-001", builtin::ARCHITECTURE_DECISION_RECORD);
        let ctx = ItemContext::from_item(&item);
        let values = service.merge_values(&EditOptions::new("ADR-001").with_name("Renamed"), &ctx);

        let yaml = service.build_frontmatter_yaml(
            "ADR-001",
            builtin::ARCHITECTURE_DECISION_RECORD,
            &values,
        );

        assert!(yaml.contains("name: \"Renamed\""));
        assert!(yaml.contains("Test Decider"));
    }
}

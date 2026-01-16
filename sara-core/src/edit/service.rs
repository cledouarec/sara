//! Edit service implementation.

use std::fs;
use std::path::PathBuf;

use crate::error::EditError;
use crate::graph::KnowledgeGraph;
use crate::model::{FieldChange, Item, ItemType, TraceabilityLinks};
use crate::parser::update_frontmatter;
use crate::query::lookup_item_or_suggest;

use super::{EditOptions, EditedValues};

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
            specification: item.attributes.specification.clone(),
            platform: item.attributes.platform.clone(),
            traceability: TraceabilityLinks::from_upstream(&item.upstream),
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
    ) -> Result<&'a Item, EditError> {
        lookup_item_or_suggest(graph, item_id)
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
    ) -> Result<(), EditError> {
        if opts.specification.is_some() && !item_type.requires_specification() {
            return Err(EditError::IoError(format!(
                "--specification is only valid for requirement types, not {}",
                item_type.display_name()
            )));
        }

        if opts.platform.is_some() && item_type != ItemType::SystemArchitecture {
            return Err(EditError::IoError(
                "--platform is only valid for System Architecture items".to_string(),
            ));
        }

        Ok(())
    }

    /// Merges edit options with current item values.
    pub fn merge_values(&self, opts: &EditOptions, current: &ItemContext) -> EditedValues {
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
            },
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
        self.add_traceability_change(
            &mut changes,
            "Refines",
            &old.traceability.refines,
            &new.traceability.refines,
        );
        self.add_traceability_change(
            &mut changes,
            "Derives from",
            &old.traceability.derives_from,
            &new.traceability.derives_from,
        );
        self.add_traceability_change(
            &mut changes,
            "Satisfies",
            &old.traceability.satisfies,
            &new.traceability.satisfies,
        );

        // Type-specific
        if old.specification.is_some() || new.specification.is_some() {
            changes.push(FieldChange::new(
                "Specification",
                old.specification.as_deref().unwrap_or("(none)"),
                new.specification.as_deref().unwrap_or("(none)"),
            ));
        }

        if old.platform.is_some() || new.platform.is_some() {
            changes.push(FieldChange::new(
                "Platform",
                old.platform.as_deref().unwrap_or("(none)"),
                new.platform.as_deref().unwrap_or("(none)"),
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
    ) -> Result<(), EditError> {
        let content =
            fs::read_to_string(file_path).map_err(|e| EditError::IoError(e.to_string()))?;
        let new_yaml = self.build_frontmatter_yaml(item_id, item_type, new_values);
        let updated_content = update_frontmatter(&content, &new_yaml);
        fs::write(file_path, updated_content).map_err(|e| EditError::IoError(e.to_string()))?;
        Ok(())
    }

    /// Builds YAML frontmatter string from edit values.
    pub fn build_frontmatter_yaml(
        &self,
        item_id: &str,
        item_type: ItemType,
        values: &EditedValues,
    ) -> String {
        let mut yaml = format!(
            "id: \"{}\"\ntype: {}\nname: \"{}\"\n",
            item_id,
            item_type.yaml_value(),
            values.name.replace('"', "\\\"")
        );

        if let Some(ref desc) = values.description {
            yaml += &format!("description: \"{}\"\n", desc.replace('"', "\\\""));
        }

        self.append_traceability_yaml(&mut yaml, "refines", &values.traceability.refines);
        self.append_traceability_yaml(&mut yaml, "derives_from", &values.traceability.derives_from);
        self.append_traceability_yaml(&mut yaml, "satisfies", &values.traceability.satisfies);

        if let Some(ref spec) = values.specification {
            yaml += &format!("specification: \"{}\"\n", spec.replace('"', "\\\""));
        }

        if let Some(ref plat) = values.platform {
            yaml += &format!("platform: \"{}\"\n", plat.replace('"', "\\\""));
        }

        yaml
    }

    /// Appends a traceability list to YAML if non-empty.
    fn append_traceability_yaml(&self, yaml: &mut String, field: &str, ids: &[String]) {
        if ids.is_empty() {
            return;
        }

        *yaml += &format!("{}:\n", field);
        for id in ids {
            *yaml += &format!("  - \"{}\"\n", id);
        }
    }

    /// Performs a non-interactive edit operation.
    pub fn edit(
        &self,
        graph: &KnowledgeGraph,
        opts: &EditOptions,
    ) -> Result<EditResult, EditError> {
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
    use crate::model::{ItemBuilder, ItemId, SourceLocation};

    fn create_test_item(id: &str, item_type: ItemType, name: &str) -> Item {
        let source = SourceLocation::new(PathBuf::from("/repo"), format!("{}.md", id), 1);
        let mut builder = ItemBuilder::new()
            .id(ItemId::new_unchecked(id))
            .item_type(item_type)
            .name(name)
            .source(source);

        if item_type.requires_specification() {
            builder = builder.specification("Test specification");
        }

        builder.build().unwrap()
    }

    #[test]
    fn test_edit_options_has_updates() {
        let opts = EditOptions::new("SOL-001");
        assert!(!opts.has_updates());

        let opts_with_name = EditOptions::new("SOL-001").with_name("New Name");
        assert!(opts_with_name.has_updates());
    }

    #[test]
    fn test_item_context_from_item() {
        let item = create_test_item("SOL-001", ItemType::Solution, "Test Solution");
        let ctx = ItemContext::from_item(&item);

        assert_eq!(ctx.id, "SOL-001");
        assert_eq!(ctx.name, "Test Solution");
        assert_eq!(ctx.item_type, ItemType::Solution);
    }

    #[test]
    fn test_validate_options_specification() {
        let service = EditService::new();

        // Valid: specification on requirement type
        let opts = EditOptions::new("SYSREQ-001").with_specification("new spec");
        assert!(
            service
                .validate_options(&opts, ItemType::SystemRequirement)
                .is_ok()
        );

        // Invalid: specification on solution type
        let opts = EditOptions::new("SOL-001").with_specification("new spec");
        assert!(service.validate_options(&opts, ItemType::Solution).is_err());
    }

    #[test]
    fn test_validate_options_platform() {
        let service = EditService::new();

        // Valid: platform on system architecture
        let opts = EditOptions::new("SYSARCH-001").with_platform("AWS");
        assert!(
            service
                .validate_options(&opts, ItemType::SystemArchitecture)
                .is_ok()
        );

        // Invalid: platform on solution
        let opts = EditOptions::new("SOL-001").with_platform("AWS");
        assert!(service.validate_options(&opts, ItemType::Solution).is_err());
    }

    #[test]
    fn test_merge_values() {
        let service = EditService::new();

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

        let merged = service.merge_values(&opts, &current);

        assert_eq!(merged.name, "New Name");
        assert_eq!(merged.description, Some("Old Description".to_string()));
    }

    #[test]
    fn test_build_change_summary() {
        let service = EditService::new();

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

        let yaml = service.build_frontmatter_yaml("SOL-001", ItemType::Solution, &values);

        assert!(yaml.contains("id: \"SOL-001\""));
        assert!(yaml.contains("type: solution"));
        assert!(yaml.contains("name: \"Test Solution\""));
        assert!(yaml.contains("description: \"A test solution\""));
    }
}

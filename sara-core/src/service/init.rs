//! Item initialization service.
//!
//! Provides functionality to initialize new requirement items or add frontmatter
//! to existing documents.

use std::fs;
use std::path::PathBuf;

use indexmap::IndexMap;

use crate::generator::{self, OutputFormat};
use crate::model::{FieldValue, ItemBuilder, ItemId, ItemType, RelationshipType, SourceLocation};
use crate::parser::{extract_name_from_content, has_frontmatter};
use crate::schema::{self, FieldDef, FieldType, RelationDirection};

/// Options for initializing a new item or adding frontmatter to an existing file.
#[derive(Debug, Clone)]
pub struct InitOptions {
    /// The file path to create or update.
    pub file: PathBuf,
    /// Optional ID (will be auto-generated if not provided).
    pub id: Option<String>,
    /// Optional name (will be extracted from file or generated if not provided).
    pub name: Option<String>,
    /// Optional description.
    pub description: Option<String>,
    /// Whether to overwrite existing frontmatter.
    pub force: bool,
    /// Type-specific configuration.
    pub type_config: TypeConfig,
}

impl InitOptions {
    /// Creates new init options with the given file and type configuration.
    pub fn new(file: PathBuf, type_config: TypeConfig) -> Self {
        Self {
            file,
            id: None,
            name: None,
            description: None,
            force: false,
            type_config,
        }
    }

    /// Returns the item type for this configuration.
    pub fn item_type(&self) -> ItemType {
        self.type_config.item_type()
    }

    /// Sets the ID.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the ID if provided.
    pub fn maybe_id(mut self, id: Option<String>) -> Self {
        self.id = id;
        self
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

    /// Sets the force flag.
    pub fn with_force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }
}

/// Value provided for one declared field when initializing an item.
#[derive(Debug, Clone)]
pub enum FieldInput {
    /// A single value (text, enum, date or item-reference fields).
    Text(String),
    /// A list of values (list fields).
    List(Vec<String>),
}

/// Inputs for the item type being initialized.
///
/// Inputs are keyed by the names the active schema declares for the type:
/// field values by field name, relation targets by relation id. Names the
/// schema does not declare for the type are ignored when the item is built,
/// and required fields without input fall back to their schema placeholder.
#[derive(Debug, Clone)]
pub struct TypeConfig {
    item_type: ItemType,
    fields: IndexMap<String, FieldInput>,
    relations: IndexMap<String, Vec<String>>,
}

impl TypeConfig {
    /// Creates an empty configuration for the given item type.
    #[must_use]
    pub fn new(item_type: ItemType) -> Self {
        Self {
            item_type,
            fields: IndexMap::new(),
            relations: IndexMap::new(),
        }
    }

    /// Returns the item type for this configuration.
    #[must_use]
    pub fn item_type(&self) -> ItemType {
        self.item_type
    }

    /// Sets the input for a declared field.
    #[must_use]
    pub fn field(mut self, name: impl Into<String>, input: FieldInput) -> Self {
        self.fields.insert(name.into(), input);
        self
    }

    /// Sets a single-value field input.
    #[must_use]
    pub fn text_field(self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.field(name, FieldInput::Text(value.into()))
    }

    /// Sets a single-value field input when one is provided.
    #[must_use]
    pub fn maybe_text_field(self, name: impl Into<String>, value: Option<String>) -> Self {
        match value {
            Some(value) => self.text_field(name, value),
            None => self,
        }
    }

    /// Sets a list field input. An empty list is treated as no input.
    #[must_use]
    pub fn list_field(self, name: impl Into<String>, values: Vec<String>) -> Self {
        if values.is_empty() {
            self
        } else {
            self.field(name, FieldInput::List(values))
        }
    }

    /// Sets the targets of a declared relation. An empty list is treated as
    /// no input.
    #[must_use]
    pub fn relation(mut self, relation: impl Into<String>, targets: Vec<String>) -> Self {
        if !targets.is_empty() {
            self.relations.insert(relation.into(), targets);
        }
        self
    }
}

/// Errors that can occur during initialization.
#[derive(Debug, thiserror::Error)]
pub enum InitError {
    /// File already has frontmatter and force was not set.
    #[error("File {0} already has frontmatter. Use force to overwrite.")]
    FrontmatterExists(PathBuf),

    /// Invalid option for the given item type.
    #[error("{0}")]
    InvalidOption(String),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result of a successful initialization.
#[derive(Debug, Clone)]
pub struct InitResult {
    /// The resolved ID.
    pub id: String,
    /// The resolved name.
    pub name: String,
    /// The item type.
    pub item_type: ItemType,
    /// The file path.
    pub file: PathBuf,
    /// Whether an existing file was updated (true) or a new file was created (false).
    pub updated_existing: bool,
    /// Whether frontmatter was replaced (only relevant if updated_existing is true).
    pub replaced_frontmatter: bool,
    /// Whether specification field needs attention.
    pub needs_specification: bool,
}

/// Service for initializing requirement items.
#[derive(Debug, Default)]
pub struct InitService;

impl InitService {
    /// Creates a new init service.
    pub fn new() -> Self {
        Self
    }

    /// Initializes an item based on the provided options.
    ///
    /// This will either create a new file or update an existing file with frontmatter.
    pub fn init(&self, opts: &InitOptions) -> Result<InitResult, InitError> {
        // Check for existing frontmatter
        if opts.file.exists() && !opts.force {
            let content = fs::read_to_string(&opts.file)?;
            if has_frontmatter(&content) {
                return Err(InitError::FrontmatterExists(opts.file.clone()));
            }
        }

        let item_type = opts.item_type();

        // Resolve ID and name
        let id = self.resolve_id(opts);
        let name = self.resolve_name(opts, &id)?;

        // Build an Item from init options
        let item = self.build_item(opts, &id, &name);

        // Write the file
        let (updated_existing, replaced_frontmatter) = self.write_file(opts, &item)?;

        // Check if specification is needed
        let needs_specification = self.check_needs_specification(&opts.type_config);

        Ok(InitResult {
            id,
            name,
            item_type,
            file: opts.file.clone(),
            updated_existing,
            replaced_frontmatter,
            needs_specification,
        })
    }

    /// Checks whether a required text field fell back to its placeholder.
    fn check_needs_specification(&self, type_config: &TypeConfig) -> bool {
        schema::item_type_def(type_config.item_type().as_str()).is_some_and(|def| {
            def.fields.iter().any(|field| {
                field.required
                    && matches!(field.field_type, FieldType::Text)
                    && !type_config.fields.contains_key(&field.name)
            })
        })
    }

    /// Resolves the ID from options or generates a new one.
    fn resolve_id(&self, opts: &InitOptions) -> String {
        opts.id
            .clone()
            .unwrap_or_else(|| opts.item_type().generate_id(None))
    }

    /// Resolves the name from options, file content, or file stem.
    fn resolve_name(&self, opts: &InitOptions, id: &str) -> Result<String, InitError> {
        if let Some(ref name) = opts.name {
            return Ok(name.clone());
        }

        if opts.file.exists() {
            let content = fs::read_to_string(&opts.file)?;
            if let Some(name) = extract_name_from_content(&content) {
                return Ok(name);
            }
        }

        Ok(self.file_stem_or_fallback(&opts.file, id))
    }

    /// Returns the file stem as a string, or the fallback if unavailable.
    fn file_stem_or_fallback(&self, file: &std::path::Path, fallback: &str) -> String {
        file.file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| fallback.to_string())
    }

    /// Builds an `Item` from init options for document generation.
    fn build_item(&self, opts: &InitOptions, id: &str, name: &str) -> crate::model::Item {
        let source = SourceLocation {
            repository: PathBuf::new(),
            file_path: opts.file.clone(),
            git_ref: None,
        };

        let mut builder = ItemBuilder::new()
            .id(ItemId::new_unchecked(id))
            .item_type(opts.item_type())
            .name(name)
            .source(source);

        if let Some(ref desc) = opts.description {
            builder = builder.description(desc);
        }

        // Fill the declared fields and relations from the configuration.
        if let Some(def) = schema::item_type_def(opts.item_type().as_str()) {
            for field in &def.fields {
                let input = opts.type_config.fields.get(&field.name);
                if let Some(value) = init_field_value(field, input) {
                    builder = builder.attribute(field.name.clone(), value);
                }
            }

            let mut relationships = Vec::new();
            for target in &def.allowed_targets {
                let Some(ids) = opts.type_config.relations.get(&target.relation) else {
                    continue;
                };
                let Some(relation) = schema::relation_def(&target.relation) else {
                    continue;
                };
                if relation.direction == RelationDirection::Downstream {
                    continue;
                }
                if let Some(rel_type) = RelationshipType::from_id(&target.relation) {
                    relationships.extend(super::ids_to_relationships(ids, rel_type));
                }
            }
            builder = builder.relationships(relationships);
        }

        builder.build().expect("Failed to build item for init")
    }

    /// Writes the file, either updating existing or creating new.
    ///
    /// Returns (updated_existing, replaced_frontmatter).
    fn write_file(
        &self,
        opts: &InitOptions,
        item: &crate::model::Item,
    ) -> Result<(bool, bool), InitError> {
        if opts.file.exists() {
            let replaced = self.update_existing_file(opts, item)?;
            Ok((true, replaced))
        } else {
            self.create_new_file(opts, item)?;
            Ok((false, false))
        }
    }

    /// Updates an existing file by adding or replacing frontmatter.
    ///
    /// Returns true if frontmatter was replaced, false if it was added.
    fn update_existing_file(
        &self,
        opts: &InitOptions,
        item: &crate::model::Item,
    ) -> Result<bool, InitError> {
        let content = fs::read_to_string(&opts.file)?;
        let frontmatter = generator::generate_metadata(item, OutputFormat::Markdown);

        let (new_content, replaced) = if has_frontmatter(&content) && opts.force {
            let body = remove_frontmatter(&content);
            (format!("{}\n{}", frontmatter, body), true)
        } else {
            (format!("{}\n{}", frontmatter, content), false)
        };

        fs::write(&opts.file, new_content)?;
        Ok(replaced)
    }

    /// Creates a new file with the generated document.
    fn create_new_file(
        &self,
        opts: &InitOptions,
        item: &crate::model::Item,
    ) -> Result<(), InitError> {
        let document = generator::generate_document(item, OutputFormat::Markdown);

        if let Some(parent) = opts.file.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&opts.file, document)?;
        Ok(())
    }
}

/// Removes YAML frontmatter delimited by `---` from `content`.
fn remove_frontmatter(content: &str) -> &str {
    let mut in_frontmatter = false;
    let mut byte_offset = 0;

    for line in content.lines() {
        // Advance past the line and its newline delimiter.
        let line_end = byte_offset + line.len() + 1;

        if line.trim() == "---" {
            if !in_frontmatter {
                in_frontmatter = true;
            } else {
                // Found closing delimiter; return everything after it.
                let end = line_end.min(content.len());
                return &content[end..];
            }
        }

        byte_offset = line_end;
    }

    content
}

/// Resolves the typed value of a declared field from its init input.
///
/// Required fields without usable input fall back to the schema placeholder
/// (or the first allowed value for enums) so initialization always produces
/// a buildable item; optional fields without input are simply omitted.
pub(super) fn init_field_value(field: &FieldDef, input: Option<&FieldInput>) -> Option<FieldValue> {
    let required_fallback = || {
        field.required.then(|| {
            field
                .placeholder
                .clone()
                .unwrap_or_else(|| "TBD".to_string())
        })
    };

    match &field.field_type {
        FieldType::Enum { values } => {
            let provided = match input {
                Some(FieldInput::Text(value)) if values.contains(value) => Some(value.clone()),
                _ => None,
            };
            provided
                .or_else(|| {
                    field.required.then(|| {
                        field
                            .placeholder
                            .clone()
                            .filter(|p| values.contains(p))
                            .or_else(|| values.first().cloned())
                            .unwrap_or_default()
                    })
                })
                .map(FieldValue::Enum)
        }
        FieldType::List(inner) => {
            let values = match input {
                Some(FieldInput::List(values)) => values.clone(),
                Some(FieldInput::Text(value)) => vec![value.clone()],
                None => required_fallback().map(|p| vec![p]).unwrap_or_default(),
            };
            if values.is_empty() {
                None
            } else {
                Some(FieldValue::List(
                    values
                        .iter()
                        .map(|v| scalar_field_value(v, inner))
                        .collect(),
                ))
            }
        }
        scalar => match input {
            Some(FieldInput::Text(value)) => Some(scalar_field_value(value, scalar)),
            _ => required_fallback().map(|p| scalar_field_value(&p, scalar)),
        },
    }
}

/// Wraps a raw string as a value of the given scalar field type.
fn scalar_field_value(value: &str, field_type: &FieldType) -> FieldValue {
    match field_type {
        FieldType::ItemRef => FieldValue::ItemRef(ItemId::new_unchecked(value)),
        FieldType::Date => FieldValue::Date(value.to_string()),
        FieldType::Enum { .. } => FieldValue::Enum(value.to_string()),
        FieldType::Text | FieldType::List(_) => FieldValue::Text(value.to_string()),
    }
}

/// Parses an item type string into an [`ItemType`].
///
/// Accepts the schema id (`use_case`), its squashed form (`usecase`) and the
/// type's id prefix in any case (`UC`), for every type the active schema
/// knows.
pub fn parse_item_type(type_str: &str) -> Option<ItemType> {
    let lower = type_str.to_lowercase();
    ItemType::all().into_iter().find(|item_type| {
        let id = item_type.as_str();
        id == lower || id.replace('_', "") == lower || item_type.prefix().to_lowercase() == lower
    })
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    use crate::schema::builtin;

    #[test]
    fn test_parse_item_type() {
        assert_eq!(parse_item_type("solution"), Some(builtin::SOLUTION));
        assert_eq!(parse_item_type("SOL"), Some(builtin::SOLUTION));
        assert_eq!(parse_item_type("use_case"), Some(builtin::USE_CASE));
        assert_eq!(parse_item_type("UC"), Some(builtin::USE_CASE));
        assert_eq!(parse_item_type("invalid"), None);
    }

    #[test]
    fn test_remove_frontmatter() {
        let content = "---\nid: test\n---\n# Body";
        let body = remove_frontmatter(content);
        assert_eq!(body.trim(), "# Body");
    }

    #[test]
    fn test_init_new_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");

        let opts = InitOptions::new(file_path.clone(), TypeConfig::new(builtin::SOLUTION))
            .with_id("SOL-001")
            .with_name("Test Solution");

        let service = InitService::new();
        let result = service.init(&opts).unwrap();

        assert_eq!(result.id, "SOL-001");
        assert_eq!(result.name, "Test Solution");
        assert!(!result.updated_existing);
        assert!(file_path.exists());

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("id: \"SOL-001\""));
        assert!(content.contains("# Solution: Test Solution"));
    }

    #[test]
    fn test_init_existing_file_without_frontmatter() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("existing.md");

        // Create existing file without frontmatter
        fs::write(&file_path, "# My Document\n\nSome content here.").unwrap();

        let opts = InitOptions::new(file_path.clone(), TypeConfig::new(builtin::USE_CASE))
            .with_id("UC-001");

        let service = InitService::new();
        let result = service.init(&opts).unwrap();

        assert_eq!(result.id, "UC-001");
        assert_eq!(result.name, "My Document"); // Extracted from heading
        assert!(result.updated_existing);
        assert!(!result.replaced_frontmatter);

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("id: \"UC-001\""));
        assert!(content.contains("# My Document"));
    }

    #[test]
    fn test_init_existing_file_with_frontmatter_no_force() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("existing.md");

        // Create existing file with frontmatter
        fs::write(&file_path, "---\nid: OLD-001\n---\n# Content").unwrap();

        let opts =
            InitOptions::new(file_path, TypeConfig::new(builtin::SOLUTION)).with_id("SOL-001");

        let service = InitService::new();
        let result = service.init(&opts);

        assert!(matches!(result, Err(InitError::FrontmatterExists(_))));
    }

    #[test]
    fn test_init_existing_file_with_frontmatter_force() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("existing.md");

        // Create existing file with frontmatter
        fs::write(&file_path, "---\nid: OLD-001\n---\n# Content").unwrap();

        let opts = InitOptions::new(file_path.clone(), TypeConfig::new(builtin::SOLUTION))
            .with_id("SOL-001")
            .with_name("New Solution")
            .with_force(true);

        let service = InitService::new();
        let result = service.init(&opts).unwrap();

        assert_eq!(result.id, "SOL-001");
        assert!(result.updated_existing);
        assert!(result.replaced_frontmatter);

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("id: \"SOL-001\""));
        assert!(!content.contains("OLD-001"));
    }

    #[test]
    fn test_needs_specification() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");

        let opts = InitOptions::new(file_path, TypeConfig::new(builtin::SYSTEM_REQUIREMENT))
            .with_id("SYSREQ-001");

        let service = InitService::new();
        let result = service.init(&opts).unwrap();

        assert!(result.needs_specification);
    }

    #[test]
    fn test_needs_specification_when_provided() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");

        let type_config = TypeConfig::new(builtin::SYSTEM_REQUIREMENT)
            .text_field("specification", "The system SHALL do something");

        let opts = InitOptions::new(file_path, type_config).with_id("SYSREQ-001");

        let service = InitService::new();
        let result = service.init(&opts).unwrap();

        assert!(!result.needs_specification);
    }
}

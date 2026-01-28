//! Init service for creating and updating items with file I/O.
//!
//! This module provides file I/O operations for item initialization,
//! using domain types from `model/` and generators from `generator/`.

#![allow(clippy::result_large_err)]

use std::fs;
use std::path::PathBuf;

use crate::generator::{
    extract_name_from_content, generate_document, generate_frontmatter, generate_id,
};
use crate::model::{InitData, InitOptions, ItemType, TypeFields, prepare_init};
use crate::parser::{extract_body, has_frontmatter};

/// Errors that can occur during initialization.
#[derive(Debug, thiserror::Error)]
pub enum InitError {
    /// File already has frontmatter and force was not set.
    #[error("File {0} already has frontmatter. Use force to overwrite.")]
    FrontmatterExists(PathBuf),

    /// Validation error from domain layer.
    #[error("{0}")]
    Validation(#[from] crate::error::ValidationError),

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

/// File I/O options for initialization.
///
/// Combines domain `InitOptions` with file I/O concerns.
#[derive(Debug, Clone)]
pub struct InitFileOptions {
    /// The file path to create or update.
    pub file: PathBuf,
    /// Whether to overwrite existing frontmatter.
    pub force: bool,
    /// Domain options for the item.
    pub init_options: InitOptions,
}

impl InitFileOptions {
    /// Creates new file options for the given file and item type.
    pub fn new(file: PathBuf, item_type: ItemType) -> Self {
        Self {
            file,
            force: false,
            init_options: InitOptions::new(item_type),
        }
    }

    /// Creates new file options with specific type fields.
    pub fn with_fields(file: PathBuf, item_type: ItemType, fields: TypeFields) -> Self {
        Self {
            file,
            force: false,
            init_options: InitOptions::with_fields(item_type, fields),
        }
    }

    /// Sets the force flag.
    pub fn with_force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }

    /// Sets the ID.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.init_options = self.init_options.with_id(id);
        self
    }

    /// Sets the ID if provided.
    pub fn maybe_id(mut self, id: Option<String>) -> Self {
        self.init_options = self.init_options.maybe_id(id);
        self
    }

    /// Sets the name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.init_options = self.init_options.with_name(name);
        self
    }

    /// Sets the name if provided.
    pub fn maybe_name(mut self, name: Option<String>) -> Self {
        self.init_options = self.init_options.maybe_name(name);
        self
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.init_options = self.init_options.with_description(description);
        self
    }

    /// Sets the description if provided.
    pub fn maybe_description(mut self, description: Option<String>) -> Self {
        self.init_options = self.init_options.maybe_description(description);
        self
    }

    /// Sets the type-specific fields.
    pub fn with_type_fields(mut self, fields: TypeFields) -> Self {
        self.init_options = self.init_options.with_type_fields(fields);
        self
    }

    /// Returns the item type.
    pub fn item_type(&self) -> ItemType {
        self.init_options.item_type
    }
}

/// Creates a new item or adds frontmatter to an existing file.
pub fn create_item(opts: &InitFileOptions) -> Result<InitResult, InitError> {
    // Check for existing frontmatter
    if opts.file.exists() && !opts.force {
        let content = fs::read_to_string(&opts.file)?;
        if has_frontmatter(&content) {
            return Err(InitError::FrontmatterExists(opts.file.clone()));
        }
    }

    let item_type = opts.item_type();

    // Resolve name from file content if not provided
    let name_from_file = if opts.init_options.name.is_none() && opts.file.exists() {
        let content = fs::read_to_string(&opts.file)?;
        extract_name_from_content(&content)
    } else {
        None
    };

    // Prepare init data using domain function
    let data = prepare_init(
        &opts.init_options,
        |t| generate_id(t, None),
        |id| {
            name_from_file
                .clone()
                .unwrap_or_else(|| file_stem_or_fallback(&opts.file, id))
        },
    )?;

    // Write the file
    let (updated_existing, replaced_frontmatter) = write_file(opts, &data)?;

    let needs_specification = data.needs_specification();

    Ok(InitResult {
        id: data.id,
        name: data.name,
        item_type,
        file: opts.file.clone(),
        updated_existing,
        replaced_frontmatter,
        needs_specification,
    })
}

/// Updates an existing item (alias for create_item with appropriate options).
pub fn update_item(opts: &InitFileOptions) -> Result<InitResult, InitError> {
    create_item(opts)
}

/// Returns the file stem as a string, or the fallback if unavailable.
fn file_stem_or_fallback(file: &std::path::Path, fallback: &str) -> String {
    file.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| fallback.to_string())
}

/// Writes the file, either updating existing or creating new.
/// Returns (updated_existing, replaced_frontmatter).
fn write_file(opts: &InitFileOptions, data: &InitData) -> Result<(bool, bool), InitError> {
    if opts.file.exists() {
        let replaced = update_existing_file(opts, data)?;
        Ok((true, replaced))
    } else {
        create_new_file(opts, data)?;
        Ok((false, false))
    }
}

/// Updates an existing file by adding or replacing frontmatter.
/// Returns true if frontmatter was replaced, false if it was added.
fn update_existing_file(opts: &InitFileOptions, data: &InitData) -> Result<bool, InitError> {
    let content = fs::read_to_string(&opts.file)?;
    let frontmatter = generate_frontmatter(
        data.item_type,
        &data.id,
        &data.name,
        data.description.as_deref(),
        &data.fields,
    );

    let (new_content, replaced) = if has_frontmatter(&content) && opts.force {
        let body = extract_body(&content);
        (format!("{}{}", frontmatter, body), true)
    } else {
        (format!("{}{}", frontmatter, content), false)
    };

    fs::write(&opts.file, new_content)?;
    Ok(replaced)
}

/// Creates a new file with the generated document.
fn create_new_file(opts: &InitFileOptions, data: &InitData) -> Result<(), InitError> {
    let document = generate_document(data);

    if let Some(parent) = opts.file.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&opts.file, document)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_new_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");

        let opts = InitFileOptions::new(file_path.clone(), ItemType::Solution)
            .with_id("SOL-001")
            .with_name("Test Solution");

        let result = create_item(&opts).unwrap();

        assert_eq!(result.id, "SOL-001");
        assert_eq!(result.name, "Test Solution");
        assert!(!result.updated_existing);
        assert!(file_path.exists());

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("id: \"SOL-001\""));
        assert!(content.contains("# Solution: Test Solution"));
    }

    #[test]
    fn test_create_item_existing_file_without_frontmatter() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("existing.md");

        // Create existing file without frontmatter
        fs::write(&file_path, "# My Document\n\nSome content here.").unwrap();

        let opts = InitFileOptions::new(file_path.clone(), ItemType::UseCase).with_id("UC-001");

        let result = create_item(&opts).unwrap();

        assert_eq!(result.id, "UC-001");
        assert_eq!(result.name, "My Document"); // Extracted from heading
        assert!(result.updated_existing);
        assert!(!result.replaced_frontmatter);

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("id: \"UC-001\""));
        assert!(content.contains("# My Document"));
    }

    #[test]
    fn test_create_item_existing_file_with_frontmatter_no_force() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("existing.md");

        // Create existing file with frontmatter
        fs::write(&file_path, "---\nid: OLD-001\n---\n# Content").unwrap();

        let opts = InitFileOptions::new(file_path, ItemType::Solution).with_id("SOL-001");

        let result = create_item(&opts);

        assert!(matches!(result, Err(InitError::FrontmatterExists(_))));
    }

    #[test]
    fn test_create_item_existing_file_with_frontmatter_force() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("existing.md");

        // Create existing file with frontmatter
        fs::write(&file_path, "---\nid: OLD-001\n---\n# Content").unwrap();

        let opts = InitFileOptions::new(file_path.clone(), ItemType::Solution)
            .with_id("SOL-001")
            .with_name("New Solution")
            .with_force(true);

        let result = create_item(&opts).unwrap();

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

        let opts =
            InitFileOptions::new(file_path, ItemType::SystemRequirement).with_id("SYSREQ-001");

        let result = create_item(&opts).unwrap();

        assert!(result.needs_specification);
    }

    #[test]
    fn test_needs_specification_when_provided() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");

        let fields = TypeFields::new().with_specification("The system SHALL do something");

        let opts = InitFileOptions::with_fields(file_path, ItemType::SystemRequirement, fields)
            .with_id("SYSREQ-001");

        let result = create_item(&opts).unwrap();

        assert!(!result.needs_specification);
    }
}

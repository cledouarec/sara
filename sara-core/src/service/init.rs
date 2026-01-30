//! Init service for creating and updating items with file I/O.
//!
//! This module provides file I/O operations for item initialization.

use std::fs;
use std::path::PathBuf;

use crate::generator::{generate_document, generate_frontmatter};
use crate::graph::KnowledgeGraph;
use crate::model::{Item, ItemType};
use crate::parser::{extract_body, has_frontmatter};

/// Suggests the next ID based on existing items in the graph.
///
/// This is a convenience wrapper around `KnowledgeGraph::suggest_next_id`.
/// If no graph is provided, returns the first ID (e.g., "SOL-001").
pub fn suggest_next_id(item_type: ItemType, graph: Option<&KnowledgeGraph>) -> String {
    graph
        .map(|g| g.suggest_next_id(item_type))
        .unwrap_or_else(|| item_type.generate_id(None))
}

/// Errors that can occur during initialization.
#[derive(Debug, thiserror::Error)]
pub enum InitError {
    /// File already has frontmatter and force was not set.
    #[error("File {0} already has frontmatter. Use force to overwrite.")]
    FrontmatterExists(PathBuf),

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
#[derive(Debug, Clone)]
pub struct InitFileOptions {
    /// Whether to overwrite existing frontmatter.
    pub force: bool,
    /// The item to create.
    pub item: Item,
}

impl InitFileOptions {
    /// Creates new file options for the given item.
    pub fn new(item: Item) -> Self {
        Self { force: false, item }
    }

    /// Sets the force flag.
    pub fn with_force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }

    /// Returns the file path from the item's source location.
    pub fn file(&self) -> &std::path::Path {
        &self.item.source.file_path
    }
}

/// Creates a new item or adds frontmatter to an existing file.
pub fn create_item(opts: &InitFileOptions) -> Result<InitResult, InitError> {
    let file = opts.file();

    // Check for existing frontmatter
    if file.exists() && !opts.force {
        let content = fs::read_to_string(file)?;
        if has_frontmatter(&content) {
            return Err(InitError::FrontmatterExists(file.to_path_buf()));
        }
    }

    let needs_specification = check_needs_specification(&opts.item);

    // Write the file
    let (updated_existing, replaced_frontmatter) = write_file(opts)?;

    Ok(InitResult {
        id: opts.item.id.as_str().to_string(),
        name: opts.item.name.clone(),
        item_type: opts.item.item_type,
        file: file.to_path_buf(),
        updated_existing,
        replaced_frontmatter,
        needs_specification,
    })
}

/// Checks if the item needs specification to be filled.
fn check_needs_specification(item: &Item) -> bool {
    if !item.item_type.requires_specification() {
        return false;
    }

    item.attributes
        .specification()
        .map(|s| s.is_empty())
        .unwrap_or(true)
}

/// Writes the file, either updating existing or creating new.
/// Returns (updated_existing, replaced_frontmatter).
fn write_file(opts: &InitFileOptions) -> Result<(bool, bool), InitError> {
    let file = opts.file();
    if file.exists() {
        let replaced = update_existing_file(opts)?;
        Ok((true, replaced))
    } else {
        create_new_file(opts)?;
        Ok((false, false))
    }
}

/// Updates an existing file by adding or replacing frontmatter.
/// Returns true if frontmatter was replaced, false if it was added.
fn update_existing_file(opts: &InitFileOptions) -> Result<bool, InitError> {
    let file = opts.file();
    let content = fs::read_to_string(file)?;
    let frontmatter = generate_frontmatter(&opts.item);

    let (new_content, replaced) = if has_frontmatter(&content) && opts.force {
        let body = extract_body(&content);
        (format!("{}{}", frontmatter, body), true)
    } else {
        (format!("{}{}", frontmatter, content), false)
    };

    fs::write(file, new_content)?;
    Ok(replaced)
}

/// Creates a new file with the generated document.
fn create_new_file(opts: &InitFileOptions) -> Result<(), InitError> {
    let file = opts.file();
    let document = generate_document(&opts.item);

    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(file, document)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AdrStatus, ItemAttributes, ItemBuilder, ItemId, SourceLocation};
    use tempfile::TempDir;

    fn build_test_item(file: &std::path::Path, item_type: ItemType, id: &str, name: &str) -> Item {
        ItemBuilder::new()
            .id(ItemId::new_unchecked(id))
            .item_type(item_type)
            .name(name)
            .source(SourceLocation::new(std::path::PathBuf::from("."), file))
            .attributes(ItemAttributes::for_type(item_type))
            .build()
            .unwrap()
    }

    #[test]
    fn test_create_new_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");

        let item = build_test_item(&file_path, ItemType::Solution, "SOL-001", "Test Solution");
        let opts = InitFileOptions::new(item);

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

        let item = build_test_item(&file_path, ItemType::UseCase, "UC-001", "My Document");
        let opts = InitFileOptions::new(item);

        let result = create_item(&opts).unwrap();

        assert_eq!(result.id, "UC-001");
        assert_eq!(result.name, "My Document");
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

        let item = build_test_item(&file_path, ItemType::Solution, "SOL-001", "New Solution");
        let opts = InitFileOptions::new(item);

        let result = create_item(&opts);

        assert!(matches!(result, Err(InitError::FrontmatterExists(_))));
    }

    #[test]
    fn test_create_item_existing_file_with_frontmatter_force() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("existing.md");

        // Create existing file with frontmatter
        fs::write(&file_path, "---\nid: OLD-001\n---\n# Content").unwrap();

        let item = build_test_item(&file_path, ItemType::Solution, "SOL-001", "New Solution");
        let opts = InitFileOptions::new(item).with_force(true);

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

        // Use build_with_resolvers because init allows empty specification
        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("SYSREQ-001"))
            .item_type(ItemType::SystemRequirement)
            .name("Test Req")
            .source(SourceLocation::new(
                std::path::PathBuf::from("."),
                &file_path,
            ))
            .attributes(ItemAttributes::SystemRequirement {
                specification: String::new(),
                depends_on: Vec::new(),
            })
            .build_with_resolvers(|_| String::new(), |id| id.to_string())
            .unwrap();

        let opts = InitFileOptions::new(item);
        let result = create_item(&opts).unwrap();

        assert!(result.needs_specification);
    }

    #[test]
    fn test_needs_specification_when_provided() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");

        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("SYSREQ-001"))
            .item_type(ItemType::SystemRequirement)
            .name("Test Req")
            .source(SourceLocation::new(
                std::path::PathBuf::from("."),
                &file_path,
            ))
            .attributes(ItemAttributes::SystemRequirement {
                specification: "The system SHALL do something".to_string(),
                depends_on: Vec::new(),
            })
            .build()
            .unwrap();

        let opts = InitFileOptions::new(item);
        let result = create_item(&opts).unwrap();

        assert!(!result.needs_specification);
    }

    #[test]
    fn test_create_adr_with_attributes() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("adr.md");

        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("ADR-001"))
            .item_type(ItemType::ArchitectureDecisionRecord)
            .name("Use Rust")
            .source(SourceLocation::new(
                std::path::PathBuf::from("."),
                &file_path,
            ))
            .attributes(ItemAttributes::Adr {
                status: AdrStatus::Proposed,
                deciders: vec!["Alice".to_string()],
                supersedes: Vec::new(),
            })
            .build()
            .unwrap();

        let opts = InitFileOptions::new(item);
        let result = create_item(&opts).unwrap();

        assert_eq!(result.id, "ADR-001");
        assert_eq!(result.name, "Use Rust");

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("status: proposed"));
        assert!(content.contains("deciders:"));
    }
}

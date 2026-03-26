//! Item initialization service.
//!
//! Provides functionality to initialize new requirement items or add frontmatter
//! to existing documents.

use std::fs;
use std::path::PathBuf;

use crate::generator::{self, OutputFormat};
use crate::model::{AdrStatus, ItemBuilder, ItemId, ItemType, RelationshipType, SourceLocation};
use crate::parser::{extract_name_from_content, has_frontmatter};

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

/// Type-specific configuration for each item type.
#[derive(Debug, Clone)]
pub enum TypeConfig {
    /// Solution - no type-specific options.
    Solution,

    /// Use Case with optional refines references.
    UseCase {
        /// Solution(s) this use case refines.
        refines: Vec<String>,
    },

    /// Scenario with optional refines references.
    Scenario {
        /// Use case(s) this scenario refines.
        refines: Vec<String>,
    },

    /// System Requirement with specification and traceability.
    SystemRequirement {
        /// Specification statement.
        specification: Option<String>,
        /// Scenario(s) this requirement derives from.
        derives_from: Vec<String>,
        /// Peer dependencies.
        depends_on: Vec<String>,
    },

    /// System Architecture with platform and traceability.
    SystemArchitecture {
        /// Target platform.
        platform: Option<String>,
        /// System requirement(s) this architecture satisfies.
        satisfies: Vec<String>,
    },

    /// Software Requirement with specification and traceability.
    SoftwareRequirement {
        /// Specification statement.
        specification: Option<String>,
        /// System architecture/requirement(s) this derives from.
        derives_from: Vec<String>,
        /// Peer dependencies.
        depends_on: Vec<String>,
    },

    /// Hardware Requirement with specification and traceability.
    HardwareRequirement {
        /// Specification statement.
        specification: Option<String>,
        /// System architecture/requirement(s) this derives from.
        derives_from: Vec<String>,
        /// Peer dependencies.
        depends_on: Vec<String>,
    },

    /// Software Detailed Design with traceability.
    SoftwareDetailedDesign {
        /// Software requirement(s) this design satisfies.
        satisfies: Vec<String>,
    },

    /// Hardware Detailed Design with traceability.
    HardwareDetailedDesign {
        /// Hardware requirement(s) this design satisfies.
        satisfies: Vec<String>,
    },

    /// Architecture Decision Record with ADR-specific fields.
    Adr {
        /// ADR status (proposed, accepted, deprecated, superseded).
        status: Option<String>,
        /// Decision makers.
        deciders: Vec<String>,
        /// Design artifacts this ADR justifies.
        justifies: Vec<String>,
        /// Older ADRs this decision supersedes.
        supersedes: Vec<String>,
        /// Newer ADR that supersedes this one.
        superseded_by: Option<String>,
    },
}

impl TypeConfig {
    /// Returns the item type for this configuration.
    pub fn item_type(&self) -> ItemType {
        match self {
            TypeConfig::Solution => ItemType::Solution,
            TypeConfig::UseCase { .. } => ItemType::UseCase,
            TypeConfig::Scenario { .. } => ItemType::Scenario,
            TypeConfig::SystemRequirement { .. } => ItemType::SystemRequirement,
            TypeConfig::SystemArchitecture { .. } => ItemType::SystemArchitecture,
            TypeConfig::SoftwareRequirement { .. } => ItemType::SoftwareRequirement,
            TypeConfig::HardwareRequirement { .. } => ItemType::HardwareRequirement,
            TypeConfig::SoftwareDetailedDesign { .. } => ItemType::SoftwareDetailedDesign,
            TypeConfig::HardwareDetailedDesign { .. } => ItemType::HardwareDetailedDesign,
            TypeConfig::Adr { .. } => ItemType::ArchitectureDecisionRecord,
        }
    }

    /// Creates a `Solution` config.
    pub fn solution() -> Self {
        TypeConfig::Solution
    }

    /// Creates a `UseCase` config.
    pub fn use_case() -> Self {
        TypeConfig::UseCase {
            refines: Vec::new(),
        }
    }

    /// Creates a `Scenario` config.
    pub fn scenario() -> Self {
        TypeConfig::Scenario {
            refines: Vec::new(),
        }
    }

    /// Creates a `SystemRequirement` config.
    pub fn system_requirement() -> Self {
        TypeConfig::SystemRequirement {
            specification: None,
            derives_from: Vec::new(),
            depends_on: Vec::new(),
        }
    }

    /// Creates a `SystemArchitecture` config.
    pub fn system_architecture() -> Self {
        TypeConfig::SystemArchitecture {
            platform: None,
            satisfies: Vec::new(),
        }
    }

    /// Creates a `SoftwareRequirement` config.
    pub fn software_requirement() -> Self {
        TypeConfig::SoftwareRequirement {
            specification: None,
            derives_from: Vec::new(),
            depends_on: Vec::new(),
        }
    }

    /// Creates a `HardwareRequirement` config.
    pub fn hardware_requirement() -> Self {
        TypeConfig::HardwareRequirement {
            specification: None,
            derives_from: Vec::new(),
            depends_on: Vec::new(),
        }
    }

    /// Creates a `SoftwareDetailedDesign` config.
    pub fn software_detailed_design() -> Self {
        TypeConfig::SoftwareDetailedDesign {
            satisfies: Vec::new(),
        }
    }

    /// Creates a `HardwareDetailedDesign` config.
    pub fn hardware_detailed_design() -> Self {
        TypeConfig::HardwareDetailedDesign {
            satisfies: Vec::new(),
        }
    }

    /// Creates an ADR config.
    pub fn adr() -> Self {
        TypeConfig::Adr {
            status: None,
            deciders: Vec::new(),
            justifies: Vec::new(),
            supersedes: Vec::new(),
            superseded_by: None,
        }
    }

    /// Creates a `TypeConfig` from an `ItemType` with default values.
    pub fn from_item_type(item_type: ItemType) -> Self {
        match item_type {
            ItemType::Solution => TypeConfig::solution(),
            ItemType::UseCase => TypeConfig::use_case(),
            ItemType::Scenario => TypeConfig::scenario(),
            ItemType::SystemRequirement => TypeConfig::system_requirement(),
            ItemType::SystemArchitecture => TypeConfig::system_architecture(),
            ItemType::SoftwareRequirement => TypeConfig::software_requirement(),
            ItemType::HardwareRequirement => TypeConfig::hardware_requirement(),
            ItemType::SoftwareDetailedDesign => TypeConfig::software_detailed_design(),
            ItemType::HardwareDetailedDesign => TypeConfig::hardware_detailed_design(),
            ItemType::ArchitectureDecisionRecord => TypeConfig::adr(),
        }
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

    /// Checks if the type needs a specification but doesn't have one.
    fn check_needs_specification(&self, type_config: &TypeConfig) -> bool {
        match type_config {
            TypeConfig::SystemRequirement { specification, .. }
            | TypeConfig::SoftwareRequirement { specification, .. }
            | TypeConfig::HardwareRequirement { specification, .. } => specification.is_none(),
            _ => false,
        }
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

        // Build relationships and attributes from TypeConfig
        match &opts.type_config {
            TypeConfig::Solution => {}

            TypeConfig::UseCase { refines } | TypeConfig::Scenario { refines } => {
                builder = builder.relationships(super::ids_to_relationships(
                    refines,
                    RelationshipType::Refines,
                ));
            }

            TypeConfig::SystemRequirement {
                specification,
                derives_from,
                depends_on,
            }
            | TypeConfig::SoftwareRequirement {
                specification,
                derives_from,
                depends_on,
            }
            | TypeConfig::HardwareRequirement {
                specification,
                derives_from,
                depends_on,
            } => {
                let spec = specification
                    .clone()
                    .unwrap_or_else(|| "The system SHALL <describe the requirement>.".to_string());
                builder = builder.specification(spec);
                builder = builder.relationships(super::ids_to_relationships(
                    derives_from,
                    RelationshipType::DerivesFrom,
                ));
                for dep in depends_on {
                    builder = builder.depends_on(ItemId::new_unchecked(dep));
                }
            }

            TypeConfig::SystemArchitecture {
                platform,
                satisfies,
            } => {
                if let Some(p) = platform {
                    builder = builder.platform(p);
                }
                builder = builder.relationships(super::ids_to_relationships(
                    satisfies,
                    RelationshipType::Satisfies,
                ));
            }

            TypeConfig::SoftwareDetailedDesign { satisfies }
            | TypeConfig::HardwareDetailedDesign { satisfies } => {
                builder = builder.relationships(super::ids_to_relationships(
                    satisfies,
                    RelationshipType::Satisfies,
                ));
            }

            TypeConfig::Adr {
                status,
                deciders,
                justifies,
                supersedes,
                superseded_by: _,
            } => {
                let adr_status = status
                    .as_deref()
                    .and_then(|s| match s {
                        "proposed" => Some(AdrStatus::Proposed),
                        "accepted" => Some(AdrStatus::Accepted),
                        "deprecated" => Some(AdrStatus::Deprecated),
                        "superseded" => Some(AdrStatus::Superseded),
                        _ => None,
                    })
                    .unwrap_or(AdrStatus::Proposed);
                builder = builder.status(adr_status);
                if !deciders.is_empty() {
                    builder = builder.deciders(deciders.clone());
                } else {
                    // Default decider to avoid build failure
                    builder = builder.decider("TBD");
                }

                let mut rels = super::ids_to_relationships(justifies, RelationshipType::Justifies);
                rels.extend(super::ids_to_relationships(
                    supersedes,
                    RelationshipType::Supersedes,
                ));

                builder = builder.relationships(rels);
                for sup in supersedes {
                    builder = builder.supersedes(ItemId::new_unchecked(sup));
                }
            }
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

/// Parses an item type string into `ItemType` enum.
pub fn parse_item_type(type_str: &str) -> Option<ItemType> {
    match type_str.to_lowercase().as_str() {
        "solution" | "sol" => Some(ItemType::Solution),
        "use_case" | "usecase" | "uc" => Some(ItemType::UseCase),
        "scenario" | "scen" => Some(ItemType::Scenario),
        "system_requirement" | "systemrequirement" | "sysreq" => Some(ItemType::SystemRequirement),
        "system_architecture" | "systemarchitecture" | "sysarch" => {
            Some(ItemType::SystemArchitecture)
        }
        "hardware_requirement" | "hardwarerequirement" | "hwreq" => {
            Some(ItemType::HardwareRequirement)
        }
        "software_requirement" | "softwarerequirement" | "swreq" => {
            Some(ItemType::SoftwareRequirement)
        }
        "hardware_detailed_design" | "hardwaredetaileddesign" | "hwdd" => {
            Some(ItemType::HardwareDetailedDesign)
        }
        "software_detailed_design" | "softwaredetaileddesign" | "swdd" => {
            Some(ItemType::SoftwareDetailedDesign)
        }
        "architecture_decision_record" | "architecturedecisionrecord" | "adr" => {
            Some(ItemType::ArchitectureDecisionRecord)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_parse_item_type() {
        assert_eq!(parse_item_type("solution"), Some(ItemType::Solution));
        assert_eq!(parse_item_type("SOL"), Some(ItemType::Solution));
        assert_eq!(parse_item_type("use_case"), Some(ItemType::UseCase));
        assert_eq!(parse_item_type("UC"), Some(ItemType::UseCase));
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

        let opts = InitOptions::new(file_path.clone(), TypeConfig::solution())
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

        let opts = InitOptions::new(file_path.clone(), TypeConfig::use_case()).with_id("UC-001");

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

        let opts = InitOptions::new(file_path, TypeConfig::solution()).with_id("SOL-001");

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

        let opts = InitOptions::new(file_path.clone(), TypeConfig::solution())
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

        let opts =
            InitOptions::new(file_path, TypeConfig::system_requirement()).with_id("SYSREQ-001");

        let service = InitService::new();
        let result = service.init(&opts).unwrap();

        assert!(result.needs_specification);
    }

    #[test]
    fn test_needs_specification_when_provided() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");

        let type_config = TypeConfig::SystemRequirement {
            specification: Some("The system SHALL do something".to_string()),
            derives_from: Vec::new(),
            depends_on: Vec::new(),
        };

        let opts = InitOptions::new(file_path, type_config).with_id("SYSREQ-001");

        let service = InitService::new();
        let result = service.init(&opts).unwrap();

        assert!(!result.needs_specification);
    }
}

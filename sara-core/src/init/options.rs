//! Init options for item initialization.

use std::path::PathBuf;

use crate::model::ItemType;

/// Options for initializing a new item or adding frontmatter to an existing file.
#[derive(Debug, Clone)]
pub struct InitOptions {
    /// The file path to create or update.
    pub file: PathBuf,
    /// The item type.
    pub item_type: ItemType,
    /// Optional ID (will be auto-generated if not provided).
    pub id: Option<String>,
    /// Optional name (will be extracted from file or generated if not provided).
    pub name: Option<String>,
    /// Optional description.
    pub description: Option<String>,
    /// Upstream references (refines) - valid for use_case and scenario.
    pub refines: Vec<String>,
    /// Upstream references (derives_from) - valid for requirement types.
    pub derives_from: Vec<String>,
    /// Upstream references (satisfies) - valid for architecture and design types.
    pub satisfies: Vec<String>,
    /// Peer dependencies (depends_on) - valid for requirement types.
    pub depends_on: Vec<String>,
    /// Specification text - valid for requirement types.
    pub specification: Option<String>,
    /// Target platform - valid for system_architecture.
    pub platform: Option<String>,
    /// Whether to overwrite existing frontmatter.
    pub force: bool,
}

impl InitOptions {
    /// Creates new init options with required fields.
    pub fn new(file: PathBuf, item_type: ItemType) -> Self {
        Self {
            file,
            item_type,
            id: None,
            name: None,
            description: None,
            refines: Vec::new(),
            derives_from: Vec::new(),
            satisfies: Vec::new(),
            depends_on: Vec::new(),
            specification: None,
            platform: None,
            force: false,
        }
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

    /// Sets the refines references.
    pub fn with_refines(mut self, refines: Vec<String>) -> Self {
        self.refines = refines;
        self
    }

    /// Sets the derives_from references.
    pub fn with_derives_from(mut self, derives_from: Vec<String>) -> Self {
        self.derives_from = derives_from;
        self
    }

    /// Sets the satisfies references.
    pub fn with_satisfies(mut self, satisfies: Vec<String>) -> Self {
        self.satisfies = satisfies;
        self
    }

    /// Sets the depends_on references (peer dependencies).
    pub fn with_depends_on(mut self, depends_on: Vec<String>) -> Self {
        self.depends_on = depends_on;
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

    /// Sets the force flag.
    pub fn with_force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }
}

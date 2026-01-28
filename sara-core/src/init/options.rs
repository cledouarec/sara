//! Init options for item initialization.

use std::path::PathBuf;

use crate::model::ItemType;

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

    /// Creates a Solution config.
    pub fn solution() -> Self {
        TypeConfig::Solution
    }

    /// Creates a UseCase config.
    pub fn use_case() -> Self {
        TypeConfig::UseCase {
            refines: Vec::new(),
        }
    }

    /// Creates a Scenario config.
    pub fn scenario() -> Self {
        TypeConfig::Scenario {
            refines: Vec::new(),
        }
    }

    /// Creates a SystemRequirement config.
    pub fn system_requirement() -> Self {
        TypeConfig::SystemRequirement {
            specification: None,
            derives_from: Vec::new(),
            depends_on: Vec::new(),
        }
    }

    /// Creates a SystemArchitecture config.
    pub fn system_architecture() -> Self {
        TypeConfig::SystemArchitecture {
            platform: None,
            satisfies: Vec::new(),
        }
    }

    /// Creates a SoftwareRequirement config.
    pub fn software_requirement() -> Self {
        TypeConfig::SoftwareRequirement {
            specification: None,
            derives_from: Vec::new(),
            depends_on: Vec::new(),
        }
    }

    /// Creates a HardwareRequirement config.
    pub fn hardware_requirement() -> Self {
        TypeConfig::HardwareRequirement {
            specification: None,
            derives_from: Vec::new(),
            depends_on: Vec::new(),
        }
    }

    /// Creates a SoftwareDetailedDesign config.
    pub fn software_detailed_design() -> Self {
        TypeConfig::SoftwareDetailedDesign {
            satisfies: Vec::new(),
        }
    }

    /// Creates a HardwareDetailedDesign config.
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

    /// Creates a TypeConfig from an ItemType with default values.
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

impl InitOptions {
    /// Creates new init options with the specified file and type configuration.
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

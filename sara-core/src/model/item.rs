//! Item types and structures for the knowledge graph.

#![allow(clippy::result_large_err)]

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::error::ValidationError;
use crate::model::FieldName;

/// Represents the type of item in the knowledge graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemType {
    Solution,
    UseCase,
    Scenario,
    SystemRequirement,
    SystemArchitecture,
    HardwareRequirement,
    SoftwareRequirement,
    HardwareDetailedDesign,
    SoftwareDetailedDesign,
    ArchitectureDecisionRecord,
}

/// ADR lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdrStatus {
    /// Decision is under consideration, not yet finalized.
    Proposed,
    /// Decision has been approved and is in effect.
    Accepted,
    /// Decision is no longer recommended but not replaced.
    Deprecated,
    /// Decision has been replaced by a newer ADR.
    Superseded,
}

impl AdrStatus {
    /// Returns the display name for this status.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Proposed => "Proposed",
            Self::Accepted => "Accepted",
            Self::Deprecated => "Deprecated",
            Self::Superseded => "Superseded",
        }
    }

    /// Returns the YAML value (snake_case string) for this status.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Accepted => "accepted",
            Self::Deprecated => "deprecated",
            Self::Superseded => "superseded",
        }
    }

    /// Returns all possible ADR status values.
    #[must_use]
    pub const fn all() -> &'static [AdrStatus] {
        &[
            Self::Proposed,
            Self::Accepted,
            Self::Deprecated,
            Self::Superseded,
        ]
    }
}

impl fmt::Display for AdrStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl ItemType {
    /// Returns all item types in hierarchy order (upstream to downstream).
    #[must_use]
    pub const fn all() -> &'static [ItemType] {
        &[
            Self::Solution,
            Self::UseCase,
            Self::Scenario,
            Self::SystemRequirement,
            Self::SystemArchitecture,
            Self::HardwareRequirement,
            Self::SoftwareRequirement,
            Self::HardwareDetailedDesign,
            Self::SoftwareDetailedDesign,
            Self::ArchitectureDecisionRecord,
        ]
    }

    /// Returns the display name for this item type.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Solution => "Solution",
            Self::UseCase => "Use Case",
            Self::Scenario => "Scenario",
            Self::SystemRequirement => "System Requirement",
            Self::SystemArchitecture => "System Architecture",
            Self::HardwareRequirement => "Hardware Requirement",
            Self::SoftwareRequirement => "Software Requirement",
            Self::HardwareDetailedDesign => "Hardware Detailed Design",
            Self::SoftwareDetailedDesign => "Software Detailed Design",
            Self::ArchitectureDecisionRecord => "Architecture Decision Record",
        }
    }

    /// Returns the common ID prefix for this item type.
    #[must_use]
    pub const fn prefix(&self) -> &'static str {
        match self {
            Self::Solution => "SOL",
            Self::UseCase => "UC",
            Self::Scenario => "SCEN",
            Self::SystemRequirement => "SYSREQ",
            Self::SystemArchitecture => "SYSARCH",
            Self::HardwareRequirement => "HWREQ",
            Self::SoftwareRequirement => "SWREQ",
            Self::HardwareDetailedDesign => "HWDD",
            Self::SoftwareDetailedDesign => "SWDD",
            Self::ArchitectureDecisionRecord => "ADR",
        }
    }

    /// Returns the item types that accept the refines field.
    #[must_use]
    pub const fn refines_types() -> &'static [ItemType] {
        &[Self::UseCase, Self::Scenario]
    }

    /// Returns true if this item type requires the refines field.
    #[must_use]
    pub fn requires_refines(&self) -> bool {
        Self::refines_types().contains(self)
    }

    /// Returns the item types that accept the derives_from field.
    #[must_use]
    pub const fn derives_from_types() -> &'static [ItemType] {
        &[
            Self::SystemRequirement,
            Self::HardwareRequirement,
            Self::SoftwareRequirement,
        ]
    }

    /// Returns true if this item type requires the derives_from field.
    #[must_use]
    pub fn requires_derives_from(&self) -> bool {
        Self::derives_from_types().contains(self)
    }

    /// Returns the item types that accept the satisfies field.
    #[must_use]
    pub const fn satisfies_types() -> &'static [ItemType] {
        &[
            Self::SystemArchitecture,
            Self::HardwareDetailedDesign,
            Self::SoftwareDetailedDesign,
        ]
    }

    /// Returns true if this item type requires the satisfies field.
    #[must_use]
    pub fn requires_satisfies(&self) -> bool {
        Self::satisfies_types().contains(self)
    }

    /// Returns the item types that accept the specification field.
    #[must_use]
    pub const fn specification_types() -> &'static [ItemType] {
        &[
            Self::SystemRequirement,
            Self::HardwareRequirement,
            Self::SoftwareRequirement,
        ]
    }

    /// Returns true if this item type requires/accepts a specification field.
    #[must_use]
    pub fn requires_specification(&self) -> bool {
        Self::specification_types().contains(self)
    }

    /// Returns the item types that accept the platform field.
    #[must_use]
    pub const fn platform_types() -> &'static [ItemType] {
        &[Self::SystemArchitecture]
    }

    /// Returns true if this item type accepts the platform field.
    #[must_use]
    pub fn accepts_platform(&self) -> bool {
        Self::platform_types().contains(self)
    }

    /// Returns the item types that accept the depends_on field (peer dependencies).
    #[must_use]
    pub const fn depends_on_types() -> &'static [ItemType] {
        &[
            Self::SystemRequirement,
            Self::HardwareRequirement,
            Self::SoftwareRequirement,
        ]
    }

    /// Returns true if this item type accepts the depends_on field (peer dependencies).
    #[must_use]
    pub fn supports_depends_on(&self) -> bool {
        Self::depends_on_types().contains(self)
    }

    /// Returns true if this is a root item type (Solution).
    #[must_use]
    pub const fn is_root(&self) -> bool {
        matches!(self, Self::Solution)
    }

    /// Returns true if this is an Architecture Decision Record type.
    #[must_use]
    pub const fn requires_deciders(&self) -> bool {
        matches!(self, Self::ArchitectureDecisionRecord)
    }

    /// Returns true if this item type supports the status field (ADR only).
    #[must_use]
    pub const fn supports_status(&self) -> bool {
        matches!(self, Self::ArchitectureDecisionRecord)
    }

    /// Returns true if this item type supports the supersedes field (ADR peer relationship).
    #[must_use]
    pub const fn supports_supersedes(&self) -> bool {
        matches!(self, Self::ArchitectureDecisionRecord)
    }

    /// Returns the required parent item type for this type, if any.
    /// Solution has no parent (root of the hierarchy).
    #[must_use]
    pub const fn required_parent_type(&self) -> Option<ItemType> {
        match self {
            Self::Solution => None,
            Self::UseCase => Some(Self::Solution),
            Self::Scenario => Some(Self::UseCase),
            Self::SystemRequirement => Some(Self::Scenario),
            Self::SystemArchitecture => Some(Self::SystemRequirement),
            Self::HardwareRequirement => Some(Self::SystemArchitecture),
            Self::SoftwareRequirement => Some(Self::SystemArchitecture),
            Self::HardwareDetailedDesign => Some(Self::HardwareRequirement),
            Self::SoftwareDetailedDesign => Some(Self::SoftwareRequirement),
            Self::ArchitectureDecisionRecord => None,
        }
    }

    /// Returns the upstream traceability field for this item type.
    #[must_use]
    pub const fn traceability_field(&self) -> Option<FieldName> {
        match self {
            Self::Solution => None,
            Self::UseCase | Self::Scenario => Some(FieldName::Refines),
            Self::SystemRequirement | Self::HardwareRequirement | Self::SoftwareRequirement => {
                Some(FieldName::DerivesFrom)
            }
            Self::SystemArchitecture
            | Self::HardwareDetailedDesign
            | Self::SoftwareDetailedDesign => Some(FieldName::Satisfies),
            Self::ArchitectureDecisionRecord => Some(FieldName::Justifies),
        }
    }

    /// Returns the YAML value (snake_case string) for this item type.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Solution => "solution",
            Self::UseCase => "use_case",
            Self::Scenario => "scenario",
            Self::SystemRequirement => "system_requirement",
            Self::SystemArchitecture => "system_architecture",
            Self::HardwareRequirement => "hardware_requirement",
            Self::SoftwareRequirement => "software_requirement",
            Self::HardwareDetailedDesign => "hardware_detailed_design",
            Self::SoftwareDetailedDesign => "software_detailed_design",
            Self::ArchitectureDecisionRecord => "architecture_decision_record",
        }
    }

    /// Returns all traceability configurations for this item type.
    ///
    /// Most item types have a single traceability link (e.g., refines, satisfies).
    /// Requirement types have two: derives_from (hierarchical) and depends_on (peer).
    /// Solution has no parent and returns an empty vec.
    #[must_use]
    pub fn traceability_configs(&self) -> Vec<TraceabilityConfig> {
        match self {
            ItemType::Solution => vec![],
            ItemType::UseCase => vec![TraceabilityConfig {
                relationship_field: FieldName::Refines,
                target_type: ItemType::Solution,
            }],
            ItemType::Scenario => vec![TraceabilityConfig {
                relationship_field: FieldName::Refines,
                target_type: ItemType::UseCase,
            }],
            ItemType::SystemRequirement => vec![
                TraceabilityConfig {
                    relationship_field: FieldName::DerivesFrom,
                    target_type: ItemType::Scenario,
                },
                TraceabilityConfig {
                    relationship_field: FieldName::DependsOn,
                    target_type: ItemType::SystemRequirement,
                },
            ],
            ItemType::SystemArchitecture => vec![TraceabilityConfig {
                relationship_field: FieldName::Satisfies,
                target_type: ItemType::SystemRequirement,
            }],
            ItemType::HardwareRequirement => vec![
                TraceabilityConfig {
                    relationship_field: FieldName::DerivesFrom,
                    target_type: ItemType::SystemArchitecture,
                },
                TraceabilityConfig {
                    relationship_field: FieldName::DependsOn,
                    target_type: ItemType::HardwareRequirement,
                },
            ],
            ItemType::SoftwareRequirement => vec![
                TraceabilityConfig {
                    relationship_field: FieldName::DerivesFrom,
                    target_type: ItemType::SystemArchitecture,
                },
                TraceabilityConfig {
                    relationship_field: FieldName::DependsOn,
                    target_type: ItemType::SoftwareRequirement,
                },
            ],
            ItemType::HardwareDetailedDesign => vec![TraceabilityConfig {
                relationship_field: FieldName::Satisfies,
                target_type: ItemType::HardwareRequirement,
            }],
            ItemType::SoftwareDetailedDesign => vec![TraceabilityConfig {
                relationship_field: FieldName::Satisfies,
                target_type: ItemType::SoftwareRequirement,
            }],
            ItemType::ArchitectureDecisionRecord => vec![
                TraceabilityConfig {
                    relationship_field: FieldName::Justifies,
                    target_type: ItemType::SystemArchitecture,
                },
                TraceabilityConfig {
                    relationship_field: FieldName::Justifies,
                    target_type: ItemType::SoftwareDetailedDesign,
                },
                TraceabilityConfig {
                    relationship_field: FieldName::Justifies,
                    target_type: ItemType::HardwareDetailedDesign,
                },
            ],
        }
    }
}

/// Configuration for traceability relationships.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TraceabilityConfig {
    /// The relationship field (refines, derives_from, satisfies, depends_on).
    pub relationship_field: FieldName,
    /// The target item type to link to (parent for hierarchical, same type for peers).
    pub target_type: ItemType,
}

impl fmt::Display for ItemType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Unique identifier for an item across all repositories.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ItemId(String);

impl ItemId {
    /// Creates a new ItemId, validating format.
    pub fn new(id: impl Into<String>) -> Result<Self, ValidationError> {
        let id = id.into();
        if id.is_empty() {
            return Err(ValidationError::InvalidId {
                id: id.clone(),
                reason: "Item ID cannot be empty".to_string(),
            });
        }

        // Validate: alphanumeric, hyphens, and underscores only
        if !id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(ValidationError::InvalidId {
                id: id.clone(),
                reason:
                    "Item ID must contain only alphanumeric characters, hyphens, and underscores"
                        .to_string(),
            });
        }

        Ok(Self(id))
    }

    /// Creates a new ItemId without validation.
    ///
    /// Use this when parsing from trusted sources where IDs have already been
    /// validated or when the ID format is known to be valid.
    pub fn new_unchecked(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns the raw identifier string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ItemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for ItemId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Upstream relationship references (this item points to parents).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpstreamRefs {
    /// Items this item refines (for UseCase, Scenario).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refines: Vec<ItemId>,

    /// Items this item derives from (for SystemRequirement, HW/SW Requirement).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub derives_from: Vec<ItemId>,

    /// Items this item satisfies (for SystemArchitecture, HW/SW DetailedDesign).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub satisfies: Vec<ItemId>,

    /// Design artifacts this ADR justifies (for ArchitectureDecisionRecord).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub justifies: Vec<ItemId>,
}

impl UpstreamRefs {
    /// Returns an iterator over all upstream item IDs.
    pub fn all_ids(&self) -> impl Iterator<Item = &ItemId> {
        self.refines
            .iter()
            .chain(self.derives_from.iter())
            .chain(self.satisfies.iter())
            .chain(self.justifies.iter())
    }

    /// Returns true if there are no upstream references.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.refines.is_empty()
            && self.derives_from.is_empty()
            && self.satisfies.is_empty()
            && self.justifies.is_empty()
    }
}

/// Downstream relationship references (this item points to children).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DownstreamRefs {
    /// Items that refine this item (for Solution, UseCase).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub is_refined_by: Vec<ItemId>,

    /// Items derived from this item (for Scenario, SystemArchitecture).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub derives: Vec<ItemId>,

    /// Items that satisfy this item (for SystemRequirement, HW/SW Requirement).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub is_satisfied_by: Vec<ItemId>,

    /// ADRs that justify this item (for design artifacts: SYSARCH, SWDD, HWDD).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub justified_by: Vec<ItemId>,
}

impl DownstreamRefs {
    /// Returns an iterator over all downstream item IDs.
    pub fn all_ids(&self) -> impl Iterator<Item = &ItemId> {
        self.is_refined_by
            .iter()
            .chain(self.derives.iter())
            .chain(self.is_satisfied_by.iter())
            .chain(self.justified_by.iter())
    }

    /// Returns true if there are no downstream references.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.is_refined_by.is_empty()
            && self.derives.is_empty()
            && self.is_satisfied_by.is_empty()
            && self.justified_by.is_empty()
    }
}

/// Type-specific attributes for items in the knowledge graph.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(tag = "_attr_type")]
pub enum ItemAttributes {
    /// Solution - no type-specific attributes.
    #[serde(rename = "solution")]
    #[default]
    Solution,

    /// Use Case - no type-specific attributes beyond upstream refs.
    #[serde(rename = "use_case")]
    UseCase,

    /// Scenario - no type-specific attributes beyond upstream refs.
    #[serde(rename = "scenario")]
    Scenario,

    /// System Requirement with specification and peer dependencies.
    #[serde(rename = "system_requirement")]
    SystemRequirement {
        /// Specification statement.
        specification: String,
        /// Peer dependencies.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        depends_on: Vec<ItemId>,
    },

    /// System Architecture with platform.
    /// Note: `justified_by` is now in DownstreamRefs.
    #[serde(rename = "system_architecture")]
    SystemArchitecture {
        /// Target platform.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        platform: Option<String>,
    },

    /// Software Requirement with specification and peer dependencies.
    #[serde(rename = "software_requirement")]
    SoftwareRequirement {
        /// Specification statement.
        specification: String,
        /// Peer dependencies.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        depends_on: Vec<ItemId>,
    },

    /// Hardware Requirement with specification and peer dependencies.
    #[serde(rename = "hardware_requirement")]
    HardwareRequirement {
        /// Specification statement.
        specification: String,
        /// Peer dependencies.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        depends_on: Vec<ItemId>,
    },

    /// Software Detailed Design.
    /// Note: `justified_by` is now in DownstreamRefs.
    #[serde(rename = "software_detailed_design")]
    SoftwareDetailedDesign,

    /// Hardware Detailed Design.
    /// Note: `justified_by` is now in DownstreamRefs.
    #[serde(rename = "hardware_detailed_design")]
    HardwareDetailedDesign,

    /// Architecture Decision Record with ADR-specific fields.
    /// Note: `justifies` is now in UpstreamRefs, `superseded_by` is auto-generated inverse.
    #[serde(rename = "architecture_decision_record")]
    Adr {
        /// ADR lifecycle status.
        status: AdrStatus,
        /// List of people involved in the decision.
        deciders: Vec<String>,
        /// Older ADRs this decision supersedes (peer relationship).
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        supersedes: Vec<ItemId>,
    },
}

impl ItemAttributes {
    /// Creates default attributes for the given item type.
    #[must_use]
    pub fn for_type(item_type: ItemType) -> Self {
        match item_type {
            ItemType::Solution => ItemAttributes::Solution,
            ItemType::UseCase => ItemAttributes::UseCase,
            ItemType::Scenario => ItemAttributes::Scenario,
            ItemType::SystemRequirement => ItemAttributes::SystemRequirement {
                specification: String::new(),
                depends_on: Vec::new(),
            },
            ItemType::SystemArchitecture => ItemAttributes::SystemArchitecture { platform: None },
            ItemType::SoftwareRequirement => ItemAttributes::SoftwareRequirement {
                specification: String::new(),
                depends_on: Vec::new(),
            },
            ItemType::HardwareRequirement => ItemAttributes::HardwareRequirement {
                specification: String::new(),
                depends_on: Vec::new(),
            },
            ItemType::SoftwareDetailedDesign => ItemAttributes::SoftwareDetailedDesign,
            ItemType::HardwareDetailedDesign => ItemAttributes::HardwareDetailedDesign,
            ItemType::ArchitectureDecisionRecord => ItemAttributes::Adr {
                status: AdrStatus::Proposed,
                deciders: Vec::new(),
                supersedes: Vec::new(),
            },
        }
    }

    /// Returns the specification if this is a requirement type.
    #[must_use]
    pub fn specification(&self) -> Option<&String> {
        match self {
            Self::SystemRequirement { specification, .. }
            | Self::SoftwareRequirement { specification, .. }
            | Self::HardwareRequirement { specification, .. } => Some(specification),
            _ => None,
        }
    }

    /// Returns the depends_on references if this is a requirement type.
    #[must_use]
    pub fn depends_on(&self) -> &[ItemId] {
        match self {
            Self::SystemRequirement { depends_on, .. }
            | Self::SoftwareRequirement { depends_on, .. }
            | Self::HardwareRequirement { depends_on, .. } => depends_on,
            _ => &[],
        }
    }

    /// Returns the platform if this is a SystemArchitecture.
    #[must_use]
    pub fn platform(&self) -> Option<&String> {
        match self {
            Self::SystemArchitecture { platform, .. } => platform.as_ref(),
            _ => None,
        }
    }

    /// Returns the ADR status if this is an ADR.
    #[must_use]
    pub fn status(&self) -> Option<AdrStatus> {
        match self {
            Self::Adr { status, .. } => Some(*status),
            _ => None,
        }
    }

    /// Returns the deciders if this is an ADR.
    #[must_use]
    pub fn deciders(&self) -> &[String] {
        match self {
            Self::Adr { deciders, .. } => deciders,
            _ => &[],
        }
    }

    /// Returns the supersedes references if this is an ADR.
    #[must_use]
    pub fn supersedes(&self) -> &[ItemId] {
        match self {
            Self::Adr { supersedes, .. } => supersedes,
            _ => &[],
        }
    }
}

use crate::model::metadata::SourceLocation;

/// Represents a single document/node in the knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    /// Unique identifier.
    pub id: ItemId,

    /// Type of this item.
    pub item_type: ItemType,

    /// Human-readable name.
    pub name: String,

    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Source file location.
    pub source: SourceLocation,

    /// Upstream relationships (toward Solution).
    #[serde(default)]
    pub upstream: UpstreamRefs,

    /// Downstream relationships (toward Detailed Designs).
    #[serde(default)]
    pub downstream: DownstreamRefs,

    /// Type-specific attributes.
    #[serde(default)]
    pub attributes: ItemAttributes,
}

impl Item {
    /// Returns an iterator over all referenced item IDs (upstream, downstream, and peer).
    pub fn all_references(&self) -> impl Iterator<Item = &ItemId> {
        // Upstream and downstream references
        let upstream_downstream = self.upstream.all_ids().chain(self.downstream.all_ids());

        // Peer references from attributes (depends_on for requirements, supersedes for ADRs)
        let peer_refs: Box<dyn Iterator<Item = &ItemId>> = match &self.attributes {
            ItemAttributes::SystemRequirement { depends_on, .. }
            | ItemAttributes::SoftwareRequirement { depends_on, .. }
            | ItemAttributes::HardwareRequirement { depends_on, .. } => Box::new(depends_on.iter()),
            ItemAttributes::Adr { supersedes, .. } => Box::new(supersedes.iter()),
            _ => Box::new(std::iter::empty()),
        };

        upstream_downstream.chain(peer_refs)
    }
}

/// Builder for constructing Item instances from parsed frontmatter.
#[derive(Debug, Default)]
pub struct ItemBuilder {
    id: Option<ItemId>,
    item_type: Option<ItemType>,
    name: Option<String>,
    description: Option<String>,
    source: Option<SourceLocation>,
    upstream: UpstreamRefs,
    downstream: DownstreamRefs,
    // Temporary storage for attributes before we know the type
    specification: Option<String>,
    platform: Option<String>,
    depends_on: Vec<ItemId>,
    status: Option<AdrStatus>,
    deciders: Vec<String>,
    supersedes: Vec<ItemId>,
}

impl ItemBuilder {
    /// Creates a new ItemBuilder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the item ID.
    pub fn id(mut self, id: ItemId) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the item type.
    pub fn item_type(mut self, item_type: ItemType) -> Self {
        self.item_type = Some(item_type);
        self
    }

    /// Sets the item name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the item description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Sets the source location.
    pub fn source(mut self, source: SourceLocation) -> Self {
        self.source = Some(source);
        self
    }

    /// Sets the upstream references.
    pub fn upstream(mut self, upstream: UpstreamRefs) -> Self {
        self.upstream = upstream;
        self
    }

    /// Sets the downstream references.
    pub fn downstream(mut self, downstream: DownstreamRefs) -> Self {
        self.downstream = downstream;
        self
    }

    /// Sets the specification text (for requirement types).
    pub fn specification(mut self, spec: impl Into<String>) -> Self {
        self.specification = Some(spec.into());
        self
    }

    /// Sets the platform (for SystemArchitecture).
    pub fn platform(mut self, platform: impl Into<String>) -> Self {
        self.platform = Some(platform.into());
        self
    }

    /// Adds a dependency (for requirement types).
    pub fn depends_on(mut self, id: ItemId) -> Self {
        self.depends_on.push(id);
        self
    }

    /// Sets the ADR status.
    pub fn status(mut self, status: AdrStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Adds a decider (for ADR).
    pub fn decider(mut self, decider: impl Into<String>) -> Self {
        self.deciders.push(decider.into());
        self
    }

    /// Sets the deciders (for ADR).
    pub fn deciders(mut self, deciders: Vec<String>) -> Self {
        self.deciders = deciders;
        self
    }

    /// Adds a superseded ADR ID.
    pub fn supersedes(mut self, id: ItemId) -> Self {
        self.supersedes.push(id);
        self
    }

    /// Sets the supersedes references (for ADR).
    pub fn supersedes_all(mut self, ids: Vec<ItemId>) -> Self {
        self.supersedes = ids;
        self
    }

    /// Sets the attributes directly.
    pub fn attributes(mut self, attrs: ItemAttributes) -> Self {
        // Extract values from the attributes enum
        match attrs {
            ItemAttributes::Solution
            | ItemAttributes::UseCase
            | ItemAttributes::Scenario
            | ItemAttributes::SoftwareDetailedDesign
            | ItemAttributes::HardwareDetailedDesign => {}
            ItemAttributes::SystemRequirement {
                specification,
                depends_on,
            } => {
                self.specification = Some(specification);
                self.depends_on = depends_on;
            }
            ItemAttributes::SystemArchitecture { platform } => {
                self.platform = platform;
            }
            ItemAttributes::SoftwareRequirement {
                specification,
                depends_on,
            } => {
                self.specification = Some(specification);
                self.depends_on = depends_on;
            }
            ItemAttributes::HardwareRequirement {
                specification,
                depends_on,
            } => {
                self.specification = Some(specification);
                self.depends_on = depends_on;
            }
            ItemAttributes::Adr {
                status,
                deciders,
                supersedes,
            } => {
                self.status = Some(status);
                self.deciders = deciders;
                self.supersedes = supersedes;
            }
        }
        self
    }

    /// Validates and returns the specification, returning an error if missing.
    fn require_specification(&self, file: &str) -> Result<String, ValidationError> {
        self.specification
            .clone()
            .ok_or_else(|| ValidationError::MissingField {
                field: "specification".to_string(),
                file: file.to_string(),
            })
    }

    /// Builds the attributes for the given item type.
    fn build_attributes(
        &self,
        item_type: ItemType,
        file: &str,
    ) -> Result<ItemAttributes, ValidationError> {
        match item_type {
            // Simple types with no additional attributes
            ItemType::Solution => Ok(ItemAttributes::Solution),
            ItemType::UseCase => Ok(ItemAttributes::UseCase),
            ItemType::Scenario => Ok(ItemAttributes::Scenario),
            ItemType::SoftwareDetailedDesign => Ok(ItemAttributes::SoftwareDetailedDesign),
            ItemType::HardwareDetailedDesign => Ok(ItemAttributes::HardwareDetailedDesign),

            // Architecture with optional platform
            ItemType::SystemArchitecture => Ok(ItemAttributes::SystemArchitecture {
                platform: self.platform.clone(),
            }),

            // Requirement types with specification and dependencies
            ItemType::SystemRequirement => Ok(ItemAttributes::SystemRequirement {
                specification: self.require_specification(file)?,
                depends_on: self.depends_on.clone(),
            }),
            ItemType::SoftwareRequirement => Ok(ItemAttributes::SoftwareRequirement {
                specification: self.require_specification(file)?,
                depends_on: self.depends_on.clone(),
            }),
            ItemType::HardwareRequirement => Ok(ItemAttributes::HardwareRequirement {
                specification: self.require_specification(file)?,
                depends_on: self.depends_on.clone(),
            }),

            // ADR with status, deciders, and supersedes
            ItemType::ArchitectureDecisionRecord => {
                let status = self.status.ok_or_else(|| ValidationError::MissingField {
                    field: "status".to_string(),
                    file: file.to_string(),
                })?;
                if self.deciders.is_empty() {
                    return Err(ValidationError::MissingField {
                        field: "deciders".to_string(),
                        file: file.to_string(),
                    });
                }
                Ok(ItemAttributes::Adr {
                    status,
                    deciders: self.deciders.clone(),
                    supersedes: self.supersedes.clone(),
                })
            }
        }
    }

    /// Builds the Item, returning an error if required fields are missing.
    pub fn build(self) -> Result<Item, ValidationError> {
        let id = self
            .id
            .clone()
            .ok_or_else(|| ValidationError::MissingField {
                field: "id".to_string(),
                file: self
                    .source
                    .as_ref()
                    .map(|s| s.file_path.display().to_string())
                    .unwrap_or_default(),
            })?;

        let item_type = self
            .item_type
            .ok_or_else(|| ValidationError::MissingField {
                field: "type".to_string(),
                file: self
                    .source
                    .as_ref()
                    .map(|s| s.file_path.display().to_string())
                    .unwrap_or_default(),
            })?;

        let name = self
            .name
            .clone()
            .ok_or_else(|| ValidationError::MissingField {
                field: "name".to_string(),
                file: self
                    .source
                    .as_ref()
                    .map(|s| s.file_path.display().to_string())
                    .unwrap_or_default(),
            })?;

        let source = self
            .source
            .clone()
            .ok_or_else(|| ValidationError::MissingField {
                field: "source".to_string(),
                file: String::new(),
            })?;

        let file_path = source.file_path.display().to_string();
        let attributes = self.build_attributes(item_type, &file_path)?;

        Ok(Item {
            id,
            item_type,
            name,
            description: self.description,
            source,
            upstream: self.upstream,
            downstream: self.downstream,
            attributes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_item_id_valid() {
        assert!(ItemId::new("SOL-001").is_ok());
        assert!(ItemId::new("UC_002").is_ok());
        assert!(ItemId::new("SYSREQ-123-A").is_ok());
    }

    #[test]
    fn test_item_id_invalid() {
        assert!(ItemId::new("").is_err());
        assert!(ItemId::new("SOL 001").is_err());
        assert!(ItemId::new("SOL.001").is_err());
    }

    #[test]
    fn test_item_type_display() {
        assert_eq!(ItemType::Solution.display_name(), "Solution");
        assert_eq!(
            ItemType::SystemRequirement.display_name(),
            "System Requirement"
        );
    }

    #[test]
    fn test_item_type_requires_specification() {
        assert!(ItemType::SystemRequirement.requires_specification());
        assert!(ItemType::HardwareRequirement.requires_specification());
        assert!(ItemType::SoftwareRequirement.requires_specification());
        assert!(!ItemType::Solution.requires_specification());
        assert!(!ItemType::Scenario.requires_specification());
    }

    #[test]
    fn test_item_builder() {
        let source = SourceLocation {
            repository: PathBuf::from("/repo"),
            file_path: PathBuf::from("docs/SOL-001.md"),
            git_ref: None,
        };

        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("SOL-001"))
            .item_type(ItemType::Solution)
            .name("Test Solution")
            .source(source)
            .build();

        assert!(item.is_ok());
        let item = item.unwrap();
        assert_eq!(item.id.as_str(), "SOL-001");
        assert_eq!(item.name, "Test Solution");
    }
}

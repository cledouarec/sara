//! Item types and structures for the knowledge graph.

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::error::SaraError;
use crate::model::FieldName;
use crate::model::relationship::{Relationship, RelationshipType};

use super::adr::AdrStatus;

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

    /// Generates a new ID for the given type with an optional sequence number.
    ///
    /// Defaults to sequence 1 if not provided.
    #[must_use]
    pub fn generate_id(&self, sequence: Option<u32>) -> String {
        let num = sequence.unwrap_or(1);
        format!("{}-{:03}", self.prefix(), num)
    }

    /// Suggests the next sequential ID based on existing items in the graph.
    ///
    /// Finds the highest existing ID for this type and returns the next one.
    /// If no graph is provided or no items exist, returns the first ID (e.g., "SOL-001").
    #[must_use]
    pub fn suggest_next_id(&self, graph: Option<&crate::graph::KnowledgeGraph>) -> String {
        let Some(graph) = graph else {
            return self.generate_id(None);
        };

        let prefix = self.prefix();
        let max_num = graph
            .items()
            .filter(|item| item.item_type == *self)
            .filter_map(|item| {
                item.id
                    .as_str()
                    .strip_prefix(prefix)
                    .and_then(|suffix| suffix.trim_start_matches('-').parse::<u32>().ok())
            })
            .max()
            .unwrap_or(0);

        format!("{}-{:03}", prefix, max_num + 1)
    }

    /// Returns true if this item type requires the refines field.
    #[must_use]
    pub const fn requires_refines(&self) -> bool {
        matches!(self, Self::UseCase | Self::Scenario)
    }

    /// Returns true if this item type requires the derives_from field.
    #[must_use]
    pub const fn requires_derives_from(&self) -> bool {
        matches!(
            self,
            Self::SystemRequirement | Self::HardwareRequirement | Self::SoftwareRequirement
        )
    }

    /// Returns true if this item type requires the satisfies field.
    #[must_use]
    pub const fn requires_satisfies(&self) -> bool {
        matches!(
            self,
            Self::SystemArchitecture | Self::HardwareDetailedDesign | Self::SoftwareDetailedDesign
        )
    }

    /// Returns true if this item type requires/accepts a specification field.
    #[must_use]
    pub const fn requires_specification(&self) -> bool {
        matches!(
            self,
            Self::SystemRequirement | Self::HardwareRequirement | Self::SoftwareRequirement
        )
    }

    /// Returns true if this item type accepts the platform field.
    #[must_use]
    pub const fn accepts_platform(&self) -> bool {
        matches!(self, Self::SystemArchitecture)
    }

    /// Returns true if this item type accepts the depends_on field (peer dependencies).
    #[must_use]
    pub const fn supports_depends_on(&self) -> bool {
        matches!(
            self,
            Self::SystemRequirement | Self::HardwareRequirement | Self::SoftwareRequirement
        )
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
    ///
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
    pub fn new(id: impl Into<String>) -> Result<Self, SaraError> {
        let id = id.into();
        if id.is_empty() {
            return Err(SaraError::InvalidId {
                id: id.clone(),
                reason: "Item ID cannot be empty".to_string(),
            });
        }

        // Validate: alphanumeric, hyphens, and underscores only
        if !id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(SaraError::InvalidId {
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
    #[serde(rename = "software_detailed_design")]
    SoftwareDetailedDesign,

    /// Hardware Detailed Design.
    #[serde(rename = "hardware_detailed_design")]
    HardwareDetailedDesign,

    /// Architecture Decision Record with ADR-specific fields.
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

    /// All relationships from this item to other items.
    #[serde(default)]
    pub relationships: Vec<Relationship>,

    /// Type-specific attributes.
    #[serde(default)]
    pub attributes: ItemAttributes,
}

impl Item {
    /// Returns an iterator over target IDs for a specific relationship type.
    pub fn relationship_ids(&self, rel_type: RelationshipType) -> impl Iterator<Item = &ItemId> {
        self.relationships
            .iter()
            .filter(move |r| r.relationship_type == rel_type)
            .map(|r| &r.to)
    }

    /// Returns true if this item has any relationships of the given type.
    #[must_use]
    pub fn has_relationship_type(&self, rel_type: RelationshipType) -> bool {
        self.relationships
            .iter()
            .any(|r| r.relationship_type == rel_type)
    }

    /// Returns true if this item has any upstream relationships.
    #[must_use]
    pub fn has_upstream(&self) -> bool {
        self.relationships
            .iter()
            .any(|r| r.relationship_type.is_upstream())
    }

    /// Returns an iterator over all referenced item IDs (relationships and peer refs from attributes).
    pub fn all_references(&self) -> impl Iterator<Item = &ItemId> {
        let relationship_refs = self.relationships.iter().map(|r| &r.to);

        // Peer references from attributes (depends_on for requirements, supersedes for ADRs)
        let peer_refs: Box<dyn Iterator<Item = &ItemId>> = match &self.attributes {
            ItemAttributes::SystemRequirement { depends_on, .. }
            | ItemAttributes::SoftwareRequirement { depends_on, .. }
            | ItemAttributes::HardwareRequirement { depends_on, .. } => Box::new(depends_on.iter()),
            ItemAttributes::Adr { supersedes, .. } => Box::new(supersedes.iter()),
            _ => Box::new(std::iter::empty()),
        };

        relationship_refs.chain(peer_refs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_generate_id() {
        assert_eq!(ItemType::Solution.generate_id(Some(1)), "SOL-001");
        assert_eq!(ItemType::UseCase.generate_id(Some(42)), "UC-042");
        assert_eq!(ItemType::SystemRequirement.generate_id(None), "SYSREQ-001");
    }
}

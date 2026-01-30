//! Relationship types and structures for the knowledge graph.

use serde::{Deserialize, Serialize};

use super::field::FieldName;
use super::item::{ItemId, ItemType};

/// Represents the type of relationship between items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipType {
    /// Refinement: child refines parent (Scenario refines Use Case).
    Refines,
    /// Inverse of Refines: parent is refined by child.
    IsRefinedBy,
    /// Derivation: parent derives child (Scenario derives System Requirement).
    Derives,
    /// Inverse of Derives: child derives from parent.
    DerivesFrom,
    /// Satisfaction: child satisfies parent (System Architecture satisfies System Requirement).
    Satisfies,
    /// Inverse of Satisfies: parent is satisfied by child.
    IsSatisfiedBy,
    /// Dependency: Requirement depends on another Requirement of the same type.
    DependsOn,
    /// Inverse of DependsOn: Requirement is required by another.
    IsRequiredBy,
    /// Justification: ADR justifies a design artifact (SYSARCH, SWDD, HWDD).
    Justifies,
    /// Inverse of Justifies: design artifact is justified by an ADR.
    IsJustifiedBy,
    /// Supersession: newer ADR supersedes older ADR.
    Supersedes,
    /// Inverse of Supersedes: older ADR is superseded by newer ADR.
    IsSupersededBy,
}

impl RelationshipType {
    /// Get the inverse relationship type.
    #[must_use]
    pub const fn inverse(&self) -> Self {
        match self {
            Self::Refines => Self::IsRefinedBy,
            Self::IsRefinedBy => Self::Refines,
            Self::Derives => Self::DerivesFrom,
            Self::DerivesFrom => Self::Derives,
            Self::Satisfies => Self::IsSatisfiedBy,
            Self::IsSatisfiedBy => Self::Satisfies,
            Self::DependsOn => Self::IsRequiredBy,
            Self::IsRequiredBy => Self::DependsOn,
            Self::Justifies => Self::IsJustifiedBy,
            Self::IsJustifiedBy => Self::Justifies,
            Self::Supersedes => Self::IsSupersededBy,
            Self::IsSupersededBy => Self::Supersedes,
        }
    }

    /// Check if this is an upstream relationship (toward Solution).
    /// For ADRs, Justifies is considered upstream as it links ADR to design artifacts.
    #[must_use]
    pub const fn is_upstream(&self) -> bool {
        matches!(
            self,
            Self::Refines | Self::DerivesFrom | Self::Satisfies | Self::Justifies
        )
    }

    /// Check if this is a downstream relationship (toward Detailed Designs).
    #[must_use]
    pub const fn is_downstream(&self) -> bool {
        matches!(
            self,
            Self::IsRefinedBy | Self::Derives | Self::IsSatisfiedBy | Self::IsJustifiedBy
        )
    }

    /// Check if this is a peer relationship (between items of the same type).
    #[must_use]
    pub const fn is_peer(&self) -> bool {
        matches!(
            self,
            Self::DependsOn | Self::IsRequiredBy | Self::Supersedes | Self::IsSupersededBy
        )
    }

    /// Check if this is a primary relationship (not an inverse).
    ///
    /// Primary relationships are the declared direction:
    /// - Refines, DerivesFrom, Satisfies, Justifies (upstream)
    /// - DependsOn, Supersedes (peer, primary)
    ///
    /// Inverse relationships exist only for graph traversal and should not
    /// be considered when checking for cycles.
    #[must_use]
    pub const fn is_primary(&self) -> bool {
        matches!(
            self,
            Self::Refines
                | Self::DerivesFrom
                | Self::Satisfies
                | Self::Justifies
                | Self::DependsOn
                | Self::Supersedes
        )
    }

    /// Returns the corresponding FieldName for this relationship type.
    #[must_use]
    pub const fn field_name(&self) -> FieldName {
        match self {
            Self::Refines => FieldName::Refines,
            Self::IsRefinedBy => FieldName::IsRefinedBy,
            Self::Derives => FieldName::Derives,
            Self::DerivesFrom => FieldName::DerivesFrom,
            Self::Satisfies => FieldName::Satisfies,
            Self::IsSatisfiedBy => FieldName::IsSatisfiedBy,
            Self::DependsOn => FieldName::DependsOn,
            Self::IsRequiredBy => FieldName::IsRequiredBy,
            Self::Justifies => FieldName::Justifies,
            Self::IsJustifiedBy => FieldName::JustifiedBy,
            Self::Supersedes => FieldName::Supersedes,
            Self::IsSupersededBy => FieldName::SupersededBy,
        }
    }
}

impl std::fmt::Display for RelationshipType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.field_name().as_str())
    }
}

/// Represents a relationship from an Item to another item.
/// The source item is implied by the Item containing this relationship.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relationship {
    /// Target item ID.
    pub to: ItemId,
    /// Type of relationship.
    pub relationship_type: RelationshipType,
}

impl Relationship {
    /// Creates a new relationship.
    #[must_use]
    pub fn new(to: ItemId, relationship_type: RelationshipType) -> Self {
        Self {
            to,
            relationship_type,
        }
    }

    /// Returns the target item ID.
    #[must_use]
    pub fn target(&self) -> &ItemId {
        &self.to
    }

    /// Returns the relationship type.
    #[must_use]
    pub fn rel_type(&self) -> RelationshipType {
        self.relationship_type
    }
}

/// Valid relationship rules based on item types.
pub struct RelationshipRules;

impl RelationshipRules {
    /// Returns the valid upstream relationship types for a given item type.
    #[must_use]
    pub fn valid_upstream_for(item_type: ItemType) -> Option<(RelationshipType, Vec<ItemType>)> {
        match item_type {
            ItemType::Solution => None,
            ItemType::UseCase => Some((RelationshipType::Refines, vec![ItemType::Solution])),
            ItemType::Scenario => Some((RelationshipType::Refines, vec![ItemType::UseCase])),
            ItemType::SystemRequirement => {
                Some((RelationshipType::DerivesFrom, vec![ItemType::Scenario]))
            }
            ItemType::SystemArchitecture => Some((
                RelationshipType::Satisfies,
                vec![ItemType::SystemRequirement],
            )),
            ItemType::HardwareRequirement => Some((
                RelationshipType::DerivesFrom,
                vec![ItemType::SystemArchitecture],
            )),
            ItemType::SoftwareRequirement => Some((
                RelationshipType::DerivesFrom,
                vec![ItemType::SystemArchitecture],
            )),
            ItemType::HardwareDetailedDesign => Some((
                RelationshipType::Satisfies,
                vec![ItemType::HardwareRequirement],
            )),
            ItemType::SoftwareDetailedDesign => Some((
                RelationshipType::Satisfies,
                vec![ItemType::SoftwareRequirement],
            )),
            ItemType::ArchitectureDecisionRecord => Some((
                RelationshipType::Justifies,
                vec![
                    ItemType::SystemArchitecture,
                    ItemType::SoftwareDetailedDesign,
                    ItemType::HardwareDetailedDesign,
                ],
            )),
        }
    }

    /// Returns the valid downstream relationship types for a given item type.
    #[must_use]
    pub fn valid_downstream_for(item_type: ItemType) -> Option<(RelationshipType, Vec<ItemType>)> {
        match item_type {
            ItemType::Solution => Some((RelationshipType::IsRefinedBy, vec![ItemType::UseCase])),
            ItemType::UseCase => Some((RelationshipType::IsRefinedBy, vec![ItemType::Scenario])),
            ItemType::Scenario => {
                Some((RelationshipType::Derives, vec![ItemType::SystemRequirement]))
            }
            ItemType::SystemRequirement => Some((
                RelationshipType::IsSatisfiedBy,
                vec![ItemType::SystemArchitecture],
            )),
            ItemType::SystemArchitecture => Some((
                RelationshipType::Derives,
                vec![ItemType::HardwareRequirement, ItemType::SoftwareRequirement],
            )),
            ItemType::HardwareRequirement => Some((
                RelationshipType::IsSatisfiedBy,
                vec![ItemType::HardwareDetailedDesign],
            )),
            ItemType::SoftwareRequirement => Some((
                RelationshipType::IsSatisfiedBy,
                vec![ItemType::SoftwareDetailedDesign],
            )),
            ItemType::HardwareDetailedDesign | ItemType::SoftwareDetailedDesign => Some((
                RelationshipType::IsJustifiedBy,
                vec![ItemType::ArchitectureDecisionRecord],
            )),
            ItemType::ArchitectureDecisionRecord => None,
        }
    }

    /// Returns the valid peer dependency types for a given item type.
    #[must_use]
    pub const fn valid_peer_for(item_type: ItemType) -> Option<ItemType> {
        match item_type {
            ItemType::SystemRequirement => Some(ItemType::SystemRequirement),
            ItemType::HardwareRequirement => Some(ItemType::HardwareRequirement),
            ItemType::SoftwareRequirement => Some(ItemType::SoftwareRequirement),
            ItemType::ArchitectureDecisionRecord => Some(ItemType::ArchitectureDecisionRecord),
            _ => None,
        }
    }

    /// Returns the valid justification targets for ADRs.
    #[must_use]
    pub fn valid_justification_targets() -> Vec<ItemType> {
        vec![
            ItemType::SystemArchitecture,
            ItemType::SoftwareDetailedDesign,
            ItemType::HardwareDetailedDesign,
        ]
    }

    /// Checks if a justification relationship is valid (ADR -> design artifact).
    #[must_use]
    pub fn is_valid_justification(from_type: ItemType, to_type: ItemType) -> bool {
        from_type == ItemType::ArchitectureDecisionRecord
            && Self::valid_justification_targets().contains(&to_type)
    }

    /// Checks if a supersession relationship is valid (ADR -> ADR).
    #[must_use]
    pub const fn is_valid_supersession(from_type: ItemType, to_type: ItemType) -> bool {
        matches!(from_type, ItemType::ArchitectureDecisionRecord)
            && matches!(to_type, ItemType::ArchitectureDecisionRecord)
    }

    /// Checks if a relationship is valid between two item types.
    #[must_use]
    pub fn is_valid_relationship(
        from_type: ItemType,
        to_type: ItemType,
        rel_type: RelationshipType,
    ) -> bool {
        match rel_type {
            // Upstream relationships
            RelationshipType::Refines
            | RelationshipType::DerivesFrom
            | RelationshipType::Satisfies
            | RelationshipType::Justifies => {
                if let Some((expected_rel, valid_targets)) = Self::valid_upstream_for(from_type) {
                    expected_rel == rel_type && valid_targets.contains(&to_type)
                } else {
                    false
                }
            }
            // Downstream relationships
            RelationshipType::IsRefinedBy
            | RelationshipType::Derives
            | RelationshipType::IsSatisfiedBy => {
                if let Some((expected_rel, valid_targets)) = Self::valid_downstream_for(from_type) {
                    expected_rel == rel_type && valid_targets.contains(&to_type)
                } else {
                    false
                }
            }
            // IsJustifiedBy needs special handling since design artifacts have multiple downstream types
            RelationshipType::IsJustifiedBy => Self::is_valid_justification(to_type, from_type),
            // Peer relationships (including ADR supersession)
            RelationshipType::DependsOn
            | RelationshipType::IsRequiredBy
            | RelationshipType::Supersedes
            | RelationshipType::IsSupersededBy => Self::valid_peer_for(from_type) == Some(to_type),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relationship_type_inverse() {
        assert_eq!(
            RelationshipType::Refines.inverse(),
            RelationshipType::IsRefinedBy
        );
        assert_eq!(
            RelationshipType::Derives.inverse(),
            RelationshipType::DerivesFrom
        );
        assert_eq!(
            RelationshipType::Satisfies.inverse(),
            RelationshipType::IsSatisfiedBy
        );
        assert_eq!(
            RelationshipType::DependsOn.inverse(),
            RelationshipType::IsRequiredBy
        );
    }

    #[test]
    fn test_relationship_type_direction() {
        assert!(RelationshipType::Refines.is_upstream());
        assert!(RelationshipType::DerivesFrom.is_upstream());
        assert!(RelationshipType::Satisfies.is_upstream());

        assert!(RelationshipType::IsRefinedBy.is_downstream());
        assert!(RelationshipType::Derives.is_downstream());
        assert!(RelationshipType::IsSatisfiedBy.is_downstream());

        assert!(RelationshipType::DependsOn.is_peer());
        assert!(RelationshipType::IsRequiredBy.is_peer());
    }

    #[test]
    fn test_valid_relationships() {
        // UseCase refines Solution
        assert!(RelationshipRules::is_valid_relationship(
            ItemType::UseCase,
            ItemType::Solution,
            RelationshipType::Refines
        ));

        // Scenario refines UseCase
        assert!(RelationshipRules::is_valid_relationship(
            ItemType::Scenario,
            ItemType::UseCase,
            RelationshipType::Refines
        ));

        // SystemRequirement derives_from Scenario
        assert!(RelationshipRules::is_valid_relationship(
            ItemType::SystemRequirement,
            ItemType::Scenario,
            RelationshipType::DerivesFrom
        ));

        // Invalid: Solution refines nothing
        assert!(!RelationshipRules::is_valid_relationship(
            ItemType::Solution,
            ItemType::UseCase,
            RelationshipType::Refines
        ));
    }

    #[test]
    fn test_peer_dependencies() {
        assert!(RelationshipRules::is_valid_relationship(
            ItemType::SystemRequirement,
            ItemType::SystemRequirement,
            RelationshipType::DependsOn
        ));

        assert!(!RelationshipRules::is_valid_relationship(
            ItemType::Solution,
            ItemType::Solution,
            RelationshipType::DependsOn
        ));
    }

    #[test]
    fn test_adr_justifies_relationship() {
        // ADR can justify design artifacts
        assert!(RelationshipRules::is_valid_relationship(
            ItemType::ArchitectureDecisionRecord,
            ItemType::SystemArchitecture,
            RelationshipType::Justifies
        ));
        assert!(RelationshipRules::is_valid_relationship(
            ItemType::ArchitectureDecisionRecord,
            ItemType::SoftwareDetailedDesign,
            RelationshipType::Justifies
        ));
        assert!(RelationshipRules::is_valid_relationship(
            ItemType::ArchitectureDecisionRecord,
            ItemType::HardwareDetailedDesign,
            RelationshipType::Justifies
        ));

        // ADR cannot justify non-design artifacts
        assert!(!RelationshipRules::is_valid_relationship(
            ItemType::ArchitectureDecisionRecord,
            ItemType::SystemRequirement,
            RelationshipType::Justifies
        ));
    }

    #[test]
    fn test_adr_supersession_relationship() {
        // ADR can supersede other ADRs (peer relationship)
        assert!(RelationshipRules::is_valid_relationship(
            ItemType::ArchitectureDecisionRecord,
            ItemType::ArchitectureDecisionRecord,
            RelationshipType::Supersedes
        ));
        assert!(RelationshipRules::is_valid_relationship(
            ItemType::ArchitectureDecisionRecord,
            ItemType::ArchitectureDecisionRecord,
            RelationshipType::IsSupersededBy
        ));

        // ADR cannot supersede non-ADR items
        assert!(!RelationshipRules::is_valid_relationship(
            ItemType::ArchitectureDecisionRecord,
            ItemType::SystemArchitecture,
            RelationshipType::Supersedes
        ));
    }

    #[test]
    fn test_adr_relationship_direction() {
        // Justifies is upstream
        assert!(RelationshipType::Justifies.is_upstream());
        // IsJustifiedBy is downstream
        assert!(RelationshipType::IsJustifiedBy.is_downstream());
        // Supersedes/IsSupersededBy are peer
        assert!(RelationshipType::Supersedes.is_peer());
        assert!(RelationshipType::IsSupersededBy.is_peer());
    }
}

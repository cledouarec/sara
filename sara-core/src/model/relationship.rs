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
}

impl RelationshipType {
    /// Get the inverse relationship type.
    pub fn inverse(&self) -> Self {
        match self {
            Self::Refines => Self::IsRefinedBy,
            Self::IsRefinedBy => Self::Refines,
            Self::Derives => Self::DerivesFrom,
            Self::DerivesFrom => Self::Derives,
            Self::Satisfies => Self::IsSatisfiedBy,
            Self::IsSatisfiedBy => Self::Satisfies,
            Self::DependsOn => Self::IsRequiredBy,
            Self::IsRequiredBy => Self::DependsOn,
        }
    }

    /// Check if this is an upstream relationship (toward Solution).
    pub fn is_upstream(&self) -> bool {
        matches!(self, Self::Refines | Self::DerivesFrom | Self::Satisfies)
    }

    /// Check if this is a downstream relationship (toward Detailed Designs).
    pub fn is_downstream(&self) -> bool {
        matches!(
            self,
            Self::IsRefinedBy | Self::Derives | Self::IsSatisfiedBy
        )
    }

    /// Check if this is a peer relationship (between items of the same type).
    pub fn is_peer(&self) -> bool {
        matches!(self, Self::DependsOn | Self::IsRequiredBy)
    }

    /// Returns the corresponding FieldName for this relationship type.
    pub fn field_name(&self) -> FieldName {
        match self {
            Self::Refines => FieldName::Refines,
            Self::IsRefinedBy => FieldName::IsRefinedBy,
            Self::Derives => FieldName::Derives,
            Self::DerivesFrom => FieldName::DerivesFrom,
            Self::Satisfies => FieldName::Satisfies,
            Self::IsSatisfiedBy => FieldName::IsSatisfiedBy,
            Self::DependsOn => FieldName::DependsOn,
            Self::IsRequiredBy => FieldName::IsRequiredBy,
        }
    }
}

impl std::fmt::Display for RelationshipType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.field_name().as_str())
    }
}

/// Represents a link between two items in the graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    /// Source item ID.
    pub from: ItemId,
    /// Target item ID.
    pub to: ItemId,
    /// Type of relationship.
    pub relationship_type: RelationshipType,
}

impl Relationship {
    /// Creates a new relationship.
    pub fn new(from: ItemId, to: ItemId, relationship_type: RelationshipType) -> Self {
        Self {
            from,
            to,
            relationship_type,
        }
    }

    /// Returns the inverse relationship.
    pub fn inverse(&self) -> Self {
        Self {
            from: self.to.clone(),
            to: self.from.clone(),
            relationship_type: self.relationship_type.inverse(),
        }
    }
}

/// Valid relationship rules based on item types.
pub struct RelationshipRules;

impl RelationshipRules {
    /// Returns the valid upstream relationship types for a given item type.
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
        }
    }

    /// Returns the valid downstream relationship types for a given item type.
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
            ItemType::HardwareDetailedDesign => None,
            ItemType::SoftwareDetailedDesign => None,
        }
    }

    /// Returns the valid peer dependency types for a given item type.
    pub fn valid_peer_for(item_type: ItemType) -> Option<ItemType> {
        match item_type {
            ItemType::SystemRequirement => Some(ItemType::SystemRequirement),
            ItemType::HardwareRequirement => Some(ItemType::HardwareRequirement),
            ItemType::SoftwareRequirement => Some(ItemType::SoftwareRequirement),
            _ => None,
        }
    }

    /// Checks if a relationship is valid between two item types.
    pub fn is_valid_relationship(
        from_type: ItemType,
        to_type: ItemType,
        rel_type: RelationshipType,
    ) -> bool {
        match rel_type {
            RelationshipType::Refines
            | RelationshipType::DerivesFrom
            | RelationshipType::Satisfies => {
                if let Some((expected_rel, valid_targets)) = Self::valid_upstream_for(from_type) {
                    expected_rel == rel_type && valid_targets.contains(&to_type)
                } else {
                    false
                }
            }
            RelationshipType::IsRefinedBy
            | RelationshipType::Derives
            | RelationshipType::IsSatisfiedBy => {
                if let Some((expected_rel, valid_targets)) = Self::valid_downstream_for(from_type) {
                    expected_rel == rel_type && valid_targets.contains(&to_type)
                } else {
                    false
                }
            }
            RelationshipType::DependsOn => Self::valid_peer_for(from_type) == Some(to_type),
            RelationshipType::IsRequiredBy => Self::valid_peer_for(from_type) == Some(to_type),
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
}

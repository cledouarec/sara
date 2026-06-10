//! Relationship types and structures for the knowledge graph.

use serde::{Deserialize, Serialize};

use super::field::FieldName;
use super::item::{ItemId, ItemType};
use crate::schema::{self, RelationDirection};

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
    /// Returns all relationship variants in declaration order.
    #[must_use]
    pub const fn all() -> &'static [RelationshipType] {
        &[
            Self::Refines,
            Self::IsRefinedBy,
            Self::Derives,
            Self::DerivesFrom,
            Self::Satisfies,
            Self::IsSatisfiedBy,
            Self::DependsOn,
            Self::IsRequiredBy,
            Self::Justifies,
            Self::IsJustifiedBy,
            Self::Supersedes,
            Self::IsSupersededBy,
        ]
    }

    /// Returns the variant matching the given schema relation id, if any.
    ///
    /// Inverse of [`RelationshipType::field_name`]: the schema relation id
    /// matches `RelationshipType::field_name().as_str()`.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        Self::all()
            .iter()
            .copied()
            .find(|r| r.field_name().as_str() == id)
    }

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
///
/// The source item is implied by the `Item` containing this relationship.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relationship {
    /// Target item ID.
    pub to: ItemId,
    /// Type of relationship.
    pub relationship_type: RelationshipType,
}

impl Relationship {
    /// Creates a new relationship to the given target.
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
///
/// All checks delegate to the active [`crate::schema::Schema`], so custom
/// schemas redefining the relationship matrix take effect transparently.
pub struct RelationshipRules;

impl RelationshipRules {
    /// Returns the upstream relationship a type may establish and its allowed
    /// targets.
    ///
    /// Built-in types have at most one upstream relation, but a custom schema
    /// may declare several. Only the first upstream entry is returned, in
    /// declaration order, to preserve the legacy single-relation API.
    #[must_use]
    pub fn valid_upstream_for(item_type: ItemType) -> Option<(RelationshipType, Vec<ItemType>)> {
        let def = schema::item_type_def(item_type.as_str())?;
        def.allowed_targets
            .iter()
            .filter_map(|t| {
                let rel = schema::relation_def(&t.relation)?;
                if rel.direction != RelationDirection::Upstream {
                    return None;
                }
                let rel_type = RelationshipType::from_id(&t.relation)?;
                let targets: Vec<ItemType> = t
                    .targets
                    .iter()
                    .filter_map(|id| ItemType::from_id(id))
                    .collect();
                Some((rel_type, targets))
            })
            .next()
    }

    /// Returns the downstream relationship a type may receive and the source
    /// types that may target it.
    ///
    /// Derived from every other type's upstream declarations: a type T is a
    /// downstream target of type S when S declares an upstream relation
    /// targeting T. The returned relation is the inverse of S's upstream
    /// relation, matching the legacy single-entry API.
    #[must_use]
    pub fn valid_downstream_for(item_type: ItemType) -> Option<(RelationshipType, Vec<ItemType>)> {
        let active = schema::active();
        let to_id = item_type.as_str();

        // Collect (inverse_rel, source_type) pairs grouped by inverse relation,
        // preserving the order types are declared in the schema.
        let mut entries: Vec<(RelationshipType, Vec<ItemType>)> = Vec::new();
        for src_def in &active.item_types {
            let Some(src_type) = ItemType::from_id(&src_def.id) else {
                continue;
            };
            for target in &src_def.allowed_targets {
                let Some(rel) = schema::relation_def(&target.relation) else {
                    continue;
                };
                if rel.direction != RelationDirection::Upstream {
                    continue;
                }
                if !target.targets.iter().any(|t| t == to_id) {
                    continue;
                }
                let Some(inverse) = RelationshipType::from_id(&rel.inverse) else {
                    continue;
                };
                if let Some(entry) = entries.iter_mut().find(|(r, _)| *r == inverse) {
                    if !entry.1.contains(&src_type) {
                        entry.1.push(src_type);
                    }
                } else {
                    entries.push((inverse, vec![src_type]));
                }
            }
        }
        entries.into_iter().next()
    }

    /// Returns the peer dependency target a type may declare, if any.
    ///
    /// Mirrors the legacy type-level peer rule: peer relations are valid
    /// between any two items of a peer-capable type, regardless of the
    /// specific peer relation.
    #[must_use]
    pub fn valid_peer_for(item_type: ItemType) -> Option<ItemType> {
        let def = schema::item_type_def(item_type.as_str())?;
        def.allowed_targets
            .iter()
            .filter_map(|t| {
                let rel = schema::relation_def(&t.relation)?;
                if rel.direction != RelationDirection::Peer {
                    return None;
                }
                t.targets.iter().find_map(|id| ItemType::from_id(id))
            })
            .next()
    }

    /// Returns the valid justification targets for ADRs.
    ///
    /// Derived from the ADR type's `justifies` upstream relation in the
    /// active schema.
    #[must_use]
    pub fn valid_justification_targets() -> Vec<ItemType> {
        let Some(def) = schema::item_type_def(ItemType::ARCHITECTURE_DECISION_RECORD.as_str())
        else {
            return Vec::new();
        };
        def.allowed_targets
            .iter()
            .filter(|t| t.relation == "justifies")
            .flat_map(|t| t.targets.iter().filter_map(|id| ItemType::from_id(id)))
            .collect()
    }

    /// Checks if a relationship is valid between two item types.
    ///
    /// Delegates to [`crate::schema::Schema::is_valid_relationship`] on the
    /// active schema; the per-relation/per-direction semantics live there.
    #[must_use]
    pub fn is_valid_relationship(
        from_type: ItemType,
        to_type: ItemType,
        rel_type: RelationshipType,
    ) -> bool {
        schema::active().is_valid_relationship(
            from_type.as_str(),
            to_type.as_str(),
            rel_type.field_name().as_str(),
        )
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
            ItemType::USE_CASE,
            ItemType::SOLUTION,
            RelationshipType::Refines
        ));

        // Scenario refines UseCase
        assert!(RelationshipRules::is_valid_relationship(
            ItemType::SCENARIO,
            ItemType::USE_CASE,
            RelationshipType::Refines
        ));

        // SystemRequirement derives_from Scenario
        assert!(RelationshipRules::is_valid_relationship(
            ItemType::SYSTEM_REQUIREMENT,
            ItemType::SCENARIO,
            RelationshipType::DerivesFrom
        ));

        // Invalid: Solution refines nothing
        assert!(!RelationshipRules::is_valid_relationship(
            ItemType::SOLUTION,
            ItemType::USE_CASE,
            RelationshipType::Refines
        ));
    }

    #[test]
    fn test_peer_dependencies() {
        assert!(RelationshipRules::is_valid_relationship(
            ItemType::SYSTEM_REQUIREMENT,
            ItemType::SYSTEM_REQUIREMENT,
            RelationshipType::DependsOn
        ));

        assert!(!RelationshipRules::is_valid_relationship(
            ItemType::SOLUTION,
            ItemType::SOLUTION,
            RelationshipType::DependsOn
        ));
    }

    #[test]
    fn test_adr_justifies_relationship() {
        // ADR can justify design artifacts
        assert!(RelationshipRules::is_valid_relationship(
            ItemType::ARCHITECTURE_DECISION_RECORD,
            ItemType::SYSTEM_ARCHITECTURE,
            RelationshipType::Justifies
        ));
        assert!(RelationshipRules::is_valid_relationship(
            ItemType::ARCHITECTURE_DECISION_RECORD,
            ItemType::SOFTWARE_DETAILED_DESIGN,
            RelationshipType::Justifies
        ));
        assert!(RelationshipRules::is_valid_relationship(
            ItemType::ARCHITECTURE_DECISION_RECORD,
            ItemType::HARDWARE_DETAILED_DESIGN,
            RelationshipType::Justifies
        ));

        // ADR cannot justify non-design artifacts
        assert!(!RelationshipRules::is_valid_relationship(
            ItemType::ARCHITECTURE_DECISION_RECORD,
            ItemType::SYSTEM_REQUIREMENT,
            RelationshipType::Justifies
        ));
    }

    #[test]
    fn test_adr_supersession_relationship() {
        // ADR can supersede other ADRs (peer relationship)
        assert!(RelationshipRules::is_valid_relationship(
            ItemType::ARCHITECTURE_DECISION_RECORD,
            ItemType::ARCHITECTURE_DECISION_RECORD,
            RelationshipType::Supersedes
        ));
        assert!(RelationshipRules::is_valid_relationship(
            ItemType::ARCHITECTURE_DECISION_RECORD,
            ItemType::ARCHITECTURE_DECISION_RECORD,
            RelationshipType::IsSupersededBy
        ));

        // ADR cannot supersede non-ADR items
        assert!(!RelationshipRules::is_valid_relationship(
            ItemType::ARCHITECTURE_DECISION_RECORD,
            ItemType::SYSTEM_ARCHITECTURE,
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

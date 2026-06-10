//! Relationship types and structures for the knowledge graph.

use serde::{Deserialize, Serialize};

use super::item::{ItemId, ItemType};
use crate::schema::{self, RelationDef, RelationDirection};

/// Identifies a relation by its schema id.
///
/// Wraps the interned snake_case id of a relation declared by the active
/// schema (or the built-in default). Inverse, direction and primality are
/// resolved against the schema, so relations introduced by a custom YAML
/// schema behave exactly like the built-in ones. Constants are provided for
/// the built-in relations; other relations are obtained through
/// [`RelationshipType::from_id`] or [`RelationshipType::all`].
///
/// Equality and hashing compare the id by content, so a constant compares
/// equal to the same relation resolved from a schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RelationshipType(&'static str);

impl RelationshipType {
    /// Refinement: child refines parent (Scenario refines Use Case).
    pub const REFINES: Self = Self("refines");
    /// Inverse of refines: parent is refined by child.
    pub const IS_REFINED_BY: Self = Self("is_refined_by");
    /// Derivation: parent derives child (Scenario derives System Requirement).
    pub const DERIVES: Self = Self("derives");
    /// Inverse of derives: child derives from parent.
    pub const DERIVES_FROM: Self = Self("derives_from");
    /// Satisfaction: child satisfies parent.
    pub const SATISFIES: Self = Self("satisfies");
    /// Inverse of satisfies: parent is satisfied by child.
    pub const IS_SATISFIED_BY: Self = Self("is_satisfied_by");
    /// Dependency: an item depends on a peer of the same type.
    pub const DEPENDS_ON: Self = Self("depends_on");
    /// Inverse of depends_on: an item is required by a peer.
    pub const IS_REQUIRED_BY: Self = Self("is_required_by");
    /// Justification: an ADR justifies a design artifact.
    pub const JUSTIFIES: Self = Self("justifies");
    /// Inverse of justifies: a design artifact is justified by an ADR.
    pub const IS_JUSTIFIED_BY: Self = Self("justified_by");
    /// Supersession: a newer ADR supersedes an older ADR.
    pub const SUPERSEDES: Self = Self("supersedes");
    /// Inverse of supersedes: an older ADR is superseded by a newer one.
    pub const IS_SUPERSEDED_BY: Self = Self("superseded_by");

    /// Returns all relations known to the active schema, in catalog order.
    ///
    /// Includes built-in relations that a partial custom schema does not
    /// redefine.
    #[must_use]
    pub fn all() -> Vec<RelationshipType> {
        schema::all_relation_defs()
            .iter()
            .map(|def| Self(def.id.as_str()))
            .collect()
    }

    /// Returns the relation with the given schema id, if the active schema
    /// (or the built-in default) defines it.
    ///
    /// Inverse of [`RelationshipType::as_str`]. The returned value carries
    /// the id interned in the schema, so it lives for the whole process.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        schema::relation_def(id).map(|def| Self(def.id.as_str()))
    }

    /// Returns the schema id (snake_case string) for this relation.
    ///
    /// This is also the frontmatter field name carrying the relation.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        self.0
    }

    /// Returns the relation definition in the active schema, if any.
    fn def(&self) -> Option<&'static RelationDef> {
        schema::relation_def(self.0)
    }

    /// Get the inverse relationship type.
    ///
    /// Resolved from the schema's relation catalog; relations missing from
    /// the schema (which validation prevents) are their own inverse.
    #[must_use]
    pub fn inverse(&self) -> Self {
        self.def()
            .and_then(|def| Self::from_id(&def.inverse))
            .unwrap_or(*self)
    }

    /// Check if this is an upstream relationship (toward the hierarchy root).
    #[must_use]
    pub fn is_upstream(&self) -> bool {
        self.def()
            .is_some_and(|def| def.direction == RelationDirection::Upstream)
    }

    /// Check if this is a downstream relationship (away from the root).
    #[must_use]
    pub fn is_downstream(&self) -> bool {
        self.def()
            .is_some_and(|def| def.direction == RelationDirection::Downstream)
    }

    /// Check if this is a peer relationship (between items of the same type).
    #[must_use]
    pub fn is_peer(&self) -> bool {
        self.def()
            .is_some_and(|def| def.direction == RelationDirection::Peer)
    }

    /// Check if this is a primary relationship (not an inverse).
    ///
    /// Primary relations are the declared side of an inverse pair; inverse
    /// relations exist for graph traversal and are not considered when
    /// checking for cycles.
    #[must_use]
    pub fn is_primary(&self) -> bool {
        self.def().is_some_and(|def| def.primary)
    }
}

impl std::fmt::Display for RelationshipType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for RelationshipType {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.0)
    }
}

impl<'de> Deserialize<'de> for RelationshipType {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let id = String::deserialize(deserializer)?;
        Self::from_id(&id)
            .ok_or_else(|| serde::de::Error::custom(format!("unknown relation `{id}`")))
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
            rel_type.as_str(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relationship_type_inverse() {
        assert_eq!(
            RelationshipType::REFINES.inverse(),
            RelationshipType::IS_REFINED_BY
        );
        assert_eq!(
            RelationshipType::DERIVES.inverse(),
            RelationshipType::DERIVES_FROM
        );
        assert_eq!(
            RelationshipType::SATISFIES.inverse(),
            RelationshipType::IS_SATISFIED_BY
        );
        assert_eq!(
            RelationshipType::DEPENDS_ON.inverse(),
            RelationshipType::IS_REQUIRED_BY
        );
    }

    #[test]
    fn test_relationship_type_direction() {
        assert!(RelationshipType::REFINES.is_upstream());
        assert!(RelationshipType::DERIVES_FROM.is_upstream());
        assert!(RelationshipType::SATISFIES.is_upstream());

        assert!(RelationshipType::IS_REFINED_BY.is_downstream());
        assert!(RelationshipType::DERIVES.is_downstream());
        assert!(RelationshipType::IS_SATISFIED_BY.is_downstream());

        assert!(RelationshipType::DEPENDS_ON.is_peer());
        assert!(RelationshipType::IS_REQUIRED_BY.is_peer());
    }

    #[test]
    fn test_valid_relationships() {
        // UseCase refines Solution
        assert!(RelationshipRules::is_valid_relationship(
            ItemType::USE_CASE,
            ItemType::SOLUTION,
            RelationshipType::REFINES
        ));

        // Scenario refines UseCase
        assert!(RelationshipRules::is_valid_relationship(
            ItemType::SCENARIO,
            ItemType::USE_CASE,
            RelationshipType::REFINES
        ));

        // SystemRequirement derives_from Scenario
        assert!(RelationshipRules::is_valid_relationship(
            ItemType::SYSTEM_REQUIREMENT,
            ItemType::SCENARIO,
            RelationshipType::DERIVES_FROM
        ));

        // Invalid: Solution refines nothing
        assert!(!RelationshipRules::is_valid_relationship(
            ItemType::SOLUTION,
            ItemType::USE_CASE,
            RelationshipType::REFINES
        ));
    }

    #[test]
    fn test_peer_dependencies() {
        assert!(RelationshipRules::is_valid_relationship(
            ItemType::SYSTEM_REQUIREMENT,
            ItemType::SYSTEM_REQUIREMENT,
            RelationshipType::DEPENDS_ON
        ));

        assert!(!RelationshipRules::is_valid_relationship(
            ItemType::SOLUTION,
            ItemType::SOLUTION,
            RelationshipType::DEPENDS_ON
        ));
    }

    #[test]
    fn test_adr_justifies_relationship() {
        // ADR can justify design artifacts
        assert!(RelationshipRules::is_valid_relationship(
            ItemType::ARCHITECTURE_DECISION_RECORD,
            ItemType::SYSTEM_ARCHITECTURE,
            RelationshipType::JUSTIFIES
        ));
        assert!(RelationshipRules::is_valid_relationship(
            ItemType::ARCHITECTURE_DECISION_RECORD,
            ItemType::SOFTWARE_DETAILED_DESIGN,
            RelationshipType::JUSTIFIES
        ));
        assert!(RelationshipRules::is_valid_relationship(
            ItemType::ARCHITECTURE_DECISION_RECORD,
            ItemType::HARDWARE_DETAILED_DESIGN,
            RelationshipType::JUSTIFIES
        ));

        // ADR cannot justify non-design artifacts
        assert!(!RelationshipRules::is_valid_relationship(
            ItemType::ARCHITECTURE_DECISION_RECORD,
            ItemType::SYSTEM_REQUIREMENT,
            RelationshipType::JUSTIFIES
        ));
    }

    #[test]
    fn test_adr_supersession_relationship() {
        // ADR can supersede other ADRs (peer relationship)
        assert!(RelationshipRules::is_valid_relationship(
            ItemType::ARCHITECTURE_DECISION_RECORD,
            ItemType::ARCHITECTURE_DECISION_RECORD,
            RelationshipType::SUPERSEDES
        ));
        assert!(RelationshipRules::is_valid_relationship(
            ItemType::ARCHITECTURE_DECISION_RECORD,
            ItemType::ARCHITECTURE_DECISION_RECORD,
            RelationshipType::IS_SUPERSEDED_BY
        ));

        // ADR cannot supersede non-ADR items
        assert!(!RelationshipRules::is_valid_relationship(
            ItemType::ARCHITECTURE_DECISION_RECORD,
            ItemType::SYSTEM_ARCHITECTURE,
            RelationshipType::SUPERSEDES
        ));
    }

    #[test]
    fn test_adr_relationship_direction() {
        // Justifies is upstream
        assert!(RelationshipType::JUSTIFIES.is_upstream());
        // IsJustifiedBy is downstream
        assert!(RelationshipType::IS_JUSTIFIED_BY.is_downstream());
        // Supersedes/IsSupersededBy are peer
        assert!(RelationshipType::SUPERSEDES.is_peer());
        assert!(RelationshipType::IS_SUPERSEDED_BY.is_peer());
    }
}

//! Relationship types and structures for the knowledge graph.

use serde::{Deserialize, Serialize};

use super::item::{ItemId, ItemType};
use crate::schema::{self, RelationDef, RelationDirection};

/// Identifies a relation by its schema id.
///
/// Wraps the interned snake_case id of a relation declared by the active
/// schema (or the built-in default). Inverse, direction and primality are
/// resolved against the schema, so relations introduced by a custom YAML
/// schema behave exactly like the built-in ones (handles for those live in
/// [`crate::schema::builtin`]). Relations are obtained through
/// [`RelationshipType::from_id`] or [`RelationshipType::all`].
///
/// Equality and hashing compare the id by content, so a handle built from a
/// static id compares equal to the same relation resolved from a schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RelationshipType(&'static str);

impl RelationshipType {
    /// Creates a handle from a static schema id, without resolving it.
    ///
    /// The id is not checked against the active schema: a handle whose id the
    /// schema does not declare resolves to no metadata. Reserved for the
    /// definition of well-known ids (see [`crate::schema::builtin`]); resolve
    /// runtime ids through [`RelationshipType::from_id`].
    pub(crate) const fn from_static(id: &'static str) -> Self {
        Self(id)
    }

    /// Returns all relations of the active schema, in catalog order.
    #[must_use]
    pub fn all() -> Vec<RelationshipType> {
        schema::active()
            .relations
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

    /// Returns the display name for this relation.
    ///
    /// Resolved from the active schema; falls back to the built-in default
    /// when a custom schema does not redefine the relation.
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        self.def().map_or("", |def| def.display_name.as_str())
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
}

/// Valid relationship rules based on item types.
///
/// All checks delegate to the active [`crate::schema::Schema`], so custom
/// schemas redefining the relationship matrix take effect transparently.
pub struct RelationshipRules;

impl RelationshipRules {
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

    use crate::schema::builtin;

    #[test]
    fn test_relationship_type_inverse() {
        assert_eq!(builtin::REFINES.inverse(), builtin::IS_REFINED_BY);
        assert_eq!(builtin::DERIVES.inverse(), builtin::DERIVES_FROM);
        assert_eq!(builtin::SATISFIES.inverse(), builtin::IS_SATISFIED_BY);
        assert_eq!(builtin::DEPENDS_ON.inverse(), builtin::IS_REQUIRED_BY);
    }

    #[test]
    fn test_relationship_type_direction() {
        assert!(builtin::REFINES.is_upstream());
        assert!(builtin::DERIVES_FROM.is_upstream());
        assert!(builtin::SATISFIES.is_upstream());

        assert!(builtin::IS_REFINED_BY.is_downstream());
        assert!(builtin::DERIVES.is_downstream());
        assert!(builtin::IS_SATISFIED_BY.is_downstream());

        assert!(builtin::DEPENDS_ON.is_peer());
        assert!(builtin::IS_REQUIRED_BY.is_peer());
    }

    #[test]
    fn test_valid_relationships() {
        // UseCase refines Solution
        assert!(RelationshipRules::is_valid_relationship(
            builtin::USE_CASE,
            builtin::SOLUTION,
            builtin::REFINES
        ));

        // Scenario refines UseCase
        assert!(RelationshipRules::is_valid_relationship(
            builtin::SCENARIO,
            builtin::USE_CASE,
            builtin::REFINES
        ));

        // SystemRequirement derives_from Scenario
        assert!(RelationshipRules::is_valid_relationship(
            builtin::SYSTEM_REQUIREMENT,
            builtin::SCENARIO,
            builtin::DERIVES_FROM
        ));

        // Invalid: Solution refines nothing
        assert!(!RelationshipRules::is_valid_relationship(
            builtin::SOLUTION,
            builtin::USE_CASE,
            builtin::REFINES
        ));
    }

    #[test]
    fn test_peer_dependencies() {
        assert!(RelationshipRules::is_valid_relationship(
            builtin::SYSTEM_REQUIREMENT,
            builtin::SYSTEM_REQUIREMENT,
            builtin::DEPENDS_ON
        ));

        assert!(!RelationshipRules::is_valid_relationship(
            builtin::SOLUTION,
            builtin::SOLUTION,
            builtin::DEPENDS_ON
        ));
    }

    #[test]
    fn test_adr_justifies_relationship() {
        // ADR can justify design artifacts
        assert!(RelationshipRules::is_valid_relationship(
            builtin::ARCHITECTURE_DECISION_RECORD,
            builtin::SYSTEM_ARCHITECTURE,
            builtin::JUSTIFIES
        ));
        assert!(RelationshipRules::is_valid_relationship(
            builtin::ARCHITECTURE_DECISION_RECORD,
            builtin::SOFTWARE_DETAILED_DESIGN,
            builtin::JUSTIFIES
        ));
        assert!(RelationshipRules::is_valid_relationship(
            builtin::ARCHITECTURE_DECISION_RECORD,
            builtin::HARDWARE_DETAILED_DESIGN,
            builtin::JUSTIFIES
        ));

        // ADR cannot justify non-design artifacts
        assert!(!RelationshipRules::is_valid_relationship(
            builtin::ARCHITECTURE_DECISION_RECORD,
            builtin::SYSTEM_REQUIREMENT,
            builtin::JUSTIFIES
        ));
    }

    #[test]
    fn test_adr_supersession_relationship() {
        // ADR can supersede other ADRs (peer relationship)
        assert!(RelationshipRules::is_valid_relationship(
            builtin::ARCHITECTURE_DECISION_RECORD,
            builtin::ARCHITECTURE_DECISION_RECORD,
            builtin::SUPERSEDES
        ));
        assert!(RelationshipRules::is_valid_relationship(
            builtin::ARCHITECTURE_DECISION_RECORD,
            builtin::ARCHITECTURE_DECISION_RECORD,
            builtin::IS_SUPERSEDED_BY
        ));

        // ADR cannot supersede non-ADR items
        assert!(!RelationshipRules::is_valid_relationship(
            builtin::ARCHITECTURE_DECISION_RECORD,
            builtin::SYSTEM_ARCHITECTURE,
            builtin::SUPERSEDES
        ));
    }

    #[test]
    fn test_adr_relationship_direction() {
        // Justifies is upstream
        assert!(builtin::JUSTIFIES.is_upstream());
        // IsJustifiedBy is downstream
        assert!(builtin::IS_JUSTIFIED_BY.is_downstream());
        // Supersedes/IsSupersededBy are peer
        assert!(builtin::SUPERSEDES.is_peer());
        assert!(builtin::IS_SUPERSEDED_BY.is_peer());
    }
}

//! Runtime-loadable definition of the domain model.
//!
//! A data-driven description of the knowledge-graph model — item types, their
//! typed fields, the relation catalog and the allowed links between types.
//! A default schema is provided (see [`Schema::builtin`]) that covers most
//! needs out of the box; projects with domain-specific needs supply their own
//! schema in YAML.
//!
//! Lives in the core with no dependency on `parser` or `generator`, preserving
//! the hexagonal boundary.

mod active;
mod builtin;
mod yaml;

#[cfg(test)]
mod parity_tests;

use serde::{Deserialize, Serialize};

pub use active::{active, install};
pub(crate) use active::{item_type_def, relation_def};

/// The direction of a relation relative to the model hierarchy.
///
/// Mirrors the classification currently encoded in
/// `RelationshipType::is_upstream` / `is_downstream` / `is_peer`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationDirection {
    /// Toward the root of the hierarchy (e.g. `refines`, `satisfies`).
    Upstream,
    /// Away from the root (inverse of an upstream relation).
    Downstream,
    /// Between items of the same type (e.g. `depends_on`, `supersedes`).
    Peer,
}

/// The declared type of a type-specific field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    /// Free-form text.
    Text,
    /// One value among a closed set (e.g. ADR status).
    Enum {
        /// Allowed values, in declaration order. Must be non-empty.
        values: Vec<String>,
    },
    /// A single reference to another item's id.
    ItemRef,
    /// An ordered list of values of the inner type.
    List(Box<FieldType>),
    /// An ISO-8601 date.
    Date,
}

/// Declaration of a single type-specific field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldDef {
    /// Canonical snake_case name as it appears in YAML frontmatter.
    pub name: String,
    /// Human-readable label for user-facing output.
    pub display_name: String,
    /// Declared value type.
    pub field_type: FieldType,
    /// Whether the field must be present when building the item.
    #[serde(default)]
    pub required: bool,
}

/// A relation that types can declare a target as (e.g. `refines`).
///
/// Relations come in inverse pairs. The `primary` relation is the one a type
/// declares in [`ItemTypeDef::allowed_targets`]; the validity of the inverse
/// is *derived* from it (see [`Schema::is_valid_relationship`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationDef {
    /// Canonical snake_case id (matches the legacy frontmatter field name).
    pub id: String,
    /// Human-readable label.
    pub display_name: String,
    /// Id of the inverse relation. `relation(inverse).inverse == id` must hold.
    pub inverse: String,
    /// Direction relative to the hierarchy.
    pub direction: RelationDirection,
    /// Whether this is the declared (primary) side of the inverse pair.
    ///
    /// Upstream relations and the primary peer relations (`depends_on`,
    /// `supersedes`) are primary; their inverses are derived.
    pub primary: bool,
}

/// A relation a type may establish toward a set of target types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllowedTarget {
    /// Id of the relation (must reference a [`RelationDef::id`]).
    pub relation: String,
    /// Ids of the item types this relation may point to, in declared order.
    pub targets: Vec<String>,
}

/// Definition of one item type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemTypeDef {
    /// Canonical snake_case id (matches the legacy `type:` value).
    pub id: String,
    /// Human-readable label.
    pub display_name: String,
    /// Id prefix used when generating identifiers (e.g. `SOL`).
    pub prefix: String,
    /// Identifier format template (e.g. `"{prefix}-{seq:03}"`).
    pub id_format: String,
    /// Required parent type ids. Empty means the type is a hierarchy root.
    #[serde(default)]
    pub parent_types: Vec<String>,
    /// Type-specific fields, in declaration order.
    #[serde(default)]
    pub fields: Vec<FieldDef>,
    /// Primary relations this type may establish, with their valid targets.
    #[serde(default)]
    pub allowed_targets: Vec<AllowedTarget>,
}

/// A complete, runtime-loadable description of the domain model.
///
/// Order is significant: `item_types` follows the hierarchy order used by
/// `ItemType::all`, and `allowed_targets`/`fields` preserve declaration order
/// so that derived structures match the legacy behavior exactly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Schema {
    /// Item type definitions, in hierarchy order.
    pub item_types: Vec<ItemTypeDef>,
    /// Relation catalog (both primary relations and their inverses).
    pub relations: Vec<RelationDef>,
}

impl Schema {
    /// Returns the item type definition with the given id, if any.
    #[must_use]
    pub fn item_type(&self, id: &str) -> Option<&ItemTypeDef> {
        self.item_types.iter().find(|t| t.id == id)
    }

    /// Returns the relation definition with the given id, if any.
    #[must_use]
    pub fn relation(&self, id: &str) -> Option<&RelationDef> {
        self.relations.iter().find(|r| r.id == id)
    }

    /// Checks whether a relation is valid from one item type to another.
    ///
    /// The full validity matrix is *derived* from what each type declares in
    /// [`ItemTypeDef::allowed_targets`], following the legacy semantics:
    ///
    /// - **Upstream**: valid when the source type declares this relation with
    ///   the destination among its targets.
    /// - **Downstream**: the inverse of an upstream relation; valid when the
    ///   corresponding upstream relation is valid in the opposite direction.
    /// - **Peer**: type-level, not relation-specific (mirrors the legacy
    ///   `valid_peer_for`): any peer relation is valid when the source type
    ///   declares *some* peer target containing the destination.
    ///
    /// Unknown type or relation ids yield `false`.
    #[must_use]
    pub fn is_valid_relationship(&self, from: &str, to: &str, relation: &str) -> bool {
        let Some(rel) = self.relation(relation) else {
            return false;
        };

        match rel.direction {
            RelationDirection::Upstream => self.item_type(from).is_some_and(|def| {
                def.allowed_targets
                    .iter()
                    .any(|t| t.relation == relation && t.targets.iter().any(|target| target == to))
            }),
            RelationDirection::Downstream => {
                // The inverse of a downstream relation is an upstream relation,
                // evaluated in the opposite direction.
                self.is_valid_relationship(to, from, &rel.inverse)
            }
            RelationDirection::Peer => self.item_type(from).is_some_and(|def| {
                def.allowed_targets.iter().any(|t| {
                    self.relation(&t.relation)
                        .is_some_and(|r| r.direction == RelationDirection::Peer)
                        && t.targets.iter().any(|target| target == to)
                })
            }),
        }
    }

    /// Serializes the schema to YAML.
    ///
    /// # Errors
    ///
    /// Returns [`SaraError::InvalidConfig`] if serialization fails (which
    /// should not happen for a well-formed in-memory schema).
    ///
    /// [`SaraError::InvalidConfig`]: crate::error::SaraError::InvalidConfig
    pub fn to_yaml(&self) -> Result<String, crate::error::SaraError> {
        serde_yaml::to_string(self).map_err(|e| crate::error::SaraError::InvalidConfig {
            path: std::path::PathBuf::from("<schema>"),
            reason: e.to_string(),
        })
    }
}

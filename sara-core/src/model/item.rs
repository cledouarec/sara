//! Item types and structures for the knowledge graph.

use std::fmt;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize, Serializer};

use crate::error::SaraError;
use crate::model::field::FieldValue;
use crate::model::metadata::{SourceLocation, serialize_path};
use crate::model::relationship::{Relationship, RelationshipType};
use crate::schema::{self, FieldDef, IdFormat, ItemTypeDef, RelationDirection};

/// Canonical name of the item identifier field.
///
/// Single definition of the core field names every item carries (the
/// schema-declared fields are named by the schema itself). Consumed by the
/// parser, the generator and the CLI flags; the `.tera` templates reference
/// the same names as context keys.
pub const FIELD_ID: &str = "id";
/// Canonical name of the item type field.
pub const FIELD_TYPE: &str = "type";
/// Canonical name of the item name field.
pub const FIELD_NAME: &str = "name";
/// Canonical name of the item description field.
pub const FIELD_DESCRIPTION: &str = "description";

/// Identifies an item type by its schema id.
///
/// Wraps the interned snake_case id of a type defined by the active schema
/// (or the built-in default). All metadata — prefix, display name, declared
/// fields, parent and relation links — is resolved against the schema, so
/// types introduced by a custom YAML schema behave exactly like the built-in
/// ones (handles for those live in [`crate::schema::builtin`]). Types are
/// obtained through [`ItemType::from_id`] or [`ItemType::all`].
///
/// Equality and hashing compare the id by content, so a handle built from a
/// static id compares equal to the same type resolved from a schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ItemType(&'static str);

impl ItemType {
    /// Creates a handle from a static schema id, without resolving it.
    ///
    /// The id is not checked against the active schema: a handle whose id the
    /// schema does not define resolves to no metadata. Reserved for the
    /// definition of well-known ids (see [`crate::schema::builtin`]); resolve
    /// runtime ids through [`ItemType::from_id`].
    pub(crate) const fn from_static(id: &'static str) -> Self {
        Self(id)
    }

    /// Returns all item types of the active schema, in hierarchy order.
    #[must_use]
    pub fn all() -> Vec<ItemType> {
        schema::active()
            .item_types
            .iter()
            .map(|def| Self(def.id.as_str()))
            .collect()
    }

    /// Returns the item type definition in the active schema, if any.
    fn def(&self) -> Option<&'static ItemTypeDef> {
        schema::item_type_def(self.0)
    }

    /// Returns the declaration of the given field, if the type declares it.
    #[must_use]
    pub fn declared_field(&self, name: &str) -> Option<&'static FieldDef> {
        self.def()
            .and_then(|def| def.fields.iter().find(|f| f.name == name))
    }

    /// Returns the fields the active schema declares for this type.
    #[must_use]
    pub fn declared_fields(&self) -> &'static [FieldDef] {
        self.def().map_or(&[], |def| def.fields.as_slice())
    }

    /// Returns the relations this type declares toward targets, in
    /// declaration order.
    #[must_use]
    pub fn declared_relations(&self) -> Vec<RelationshipType> {
        self.def().map_or_else(Vec::new, |def| {
            def.allowed_targets
                .iter()
                .filter_map(|t| RelationshipType::from_id(&t.relation))
                .collect()
        })
    }

    /// Returns the display name for this item type.
    ///
    /// Resolved from the active schema; falls back to the built-in default
    /// when a custom schema does not redefine the type.
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        schema::item_type_def(self.as_str())
            .map(|d| d.display_name.as_str())
            .unwrap_or_default()
    }

    /// Returns the common ID prefix for this item type.
    ///
    /// Resolved from the active schema; falls back to the built-in default
    /// when a custom schema does not redefine the type.
    #[must_use]
    pub fn prefix(&self) -> &'static str {
        schema::item_type_def(self.as_str())
            .map(|d| d.prefix.as_str())
            .unwrap_or_default()
    }

    /// Returns the compiled identifier template of this type, if the active
    /// schema defines the type.
    fn id_format(&self) -> Option<IdFormat> {
        schema::item_type_def(self.as_str()).and_then(|def| IdFormat::parse(&def.id_format).ok())
    }

    /// Generates a new ID for the given type with an optional sequence number.
    ///
    /// Renders the type's `id_format`; defaults to sequence 1 if not
    /// provided. Falls back to `PREFIX-NNN` when the active schema does not
    /// define the type.
    #[must_use]
    pub fn generate_id(&self, sequence: Option<u32>) -> String {
        let seq = sequence.unwrap_or(1);
        match self.id_format() {
            Some(format) => format.render(self.prefix(), self.as_str(), seq),
            None => format!("{}-{seq:03}", self.prefix()),
        }
    }

    /// Suggests the next ID based on existing items in the graph.
    ///
    /// Scans the items of this type whose id matches the type's `id_format`
    /// with non-sequence placeholders bound to their current values (so a
    /// format embedding `{year}` restarts its counter each year), and
    /// renders the highest matched sequence plus one. Formats without a
    /// `{seq}` placeholder render directly (e.g. a fresh UUID). If no graph
    /// is provided or no id matches, returns the first ID (e.g. "SOL-001").
    #[must_use]
    pub fn suggest_next_id(&self, graph: Option<&crate::graph::KnowledgeGraph>) -> String {
        let Some(format) = self.id_format() else {
            return self.generate_id(None);
        };
        let prefix = self.prefix();
        if !format.has_seq() {
            return format.render(prefix, self.as_str(), 1);
        }
        let max_seq = graph
            .into_iter()
            .flat_map(|g| g.items())
            .filter(|item| item.item_type == *self)
            .filter_map(|item| format.extract_seq(item.id.as_str(), prefix, self.as_str()))
            .max()
            .unwrap_or(0);
        format.render(prefix, self.as_str(), max_seq.saturating_add(1))
    }

    /// Returns true if this is a root item type.
    ///
    /// A root has no required parent and declares no upstream relation, so it
    /// anchors the hierarchy (detached types like ADRs are not roots: they
    /// have no parent but justify other items).
    #[must_use]
    pub fn is_root(&self) -> bool {
        self.def().is_some_and(|def| {
            def.parent_types.is_empty()
                && !def.allowed_targets.iter().any(|t| {
                    schema::relation_def(&t.relation)
                        .is_some_and(|r| r.direction == RelationDirection::Upstream)
                })
        })
    }

    /// Returns the required parent item type for this type, if any.
    ///
    /// Solution has no parent (root of the hierarchy). Resolved from the
    /// first entry of the active schema's `parent_types` list.
    #[must_use]
    pub fn required_parent_type(&self) -> Option<ItemType> {
        schema::item_type_def(self.as_str())
            .and_then(|d| d.parent_types.first())
            .and_then(|id| ItemType::from_id(id))
    }

    /// Returns the schema id (snake_case string) for this item type.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        self.0
    }

    /// Returns the item type with the given schema id, if the active schema
    /// (or the built-in default) defines it.
    ///
    /// Inverse of [`ItemType::as_str`]. The returned value carries the id
    /// interned in the schema, so it lives for the whole process.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        schema::item_type_def(id).map(|def| Self(def.id.as_str()))
    }

    /// Returns all traceability configurations for this item type.
    ///
    /// One entry per `(relation, target type)` pair the type declares for an
    /// upstream relation or a primary peer relation: the links a user states
    /// when creating or editing an item. Inverse relations are derived, so
    /// they carry no configuration. Roots and detached types may have none.
    /// Targets that do not map to a type of the active schema are skipped.
    #[must_use]
    pub fn traceability_configs(&self) -> Vec<TraceabilityConfig> {
        let Some(def) = schema::item_type_def(self.as_str()) else {
            return Vec::new();
        };

        def.allowed_targets
            .iter()
            .filter(|t| {
                schema::relation_def(&t.relation).is_some_and(|r| {
                    r.direction == RelationDirection::Upstream
                        || (r.direction == RelationDirection::Peer && r.primary)
                })
            })
            .flat_map(|t| {
                let relationship = RelationshipType::from_id(&t.relation);
                t.targets.iter().filter_map(move |target| {
                    let target_type = ItemType::from_id(target)?;
                    Some(TraceabilityConfig {
                        relationship: relationship?,
                        target_type,
                    })
                })
            })
            .collect()
    }
}

/// Configuration for traceability relationships.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TraceabilityConfig {
    /// The relation carrying the link (refines, derives_from, satisfies, depends_on).
    pub relationship: RelationshipType,
    /// The target item type to link to (parent for hierarchical, same type for peers).
    pub target_type: ItemType,
}

impl fmt::Display for ItemType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl Serialize for ItemType {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.0)
    }
}

impl<'de> Deserialize<'de> for ItemType {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let id = String::deserialize(deserializer)?;
        Self::from_id(&id)
            .ok_or_else(|| serde::de::Error::custom(format!("unknown item type `{id}`")))
    }
}

/// Unique identifier for an item across all repositories.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
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

/// Type-specific attributes for items, stored as an ordered field map.
///
/// Each declared field of the active schema maps to a [`FieldValue`] entry
/// keyed by the field's canonical snake_case name. The map preserves
/// declaration order so that template output remains stable.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(transparent)]
pub struct ItemAttributes {
    fields: IndexMap<String, FieldValue>,
}

impl ItemAttributes {
    /// Creates an empty attribute map.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the value for a field, if present.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&FieldValue> {
        self.fields.get(name)
    }

    /// Inserts a value for a field, returning the previous value if any.
    pub fn insert(&mut self, name: impl Into<String>, value: FieldValue) -> Option<FieldValue> {
        self.fields.insert(name.into(), value)
    }

    /// Removes a field, returning the previous value if any.
    pub fn remove(&mut self, name: &str) -> Option<FieldValue> {
        self.fields.shift_remove(name)
    }

    /// Returns an iterator over `(field_name, value)` pairs in declaration order.
    pub fn iter(&self) -> indexmap::map::Iter<'_, String, FieldValue> {
        self.fields.iter()
    }

    /// Returns true when the attribute map is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// Returns the number of fields currently stored.
    #[must_use]
    pub fn len(&self) -> usize {
        self.fields.len()
    }
}

/// An item of the knowledge graph: a Markdown file whose frontmatter declares
/// an id, a type and a name.
///
/// It serializes as clients read it: the core fields under their frontmatter
/// names, the path of its file, the relations its file declares and the
/// values of its fields.
// The core keys mirror `FIELD_ID`, `FIELD_TYPE`, `FIELD_NAME` and
// `FIELD_DESCRIPTION`; serde attributes cannot reference constants, so a
// guard test pins the correspondence.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct Item {
    /// Item id, such as `SYSREQ-001`.
    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    pub id: ItemId,

    /// Item type id, as declared in the model.
    #[serde(rename = "type")]
    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    pub item_type: ItemType,

    /// Item name.
    pub name: String,

    /// Item description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Location of the file holding the item.
    #[serde(rename = "file", serialize_with = "serialize_file")]
    #[cfg_attr(
        feature = "schemars",
        schemars(
            with = "String",
            description = "Path of the Markdown file holding the item."
        )
    )]
    pub source: SourceLocation,

    /// Relationships declared in the item's file.
    #[serde(
        rename = "relations",
        default,
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "serialize_relations"
    )]
    #[cfg_attr(
        feature = "schemars",
        schemars(
            with = "IndexMap<String, Vec<String>>",
            description = "Ids of the items the item's file links to, by relation id."
        )
    )]
    pub relationships: Vec<Relationship>,

    /// Values of the type-specific fields the item sets.
    #[serde(
        rename = "fields",
        default,
        skip_serializing_if = "ItemAttributes::is_empty"
    )]
    #[cfg_attr(
        feature = "schemars",
        schemars(
            with = "IndexMap<String, FieldValue>",
            description = "Values of the fields the item sets, by field name."
        )
    )]
    pub attributes: ItemAttributes,
}

/// Serializes the location of an item as the path of its file.
fn serialize_file<S: Serializer>(
    source: &SourceLocation,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serialize_path(&source.full_path(), serializer)
}

/// Serializes relationships as the ids of their targets by relation, in the
/// order the relations first appear.
fn serialize_relations<S: Serializer>(
    relationships: &[Relationship],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let mut targets: IndexMap<RelationshipType, Vec<&ItemId>> = IndexMap::new();
    for relationship in relationships {
        targets
            .entry(relationship.relationship_type)
            .or_default()
            .push(&relationship.to);
    }
    serializer.collect_map(targets)
}

impl Item {
    /// Returns an iterator over target IDs for a specific relationship type.
    pub fn relationship_ids(&self, rel_type: RelationshipType) -> impl Iterator<Item = &ItemId> {
        self.relationships
            .iter()
            .filter(move |r| r.relationship_type == rel_type)
            .map(|r| &r.to)
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
        self.relationships.iter().map(|r| &r.to)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_yaml::Value;

    use crate::graph::KnowledgeGraphBuilder;
    use crate::model::ItemBuilder;
    use crate::schema::builtin;
    use crate::test_utils::create_test_item;

    /// Returns the keys of a serialized mapping, in order.
    fn keys(value: &Value) -> Vec<&str> {
        value
            .as_mapping()
            .unwrap()
            .keys()
            .map(|key| key.as_str().unwrap())
            .collect()
    }

    #[test]
    fn test_item_serializes_its_client_keys() {
        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("SYSREQ-001"))
            .item_type(builtin::SYSTEM_REQUIREMENT)
            .name("Response time")
            .description("Fast enough")
            .source(SourceLocation::new("/repo", "SYSREQ-001.md"))
            .relationships(vec![
                Relationship::new(ItemId::new_unchecked("SCEN-001"), builtin::DERIVES_FROM),
                Relationship::new(ItemId::new_unchecked("SCEN-002"), builtin::DERIVES_FROM),
            ])
            .attribute(
                "specification",
                FieldValue::text("The system SHALL respond."),
            )
            .build()
            .unwrap();

        let value = serde_yaml::to_value(&item).unwrap();

        assert_eq!(
            keys(&value),
            [
                FIELD_ID,
                FIELD_TYPE,
                FIELD_NAME,
                FIELD_DESCRIPTION,
                "file",
                "relations",
                "fields"
            ]
        );
        assert_eq!(value[FIELD_TYPE], "system_requirement");
        assert_eq!(
            value["file"],
            item.source.full_path().display().to_string().as_str()
        );
        assert_eq!(
            value["relations"]["derives_from"],
            Value::from(vec!["SCEN-001", "SCEN-002"])
        );
        assert_eq!(
            value["fields"]["specification"],
            "The system SHALL respond."
        );
    }

    #[test]
    fn test_item_leaves_out_its_empty_parts() {
        let item = create_test_item("SOL-001", builtin::SOLUTION);

        let value = serde_yaml::to_value(&item).unwrap();

        assert_eq!(keys(&value), [FIELD_ID, FIELD_TYPE, FIELD_NAME, "file"]);
    }

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
        assert_eq!(builtin::SOLUTION.display_name(), "Solution");
        assert_eq!(
            builtin::SYSTEM_REQUIREMENT.display_name(),
            "System Requirement"
        );
    }

    #[test]
    fn test_item_type_declared_field() {
        assert!(
            builtin::SYSTEM_REQUIREMENT
                .declared_field("specification")
                .is_some_and(|f| f.required)
        );
        assert!(builtin::SOLUTION.declared_field("specification").is_none());
        assert!(builtin::SCENARIO.declared_field("specification").is_none());
    }

    #[test]
    fn test_generate_id() {
        assert_eq!(builtin::SOLUTION.generate_id(Some(1)), "SOL-001");
        assert_eq!(builtin::USE_CASE.generate_id(Some(42)), "UC-042");
        assert_eq!(builtin::SYSTEM_REQUIREMENT.generate_id(None), "SYSREQ-001");
    }

    #[test]
    fn test_generate_id_follows_the_builtin_format() {
        assert_eq!(builtin::SOLUTION.generate_id(None), "SOL-001");
        assert_eq!(builtin::SOLUTION.generate_id(Some(42)), "SOL-042");
    }

    #[test]
    fn test_generate_id_for_a_type_missing_from_the_schema() {
        let ghost = ItemType::from_static("ghost");
        assert_eq!(ghost.generate_id(None), "-001");
    }

    #[test]
    fn test_suggest_next_id_scans_legacy_unpadded_ids() {
        let graph = KnowledgeGraphBuilder::new()
            .add_item(create_test_item("SOL-7", builtin::SOLUTION))
            .add_item(create_test_item("SOL-002", builtin::SOLUTION))
            .build()
            .unwrap();
        assert_eq!(builtin::SOLUTION.suggest_next_id(Some(&graph)), "SOL-008");
    }
}

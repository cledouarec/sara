//! Item types and structures for the knowledge graph.

use std::fmt;
use std::str::FromStr;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use super::adr::AdrStatus;
use crate::error::SaraError;
use crate::model::FieldName;
use crate::model::field::FieldValue;
use crate::model::relationship::{Relationship, RelationshipType};
use crate::schema::{self, RelationDirection};

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
    /// Solution has no parent (root of the hierarchy). Resolved from the
    /// first entry of the active schema's `parent_types` list.
    #[must_use]
    pub fn required_parent_type(&self) -> Option<ItemType> {
        schema::item_type_def(self.as_str())
            .and_then(|d| d.parent_types.first())
            .and_then(|id| ItemType::from_id(id))
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

    /// Returns the variant matching the given schema id, if any.
    ///
    /// Inverse of [`ItemType::as_str`]. Used when mapping a schema definition
    /// back to a concrete enum variant.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        Self::all().iter().copied().find(|t| t.as_str() == id)
    }

    /// Returns all traceability configurations for this item type.
    ///
    /// Most item types have a single traceability link (e.g., refines,
    /// satisfies). Requirement types have two: derives_from (hierarchical) and
    /// depends_on (peer). Roots (Solution) and ADR-like detached types may
    /// have none.
    ///
    /// Derived from the active schema's `allowed_targets`, filtered to keep
    /// upstream relations and the `depends_on` peer relation, mirroring the
    /// legacy semantics. Targets that do not map to a known [`ItemType`]
    /// variant are skipped (relevant once custom schemas introduce new types
    /// in later phases).
    #[must_use]
    pub fn traceability_configs(&self) -> Vec<TraceabilityConfig> {
        let Some(def) = schema::item_type_def(self.as_str()) else {
            return Vec::new();
        };

        def.allowed_targets
            .iter()
            .filter(|t| {
                schema::relation_def(&t.relation).is_some_and(|r| {
                    r.direction == RelationDirection::Upstream || t.relation == "depends_on"
                })
            })
            .flat_map(|t| {
                let field = RelationshipType::from_id(&t.relation).map(|r| r.field_name());
                t.targets.iter().filter_map(move |target| {
                    let target_type = ItemType::from_id(target)?;
                    Some(TraceabilityConfig {
                        relationship_field: field?,
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

/// Canonical field name for the specification text.
const FIELD_SPECIFICATION: &str = "specification";
/// Canonical field name for the platform string.
const FIELD_PLATFORM: &str = "platform";
/// Canonical field name for the ADR lifecycle status.
const FIELD_STATUS: &str = "status";
/// Canonical field name for the ADR deciders list.
const FIELD_DECIDERS: &str = "deciders";
/// Canonical field name for the peer-dependency list.
const FIELD_DEPENDS_ON: &str = "depends_on";
/// Canonical field name for the ADR supersession list.
const FIELD_SUPERSEDES: &str = "supersedes";

/// Type-specific attributes for items, stored as an ordered field map.
///
/// Each declared field of the active schema maps to a [`FieldValue`] entry
/// keyed by the field's canonical snake_case name. The map preserves
/// declaration order so that template output remains stable.
///
/// Legacy accessors ([`Self::specification`], [`Self::status`], [`Self::platform`],
/// [`Self::deciders`], [`Self::depends_on`], [`Self::supersedes`]) read from the
/// map and convert to the historical return types so existing callers keep
/// working unchanged.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

    /// Creates default attributes for the given item type.
    ///
    /// In the map representation, "default" means an empty field set; the
    /// builder and parsers populate the required entries as they construct
    /// the item.
    #[must_use]
    pub fn for_type(_item_type: ItemType) -> Self {
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

    /// Returns the specification text if a `specification` text field is set.
    #[must_use]
    pub fn specification(&self) -> Option<&String> {
        self.get(FIELD_SPECIFICATION).and_then(FieldValue::as_text)
    }

    /// Returns the depends_on references as owned [`ItemId`]s.
    #[must_use]
    pub fn depends_on(&self) -> Vec<ItemId> {
        collect_item_refs(self.get(FIELD_DEPENDS_ON))
    }

    /// Returns the platform text if a `platform` text field is set.
    #[must_use]
    pub fn platform(&self) -> Option<&String> {
        self.get(FIELD_PLATFORM).and_then(FieldValue::as_text)
    }

    /// Returns the ADR lifecycle status if a `status` enum field is set.
    #[must_use]
    pub fn status(&self) -> Option<AdrStatus> {
        self.get(FIELD_STATUS)
            .and_then(FieldValue::as_enum)
            .and_then(|s| AdrStatus::from_str(s).ok())
    }

    /// Returns the ADR deciders as owned strings.
    #[must_use]
    pub fn deciders(&self) -> Vec<String> {
        self.get(FIELD_DECIDERS)
            .and_then(FieldValue::as_list)
            .map(|list| {
                list.iter()
                    .filter_map(FieldValue::as_text)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Returns the supersedes references as owned [`ItemId`]s.
    #[must_use]
    pub fn supersedes(&self) -> Vec<ItemId> {
        collect_item_refs(self.get(FIELD_SUPERSEDES))
    }
}

/// Extracts the inner [`ItemId`]s from a [`FieldValue::List`] of [`FieldValue::ItemRef`].
fn collect_item_refs(value: Option<&FieldValue>) -> Vec<ItemId> {
    value
        .and_then(FieldValue::as_list)
        .map(|list| {
            list.iter()
                .filter_map(FieldValue::as_item_ref)
                .cloned()
                .collect()
        })
        .unwrap_or_default()
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

        // Peer references stored as item-ref lists in the attribute map
        // (`depends_on` for requirements, `supersedes` for ADRs).
        let peer_refs: Vec<&ItemId> = [FIELD_DEPENDS_ON, FIELD_SUPERSEDES]
            .iter()
            .filter_map(|name| self.attributes.get(name).and_then(FieldValue::as_list))
            .flat_map(|list| list.iter().filter_map(FieldValue::as_item_ref))
            .collect();

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

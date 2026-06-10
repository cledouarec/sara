//! Field name definitions and runtime field values.
//!
//! Defines a single source of truth for all field names used in YAML
//! frontmatter, plus a typed [`FieldValue`] enum used to store the
//! runtime value of declared fields in an item's attribute map.

use serde::{Deserialize, Serialize};

use super::item::ItemId;

/// All field names used in YAML frontmatter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldName {
    // Required fields
    Id,
    Type,
    Name,

    // Optional metadata
    Description,

    // Upstream traceability (relationships)
    Refines,
    DerivesFrom,
    Satisfies,

    // Downstream traceability (relationships)
    IsRefinedBy,
    Derives,
    IsSatisfiedBy,

    // Peer relationships
    DependsOn,
    IsRequiredBy,

    // Requirement-specific fields
    Specification,
    Platform,
    JustifiedBy,

    // ADR-specific fields
    Status,
    Deciders,
    Justifies,
    Supersedes,
    SupersededBy,
}

impl FieldName {
    /// Returns the YAML field name (snake_case).
    ///
    /// Used for serialization, deserialization, and error messages.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::Type => "type",
            Self::Name => "name",
            Self::Description => "description",
            Self::Refines => "refines",
            Self::DerivesFrom => "derives_from",
            Self::Satisfies => "satisfies",
            Self::IsRefinedBy => "is_refined_by",
            Self::Derives => "derives",
            Self::IsSatisfiedBy => "is_satisfied_by",
            Self::DependsOn => "depends_on",
            Self::IsRequiredBy => "is_required_by",
            Self::Specification => "specification",
            Self::Platform => "platform",
            Self::JustifiedBy => "justified_by",
            Self::Status => "status",
            Self::Deciders => "deciders",
            Self::Justifies => "justifies",
            Self::Supersedes => "supersedes",
            Self::SupersededBy => "superseded_by",
        }
    }

    /// Returns the human-readable display name.
    ///
    /// Used for user-facing output like change summaries.
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Id => "ID",
            Self::Type => "Type",
            Self::Name => "Name",
            Self::Description => "Description",
            Self::Refines => "Refines",
            Self::DerivesFrom => "Derives from",
            Self::Satisfies => "Satisfies",
            Self::IsRefinedBy => "Is refined by",
            Self::Derives => "Derives",
            Self::IsSatisfiedBy => "Is satisfied by",
            Self::DependsOn => "Depends on",
            Self::IsRequiredBy => "Is required by",
            Self::Specification => "Specification",
            Self::Platform => "Platform",
            Self::JustifiedBy => "Justified by",
            Self::Status => "Status",
            Self::Deciders => "Deciders",
            Self::Justifies => "Justifies",
            Self::Supersedes => "Supersedes",
            Self::SupersededBy => "Superseded by",
        }
    }

    /// Returns true if this is an upstream traceability field.
    pub const fn is_upstream(&self) -> bool {
        matches!(
            self,
            Self::Refines | Self::DerivesFrom | Self::Satisfies | Self::Justifies
        )
    }

    /// Returns true if this is a downstream traceability field.
    pub const fn is_downstream(&self) -> bool {
        matches!(
            self,
            Self::IsRefinedBy | Self::Derives | Self::IsSatisfiedBy | Self::JustifiedBy
        )
    }

    /// Returns true if this is a peer relationship field.
    pub const fn is_peer(&self) -> bool {
        matches!(
            self,
            Self::DependsOn | Self::IsRequiredBy | Self::Supersedes | Self::SupersededBy
        )
    }

    /// Returns true if this is a traceability field (upstream, downstream, or peer).
    pub const fn is_traceability(&self) -> bool {
        self.is_upstream() || self.is_downstream() || self.is_peer()
    }
}

impl std::fmt::Display for FieldName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Runtime value of a declared field, mirroring [`crate::schema::FieldType`].
///
/// Item attributes are stored as a map of `String -> FieldValue` so that the
/// model can express schema-declared fields uniformly. Each variant carries
/// the concrete payload for a declared type:
///
/// - [`FieldValue::Text`]: free-form text (specification, platform).
/// - [`FieldValue::Enum`]: a value from a closed set (e.g., ADR status).
/// - [`FieldValue::ItemRef`]: a single reference to another item's id.
/// - [`FieldValue::List`]: an ordered list of values (deciders, depends_on).
/// - [`FieldValue::Date`]: an ISO-8601 date held as a string.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldValue {
    /// Free-form text.
    Text(String),
    /// One value among a closed set, kept as its snake_case identifier.
    Enum(String),
    /// A single reference to another item's id.
    ItemRef(ItemId),
    /// An ordered list of values.
    List(Vec<FieldValue>),
    /// An ISO-8601 date string.
    Date(String),
}

impl FieldValue {
    /// Returns the inner string if this is a [`FieldValue::Text`].
    #[must_use]
    pub fn as_text(&self) -> Option<&String> {
        if let Self::Text(s) = self {
            Some(s)
        } else {
            None
        }
    }

    /// Returns the inner string if this is a [`FieldValue::Enum`].
    #[must_use]
    pub fn as_enum(&self) -> Option<&String> {
        if let Self::Enum(s) = self {
            Some(s)
        } else {
            None
        }
    }

    /// Returns the inner [`ItemId`] if this is a [`FieldValue::ItemRef`].
    #[must_use]
    pub fn as_item_ref(&self) -> Option<&ItemId> {
        if let Self::ItemRef(id) = self {
            Some(id)
        } else {
            None
        }
    }

    /// Returns the inner slice if this is a [`FieldValue::List`].
    #[must_use]
    pub fn as_list(&self) -> Option<&[FieldValue]> {
        if let Self::List(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns the inner string if this is a [`FieldValue::Date`].
    #[must_use]
    pub fn as_date(&self) -> Option<&String> {
        if let Self::Date(s) = self {
            Some(s)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_name_as_str() {
        assert_eq!(FieldName::Id.as_str(), "id");
        assert_eq!(FieldName::DerivesFrom.as_str(), "derives_from");
        assert_eq!(FieldName::Specification.as_str(), "specification");
    }

    #[test]
    fn test_field_name_is_upstream() {
        assert!(FieldName::Refines.is_upstream());
        assert!(FieldName::DerivesFrom.is_upstream());
        assert!(FieldName::Satisfies.is_upstream());
        assert!(!FieldName::IsRefinedBy.is_upstream());
        assert!(!FieldName::Specification.is_upstream());
    }

    #[test]
    fn test_field_name_is_downstream() {
        assert!(FieldName::IsRefinedBy.is_downstream());
        assert!(FieldName::Derives.is_downstream());
        assert!(FieldName::IsSatisfiedBy.is_downstream());
        assert!(!FieldName::Refines.is_downstream());
    }

    #[test]
    fn test_field_name_display() {
        assert_eq!(format!("{}", FieldName::DerivesFrom), "derives_from");
    }
}

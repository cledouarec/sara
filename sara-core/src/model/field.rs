//! Runtime field values for item attributes.
//!
//! Defines the typed [`FieldValue`] enum used to store the runtime value of
//! schema-declared fields in an item's attribute map.

use serde::{Deserialize, Serialize};

use super::item::ItemId;

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
    /// Creates a [`FieldValue::Text`] from anything string-like.
    #[must_use]
    pub fn text(value: impl Into<String>) -> Self {
        Self::Text(value.into())
    }

    /// Creates a [`FieldValue::List`] of [`FieldValue::Text`] entries.
    #[must_use]
    pub fn text_list(values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self::List(values.into_iter().map(Self::text).collect())
    }

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

impl std::fmt::Display for FieldValue {
    /// Renders the value for user-facing output (change summaries, diffs):
    /// scalars print their inner text, lists join their entries with `, `.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(s) | Self::Enum(s) | Self::Date(s) => write!(f, "{s}"),
            Self::ItemRef(id) => write!(f, "{id}"),
            Self::List(values) => {
                let rendered: Vec<String> = values.iter().map(ToString::to_string).collect();
                write!(f, "{}", rendered.join(", "))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_value_accessors() {
        assert_eq!(
            FieldValue::Text("spec".to_string()).as_text(),
            Some(&"spec".to_string())
        );
        assert_eq!(
            FieldValue::Enum("proposed".to_string()).as_enum(),
            Some(&"proposed".to_string())
        );
        assert!(FieldValue::Text("spec".to_string()).as_enum().is_none());
    }
}

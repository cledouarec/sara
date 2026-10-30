//! Runtime field values for item attributes.
//!
//! Defines the typed [`FieldValue`] enum used to store the runtime value of
//! schema-declared fields in an item's attribute map.

#[cfg(feature = "schemars")]
use std::borrow::Cow;

#[cfg(feature = "schemars")]
use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Serialize, Serializer};

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
///
/// It serializes as clients read it: a list as an array of its entries'
/// text, any other value as its text.
#[derive(Debug, Clone, PartialEq, Eq)]
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

impl Serialize for FieldValue {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::List(values) => serializer.collect_seq(values.iter().map(ToString::to_string)),
            scalar => serializer.collect_str(scalar),
        }
    }
}

#[cfg(feature = "schemars")]
impl JsonSchema for FieldValue {
    fn inline_schema() -> bool {
        true
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("FieldValue")
    }

    fn json_schema(_generator: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "description": "A string, or a list of strings for a list field.",
            "anyOf": [
                { "type": "string" },
                { "type": "array", "items": { "type": "string" } }
            ]
        })
    }
}

#[cfg(test)]
mod tests {
    use serde_yaml::Value;

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

    #[test]
    fn test_field_value_serializes_scalars_as_strings() {
        let scalars = [
            FieldValue::text("spec"),
            FieldValue::Enum("proposed".to_string()),
            FieldValue::ItemRef(ItemId::new_unchecked("SYSREQ-001")),
            FieldValue::Date("2026-10-30".to_string()),
        ];

        let values: Vec<Value> = scalars
            .iter()
            .map(|scalar| serde_yaml::to_value(scalar).unwrap())
            .collect();

        assert_eq!(
            values,
            ["spec", "proposed", "SYSREQ-001", "2026-10-30"].map(Value::from)
        );
    }

    #[test]
    fn test_field_value_serializes_lists_as_strings() {
        let list = FieldValue::List(vec![
            FieldValue::text("alice"),
            FieldValue::text_list(["bob", "carol"]),
        ]);

        assert_eq!(
            serde_yaml::to_value(&list).unwrap(),
            Value::from(vec!["alice", "bob, carol"])
        );
    }
}

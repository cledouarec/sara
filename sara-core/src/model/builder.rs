//! Builder for constructing `Item` instances.

use std::path::PathBuf;

use super::adr::AdrStatus;
use super::field::FieldValue;
use super::item::{Item, ItemAttributes, ItemId, ItemType};
use super::metadata::SourceLocation;
use super::relationship::{Relationship, RelationshipType};
use crate::error::SaraError;
use crate::schema;

/// Field name used for the requirement specification text.
const FIELD_SPECIFICATION: &str = "specification";
/// Field name used for the system-architecture platform string.
const FIELD_PLATFORM: &str = "platform";
/// Field name used for the ADR lifecycle status.
const FIELD_STATUS: &str = "status";
/// Field name used for the ADR deciders list.
const FIELD_DECIDERS: &str = "deciders";

/// Builder for constructing `Item` instances from parsed frontmatter.
///
/// Type-specific values are accumulated directly into an [`ItemAttributes`]
/// map; build-time validation enforces the required entries for each
/// [`ItemType`].
#[derive(Debug, Default)]
pub struct ItemBuilder {
    id: Option<ItemId>,
    item_type: Option<ItemType>,
    name: Option<String>,
    description: Option<String>,
    source: Option<SourceLocation>,
    relationships: Vec<Relationship>,
    attributes: ItemAttributes,
}

impl ItemBuilder {
    /// Creates a new `ItemBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the item ID.
    pub fn id(mut self, id: ItemId) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the item type.
    pub fn item_type(mut self, item_type: ItemType) -> Self {
        self.item_type = Some(item_type);
        self
    }

    /// Sets the item name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the item description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Sets the source location.
    pub fn source(mut self, source: SourceLocation) -> Self {
        self.source = Some(source);
        self
    }

    /// Adds relationships for this item.
    pub fn relationships(mut self, relationships: Vec<Relationship>) -> Self {
        self.relationships.extend(relationships);
        self
    }

    /// Sets the specification text (for requirement types).
    pub fn specification(mut self, spec: impl Into<String>) -> Self {
        self.attributes
            .insert(FIELD_SPECIFICATION, FieldValue::Text(spec.into()));
        self
    }

    /// Sets the platform (for `SystemArchitecture`).
    pub fn platform(mut self, platform: impl Into<String>) -> Self {
        self.attributes
            .insert(FIELD_PLATFORM, FieldValue::Text(platform.into()));
        self
    }

    /// Adds a peer dependency relationship.
    pub fn depends_on(mut self, id: ItemId) -> Self {
        self.relationships
            .push(Relationship::new(id, RelationshipType::DEPENDS_ON));
        self
    }

    /// Sets the ADR status.
    pub fn status(mut self, status: AdrStatus) -> Self {
        self.attributes
            .insert(FIELD_STATUS, FieldValue::Enum(status.as_str().to_string()));
        self
    }

    /// Adds a decider (for ADR).
    pub fn decider(mut self, decider: impl Into<String>) -> Self {
        push_text(&mut self.attributes, FIELD_DECIDERS, decider.into());
        self
    }

    /// Sets the deciders (for ADR).
    pub fn deciders(mut self, deciders: Vec<String>) -> Self {
        self.attributes.insert(
            FIELD_DECIDERS,
            FieldValue::List(deciders.into_iter().map(FieldValue::Text).collect()),
        );
        self
    }

    /// Adds a supersession relationship toward an older peer.
    pub fn supersedes(mut self, id: ItemId) -> Self {
        self.relationships
            .push(Relationship::new(id, RelationshipType::SUPERSEDES));
        self
    }

    /// Adds supersession relationships toward several older peers.
    pub fn supersedes_all(mut self, ids: Vec<ItemId>) -> Self {
        self.relationships.extend(
            ids.into_iter()
                .map(|id| Relationship::new(id, RelationshipType::SUPERSEDES)),
        );
        self
    }

    /// Sets the value of a declared field by name.
    pub fn attribute(mut self, name: impl Into<String>, value: FieldValue) -> Self {
        self.attributes.insert(name, value);
        self
    }

    /// Replaces the entire attribute map with the supplied one.
    pub fn attributes(mut self, attrs: ItemAttributes) -> Self {
        self.attributes = attrs;
        self
    }

    /// Validates that the fields the schema marks as required are populated.
    ///
    /// A required list field must also be non-empty, since an empty list
    /// carries no more information than an absent one.
    fn validate_required_attributes(
        &self,
        item_type: ItemType,
        file: &str,
    ) -> Result<(), SaraError> {
        let Some(def) = schema::item_type_def(item_type.as_str()) else {
            return Ok(());
        };

        for field in def.fields.iter().filter(|f| f.required) {
            let satisfied = match self.attributes.get(&field.name) {
                Some(FieldValue::List(values)) => !values.is_empty(),
                Some(_) => true,
                None => false,
            };
            if !satisfied {
                return Err(SaraError::MissingField {
                    field: field.name.clone(),
                    file: PathBuf::from(file),
                });
            }
        }

        Ok(())
    }

    /// Builds the `Item`, returning an error if required fields are missing.
    ///
    /// # Errors
    ///
    /// Returns `SaraError::MissingField` if a required field (id, type,
    /// name, source, or type-specific attributes) is not set.
    pub fn build(self) -> Result<Item, SaraError> {
        let id = self.id.clone().ok_or_else(|| SaraError::MissingField {
            field: "id".to_string(),
            file: self
                .source
                .as_ref()
                .map(|s| s.file_path.clone())
                .unwrap_or_default(),
        })?;

        let item_type = self.item_type.ok_or_else(|| SaraError::MissingField {
            field: "type".to_string(),
            file: self
                .source
                .as_ref()
                .map(|s| s.file_path.clone())
                .unwrap_or_default(),
        })?;

        let name = self.name.clone().ok_or_else(|| SaraError::MissingField {
            field: "name".to_string(),
            file: self
                .source
                .as_ref()
                .map(|s| s.file_path.clone())
                .unwrap_or_default(),
        })?;

        let source = self.source.clone().ok_or_else(|| SaraError::MissingField {
            field: "source".to_string(),
            file: PathBuf::new(),
        })?;

        let file_path = source.file_path.display().to_string();
        self.validate_required_attributes(item_type, &file_path)?;

        Ok(Item {
            id,
            item_type,
            name,
            description: self.description,
            source,
            relationships: self.relationships,
            attributes: self.attributes,
        })
    }
}

/// Appends a `Text` value to the list stored under `field`, creating the
/// list entry if it does not yet exist.
fn push_text(attrs: &mut ItemAttributes, field: &str, text: String) {
    push_value(attrs, field, FieldValue::Text(text));
}

/// Appends a [`FieldValue`] to the list stored under `field`. If the field
/// already holds a non-list value, it is replaced with a new single-element
/// list — matching the prior builder semantics where typed setters always
/// appended into the same logical collection.
fn push_value(attrs: &mut ItemAttributes, field: &str, value: FieldValue) {
    match attrs.get(field) {
        Some(FieldValue::List(existing)) => {
            let mut updated = existing.clone();
            updated.push(value);
            attrs.insert(field.to_string(), FieldValue::List(updated));
        }
        _ => {
            attrs.insert(field.to_string(), FieldValue::List(vec![value]));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::relationship::RelationshipType;

    #[test]
    fn test_item_builder() {
        let source = SourceLocation {
            repository: PathBuf::from("/repo"),
            file_path: PathBuf::from("docs/SOL-001.md"),
            git_ref: None,
        };

        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("SOL-001"))
            .item_type(ItemType::SOLUTION)
            .name("Test Solution")
            .source(source)
            .build();

        assert!(item.is_ok());
        let item = item.unwrap();
        assert_eq!(item.id.as_str(), "SOL-001");
        assert_eq!(item.name, "Test Solution");
    }

    #[test]
    fn test_item_builder_with_relationships() {
        let source = SourceLocation {
            repository: PathBuf::from("/repo"),
            file_path: PathBuf::from("docs/UC-001.md"),
            git_ref: None,
        };

        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("UC-001"))
            .item_type(ItemType::USE_CASE)
            .name("Test Use Case")
            .source(source)
            .relationships(vec![Relationship::new(
                ItemId::new_unchecked("SOL-001"),
                RelationshipType::REFINES,
            )])
            .build()
            .unwrap();

        let refines: Vec<_> = item.relationship_ids(RelationshipType::REFINES).collect();
        assert_eq!(refines.len(), 1);
        assert_eq!(refines[0].as_str(), "SOL-001");
    }
}

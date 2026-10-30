//! Checks of the field values and relation targets given to write tools.
//!
//! The core services ignore names a type does not declare. The write tools
//! check every input up front instead, so an agent learns exactly what to
//! fix and nothing is written on a mistake.

use std::collections::BTreeMap;

use sara_core::graph::KnowledgeGraph;
use sara_core::model::{ItemType, RelationshipRules, RelationshipType};
use sara_core::schema::FieldType;
use sara_core::service::FieldInput;

use crate::server::find_item;

/// Checks field values against the fields `item_type` declares and turns
/// them into inputs of the core services.
///
/// # Errors
///
/// Returns a message for a field the type does not declare, a list given to
/// a single-value field, or a value an enumeration does not allow.
pub fn field_inputs(
    item_type: ItemType,
    fields: BTreeMap<String, FieldInput>,
) -> Result<Vec<(String, FieldInput)>, String> {
    fields
        .into_iter()
        .map(|(name, value)| {
            let Some(field) = item_type.declared_field(&name) else {
                return Err(format!(
                    "{} declares no field '{name}'; its fields are: {}.",
                    item_type.as_str(),
                    field_names(item_type)
                ));
            };
            let input = match (&field.field_type, value) {
                (FieldType::List(_), FieldInput::Text(value)) => FieldInput::List(vec![value]),
                (FieldType::List(_), list @ FieldInput::List(_)) => list,
                (_, FieldInput::List(_)) => {
                    return Err(format!("Field '{name}' takes a single value, not a list."));
                }
                (FieldType::Enum { values }, FieldInput::Text(value))
                    if !values.contains(&value) =>
                {
                    return Err(format!(
                        "'{value}' is not a value of field '{name}'; expected one of: {}.",
                        values.join(", ")
                    ));
                }
                (_, text @ FieldInput::Text(_)) => text,
            };
            Ok((name, input))
        })
        .collect()
}

/// Checks relation targets given for an item of `item_type`.
///
/// Each relation must be among `allowed`, each target must be an item of
/// `graph`, and the model must allow the relation from `item_type` to the
/// target's type.
///
/// # Errors
///
/// Returns a message naming the relation or target at fault and what would
/// be accepted instead.
pub fn relation_inputs(
    graph: &KnowledgeGraph,
    item_type: ItemType,
    relations: BTreeMap<String, Vec<String>>,
    allowed: &[RelationshipType],
) -> Result<Vec<(RelationshipType, Vec<String>)>, String> {
    relations
        .into_iter()
        .map(|(name, targets)| {
            let relation = RelationshipType::from_id(&name)
                .filter(|relation| allowed.contains(relation))
                .ok_or_else(|| {
                    format!(
                        "{} items cannot declare '{name}' here; allowed relations: {}.",
                        item_type.as_str(),
                        names(allowed.iter().map(RelationshipType::as_str))
                    )
                })?;
            for target in &targets {
                let target_type = find_item(graph, target)?.item_type;
                if !RelationshipRules::is_valid_relationship(item_type, target_type, relation) {
                    return Err(format!(
                        "{target} is a {}, but '{name}' from a {} accepts only: {}.",
                        target_type.as_str(),
                        item_type.as_str(),
                        names(target_types(item_type, relation))
                    ));
                }
            }
            Ok((relation, targets))
        })
        .collect()
}

/// Lists the fields `item_type` declares.
fn field_names(item_type: ItemType) -> String {
    names(
        item_type
            .declared_fields()
            .iter()
            .map(|field| field.name.as_str()),
    )
}

/// Lists the item types `relation` may lead to from an item of `item_type`.
fn target_types(item_type: ItemType, relation: RelationshipType) -> Vec<&'static str> {
    ItemType::all()
        .into_iter()
        .filter(|target| RelationshipRules::is_valid_relationship(item_type, *target, relation))
        .map(|target| target.as_str())
        .collect()
}

/// Joins names for a message, or says there are none.
fn names<'a>(names: impl IntoIterator<Item = &'a str>) -> String {
    let names: Vec<&str> = names.into_iter().collect();
    if names.is_empty() {
        "none".to_string()
    } else {
        names.join(", ")
    }
}

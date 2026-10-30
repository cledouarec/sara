//! Items directly linked to an item by one relation.

use serde::Serialize;

use crate::model::{ItemSummary, RelationshipType};
use crate::schema::RelationDirection;

/// Items linked to an item by one relation.
///
/// Each link is read from the item: a link the item's file does not declare
/// appears under the inverse of the relation the linked item declares.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct RelationGroup {
    /// Relation id, read from the item.
    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    pub relation: RelationshipType,
    /// Human-readable name of the relation.
    pub label: &'static str,
    /// Direction of the relation; absent when the model does not declare it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direction: Option<RelationDirection>,
    /// The linked items, sorted by id.
    pub items: Vec<ItemSummary>,
}

#[cfg(test)]
mod tests {
    use serde_yaml::Value;

    use super::*;
    use crate::schema::builtin;
    use crate::test_utils::create_test_item;

    /// A group linking to one software requirement through `relation`.
    fn group(relation: RelationshipType) -> RelationGroup {
        let requirement = create_test_item("SWREQ-001", builtin::SOFTWARE_REQUIREMENT);
        RelationGroup {
            relation,
            label: relation.display_name(),
            direction: relation.direction(),
            items: vec![ItemSummary::from(&requirement)],
        }
    }

    #[test]
    fn test_relation_group_serializes_relation_and_items() {
        let value = serde_yaml::to_value(group(builtin::SATISFIES)).unwrap();

        assert_eq!(value["relation"], Value::from("satisfies"));
        assert_eq!(
            value["label"],
            Value::from(builtin::SATISFIES.display_name())
        );
        assert_eq!(value["direction"], Value::from("upstream"));
        assert_eq!(value["items"][0]["id"], Value::from("SWREQ-001"));
        assert_eq!(
            value["items"][0]["type"],
            Value::from("software_requirement")
        );
    }

    #[test]
    fn test_relation_group_leaves_out_an_unknown_direction() {
        let mut group = group(builtin::SATISFIES);
        group.direction = None;

        let value = serde_yaml::to_value(group).unwrap();

        assert!(value.get("direction").is_none());
    }
}

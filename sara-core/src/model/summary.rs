//! Short form of an item, enough to identify it.

use serde::Serialize;

use super::item::{Item, ItemId, ItemType};

/// Reference to an item: enough to identify it and decide whether to read it.
// The keys mirror `FIELD_ID`, `FIELD_TYPE` and `FIELD_NAME`; serde attributes
// cannot reference constants, so a guard test pins the correspondence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct ItemSummary {
    /// Item id, such as `SYSREQ-001`.
    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    pub id: ItemId,
    /// Item type id, as declared in the model.
    #[serde(rename = "type")]
    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    pub item_type: ItemType,
    /// Item name.
    pub name: String,
}

impl From<&Item> for ItemSummary {
    fn from(item: &Item) -> Self {
        Self {
            id: item.id.clone(),
            item_type: item.item_type,
            name: item.name.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_yaml::Value;

    use super::*;
    use crate::model::{FIELD_ID, FIELD_NAME, FIELD_TYPE};
    use crate::schema::builtin;
    use crate::test_utils::create_test_item;

    #[test]
    fn test_item_summary_serializes_the_core_keys() {
        let item = create_test_item("SOL-001", builtin::SOLUTION);

        let value = serde_yaml::to_value(ItemSummary::from(&item)).unwrap();

        let keys: Vec<&str> = value
            .as_mapping()
            .unwrap()
            .keys()
            .map(|key| key.as_str().unwrap())
            .collect();
        assert_eq!(keys, [FIELD_ID, FIELD_TYPE, FIELD_NAME]);
        assert_eq!(value[FIELD_ID], Value::from("SOL-001"));
        assert_eq!(value[FIELD_TYPE], Value::from("solution"));
        assert_eq!(value[FIELD_NAME], Value::from("Test SOL-001"));
    }
}

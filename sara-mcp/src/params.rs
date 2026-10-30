//! Parameters of the MCP tools.
//!
//! Doc comments become the descriptions of the JSON Schemas clients see.

use sara_core::model::ItemType;

/// Lists the item types of the active model as `id (PREFIX)` entries.
pub fn type_list() -> String {
    ItemType::all()
        .iter()
        .map(|item_type| format!("{} ({})", item_type.as_str(), item_type.prefix()))
        .collect::<Vec<_>>()
        .join(", ")
}

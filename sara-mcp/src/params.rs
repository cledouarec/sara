//! Parameters of the MCP tools.
//!
//! Doc comments become the descriptions of the JSON Schemas clients see.

use sara_core::model::ItemType;
use sara_core::service::parse_item_type;
use schemars::JsonSchema;
use serde::Deserialize;

/// Fewest results a client may ask a tool for.
const MIN_LIMIT: usize = 1;

/// How many results a tool returns.
#[derive(Debug, Clone, Copy)]
pub struct Limit {
    /// Results returned when the client sets no limit.
    default: usize,
    /// Most results returned, whatever the client asks.
    max: usize,
}

impl Limit {
    /// Returns how many results to return for the limit a client asked for,
    /// if any.
    #[must_use]
    pub fn resolve(self, requested: Option<usize>) -> usize {
        requested.unwrap_or(self.default).clamp(MIN_LIMIT, self.max)
    }
}

/// Hits `sara_search_items` returns in one page.
pub const SEARCH_LIMIT: Limit = Limit {
    default: 20,
    max: 100,
};

/// Parameters of `sara_search_items`.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchItemsParams {
    /// Words to look for in item ids, names, descriptions, fields and
    /// Markdown bodies, ignoring case. Leave empty to list items by id.
    #[serde(default)]
    pub query: String,
    /// Item types to search, each by type id or ID prefix, such as
    /// `system_requirement` or `SYSREQ`. Leave empty to search all types.
    #[serde(default)]
    pub types: Vec<String>,
    /// Most hits to return, from 1 to 100. Defaults to 20.
    pub limit: Option<usize>,
    /// Number of hits to skip, to read the next page. Defaults to 0.
    #[serde(default)]
    pub offset: usize,
}

/// Resolves item type names given by a client, by type id or ID prefix.
///
/// # Errors
///
/// Returns a message naming the unknown type and listing the known ones.
pub fn parse_item_types(names: &[String]) -> Result<Vec<ItemType>, String> {
    names.iter().map(|name| parse_one_item_type(name)).collect()
}

/// Lists the item types of the active model as `id (PREFIX)` entries.
pub fn type_list() -> String {
    ItemType::all()
        .iter()
        .map(|item_type| format!("{} ({})", item_type.as_str(), item_type.prefix()))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Resolves one item type name given by a client, by type id or ID prefix.
///
/// # Errors
///
/// Returns a message naming the unknown type and listing the known ones.
pub fn parse_one_item_type(name: &str) -> Result<ItemType, String> {
    parse_item_type(name).ok_or_else(|| {
        format!(
            "Unknown item type '{name}'. Item types (id and ID prefix): {}.",
            type_list()
        )
    })
}

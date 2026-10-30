//! Results of the MCP tools that are more than one core type.
//!
//! The core types serialize as clients read them, so most tools return them
//! as they are. A view only puts several of them together and adds what a
//! tool computes on top: paging, counts by type, the validity of a check.

use sara_core::graph::SearchHit;
use schemars::JsonSchema;
use serde::Serialize;

/// One page of search hits.
#[derive(Debug, Serialize, JsonSchema)]
pub struct SearchResults {
    /// Number of matching items, over all pages.
    pub total: usize,
    /// Position of the first returned hit among all matches.
    pub offset: usize,
    /// Whether more hits follow this page.
    pub has_more: bool,
    /// Offset of the next page, when there is one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_offset: Option<usize>,
    /// Matching items, best first.
    pub hits: Vec<SearchHit>,
}

impl SearchResults {
    /// Keeps the `limit` hits starting at `offset`, with paging details.
    pub fn page(hits: Vec<SearchHit>, offset: usize, limit: usize) -> Self {
        let total = hits.len();
        let hits: Vec<SearchHit> = hits.into_iter().skip(offset).take(limit).collect();
        let end = offset.saturating_add(hits.len());
        let has_more = end < total;

        Self {
            total,
            offset,
            has_more,
            next_offset: has_more.then_some(end),
            hits,
        }
    }
}

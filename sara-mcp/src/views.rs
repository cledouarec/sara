//! Results of the MCP tools that are more than one core type.
//!
//! The core types serialize as clients read them, so most tools return them
//! as they are. A view only puts several of them together and adds what a
//! tool computes on top: paging, counts by type, the validity of a check.

use sara_core::graph::{KnowledgeGraph, Reach, RelationGroup, SearchHit, TraversalOptions};
use sara_core::model::{Item, ItemSummary};
use schemars::JsonSchema;
use serde::Serialize;

/// Full view of one item.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ItemView {
    /// The item, with the relations its file declares.
    #[serde(flatten)]
    pub item: Item,
    /// Human-readable name of the item type.
    pub type_name: &'static str,
    /// Items directly linked to this one, in both directions, by relation;
    /// a linked item missing from `relations` is linked from its own file.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<RelationGroup>,
    /// Markdown body of the item, after its frontmatter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

impl ItemView {
    /// Builds the view of `item`, with its direct links in `graph` and the
    /// given body.
    pub fn new(graph: &KnowledgeGraph, item: &Item, body: Option<String>) -> Self {
        Self {
            item: item.clone(),
            type_name: item.item_type.display_name(),
            links: graph.direct_relationships(&item.id, &TraversalOptions::new()),
            body,
        }
    }
}

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

/// Traceability chains of an item.
#[derive(Debug, Serialize, JsonSchema)]
pub struct TraceView {
    /// The traced item.
    pub origin: ItemSummary,
    /// Items the traced item traces up to, toward the solution.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upstream: Option<Reach>,
    /// Items tracing down from the traced item, toward detailed designs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub downstream: Option<Reach>,
}

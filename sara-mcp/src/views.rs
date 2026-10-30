//! Results of the MCP tools that are more than one core type.
//!
//! The core types serialize as clients read them, so most tools return them
//! as they are. A view only puts several of them together and adds what a
//! tool computes on top: paging, counts by type, the validity of a check.

use std::collections::BTreeMap;

use sara_core::graph::{
    KnowledgeGraph, Reach, ReachedItem, RelationGroup, SearchHit, TraversalOptions,
};
use sara_core::model::{Item, ItemId, ItemSummary, RelationshipType};
use sara_core::repository::ScanWarning;
use sara_core::validation::{Severity, ValidationIssue, ValidationReport};
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

/// Items a change may impact.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ImpactView {
    /// The items about to change.
    pub origins: Vec<ItemSummary>,
    /// Number of impacted items, including those beyond the limit.
    pub total: usize,
    /// Number of impacted items by item type id, over all of them.
    pub by_type: BTreeMap<String, usize>,
    /// Whether the depth limit or the item limit left items out.
    pub truncated: bool,
    /// Impacted items, nearest first.
    pub items: Vec<ImpactedItem>,
}

impl ImpactView {
    /// Summarizes `reach`, the items depending on `origins`, keeping the
    /// first `limit` of them.
    pub fn new(
        graph: &KnowledgeGraph,
        origins: Vec<ItemSummary>,
        mut reach: Reach,
        limit: usize,
    ) -> Self {
        let total = reach.items.len();
        let mut by_type = BTreeMap::new();
        for reached in &reach.items {
            *by_type
                .entry(reached.item.item_type.as_str().to_string())
                .or_insert(0) += 1;
        }

        // Items come nearest first, so truncation keeps every item nearer
        // than a kept one, and links come from the nearest items, so the
        // first one leading to an item comes from the item that reached it.
        reach.truncate(limit);
        let Reach {
            items,
            links,
            truncated,
        } = reach;
        let items = items
            .into_iter()
            .filter_map(|reached| {
                let item = graph.get(&reached.item.id)?;
                let link = links.iter().find(|link| link.to == reached.item.id)?;
                Some(ImpactedItem {
                    file: item.source.full_path().display().to_string(),
                    from: link.from.clone(),
                    relation: link.relationship,
                    reached,
                })
            })
            .collect();

        Self {
            origins,
            total,
            by_type,
            truncated,
            items,
        }
    }
}

/// An item a change may impact, and how the change reaches it.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ImpactedItem {
    /// The impacted item and its distance from the changed items.
    #[serde(flatten)]
    pub reached: ReachedItem,
    /// Path of the Markdown file holding the item.
    pub file: String,
    /// Item through which the change reaches this one.
    #[schemars(with = "String")]
    pub from: ItemId,
    /// Relation linking `from` to this item, read from `from`.
    #[schemars(with = "String")]
    pub relation: RelationshipType,
}

/// Outcome of validating the whole graph.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ValidationView {
    /// Whether the graph passed: no error and, in strict mode, nothing
    /// skipped during the scan.
    pub valid: bool,
    /// Whether warnings counted as errors.
    pub strict: bool,
    /// Number of items checked.
    pub items_checked: usize,
    /// Number of relationships checked.
    pub relationships_checked: usize,
    /// Number of errors found.
    pub error_count: usize,
    /// Number of warnings found.
    pub warning_count: usize,
    /// Problems found, errors first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub issues: Vec<ValidationIssue>,
    /// Files and repositories skipped during the scan.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scan_warnings: Vec<ScanWarning>,
}

impl ValidationView {
    /// Summarizes a validation report and the paths skipped by the scan.
    pub fn new(report: ValidationReport, scan_warnings: Vec<ScanWarning>, strict: bool) -> Self {
        let valid = report.is_valid() && (!strict || scan_warnings.is_empty());
        let error_count = report.error_count();
        let warning_count = report.warning_count();
        let mut issues = report.issues;
        issues.sort_by_key(|issue| issue.severity != Severity::Error);

        Self {
            valid,
            strict,
            items_checked: report.items_checked,
            relationships_checked: report.relationships_checked,
            error_count,
            warning_count,
            issues,
            scan_warnings,
        }
    }
}

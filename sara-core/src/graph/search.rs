//! In-memory full-text search over the items of a knowledge graph.
//!
//! [`search`] scores every item against the words of a query. Each word
//! counts once, for the most significant place it appears in:
//!
//! | Where the word appears | Points |
//! |------------------------|--------|
//! | the item id            | 8      |
//! | the name               | 5      |
//! | the description        | 3      |
//! | another field          | 2      |
//! | the Markdown body      | 1      |
//!
//! A query equal to an item id adds 100 points to that item. Hits are ranked
//! by the number of query words they contain, then by points, then by id.
//! Matching ignores case and accepts partial words, so `command` also finds
//! `commands`.
//!
//! The graph does not hold Markdown bodies: the caller provides them, which
//! keeps this module free of file access. Items are scored in parallel.

use std::borrow::Cow;
use std::path::PathBuf;

use rayon::prelude::*;
use serde::{Serialize, Serializer};

use crate::graph::KnowledgeGraph;
use crate::model::{
    FIELD_DESCRIPTION, FIELD_ID, FIELD_NAME, Item, ItemSummary, ItemType, serialize_path,
};

/// Points for a query word found in the item id.
const ID_WEIGHT: u32 = 8;

/// Points for a query word found in the name.
const NAME_WEIGHT: u32 = 5;

/// Points for a query word found in the description.
const DESCRIPTION_WEIGHT: u32 = 3;

/// Points for a query word found in another field.
const FIELD_WEIGHT: u32 = 2;

/// Points for a query word found in the Markdown body.
const BODY_WEIGHT: u32 = 1;

/// Bonus for a query equal to the item id, which ranks that item first.
const EXACT_ID_BONUS: u32 = 100;

/// Characters kept on each side of the first match in a snippet.
const SNIPPET_RADIUS: usize = 80;

/// Marks text cut off at either end of a snippet.
const ELLIPSIS: char = '…';

/// Name of [`MatchedField::Body`].
const BODY_NAME: &str = "body";

/// Part of an item where a query word was found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatchedField {
    /// The item id.
    Id,
    /// The item name.
    Name,
    /// The item description.
    Description,
    /// A field declared by the item type, by name.
    Field(String),
    /// The Markdown body.
    Body,
}

impl MatchedField {
    /// Returns the name of the part: its frontmatter key, or `body`.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Id => FIELD_ID,
            Self::Name => FIELD_NAME,
            Self::Description => FIELD_DESCRIPTION,
            Self::Field(name) => name,
            Self::Body => BODY_NAME,
        }
    }
}

impl Serialize for MatchedField {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

/// An item matching a search query.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct SearchHit {
    /// The matching item.
    #[serde(flatten)]
    pub item: ItemSummary,
    /// Path of the Markdown file holding the item.
    #[serde(serialize_with = "serialize_path")]
    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    pub file: PathBuf,
    /// Number of distinct query words found in the item.
    #[serde(skip)]
    pub matched_terms: usize,
    /// Points earned by the item, as described in the module documentation.
    #[serde(skip)]
    pub score: u32,
    /// Where query words were found, most significant first: `id`, `name`,
    /// `description`, a field name or `body`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(feature = "schemars", schemars(with = "Vec<String>"))]
    pub matched_in: Vec<MatchedField>,
    /// Text around the first match in the description, a field or the body,
    /// when a query word was found there.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
}

/// Searches the items of `graph` for the words of `query`.
///
/// Only items whose type is in `types` are considered, or all items when
/// `types` is empty. `body_of` returns the Markdown body of an item, if any;
/// it may run on several threads at once, and is not called when the query
/// holds no word. A query without words lists every considered item, sorted
/// by id. Ranking and matching rules are described in the module
/// documentation.
#[must_use]
pub fn search(
    graph: &KnowledgeGraph,
    query: &str,
    types: &[ItemType],
    body_of: impl Fn(&Item) -> Option<String> + Sync,
) -> Vec<SearchHit> {
    let terms: Vec<String> = query
        .split(|c: char| !c.is_alphanumeric())
        .filter(|term| !term.is_empty())
        .map(str::to_lowercase)
        .collect();
    let query = query.trim().to_lowercase();

    let candidates: Vec<&Item> = graph
        .items()
        .filter(|item| types.is_empty() || types.contains(&item.item_type))
        .collect();
    let mut hits: Vec<SearchHit> = candidates
        .into_par_iter()
        .filter_map(|item| {
            if terms.is_empty() {
                Some(SearchHit {
                    item: ItemSummary::from(item),
                    file: item.source.full_path(),
                    matched_terms: 0,
                    score: 0,
                    matched_in: Vec::new(),
                    snippet: None,
                })
            } else {
                score_item(item, &terms, &query, &body_of)
            }
        })
        .collect();

    hits.sort_by(|a, b| {
        b.matched_terms
            .cmp(&a.matched_terms)
            .then(b.score.cmp(&a.score))
            .then_with(|| a.item.id.as_str().cmp(b.item.id.as_str()))
    });
    hits
}

/// A part of an item searched for the query words.
struct Part<'a> {
    /// Where the text lies in the item.
    field: MatchedField,
    /// Points for a query word found in the text.
    weight: u32,
    /// The text, as written.
    text: Cow<'a, str>,
    /// The text, lowercased for matching.
    lower: String,
    /// Whether a query word was found in the text.
    matched: bool,
}

impl<'a> Part<'a> {
    /// Creates a part holding `text`, not matched yet.
    fn new(field: MatchedField, weight: u32, text: impl Into<Cow<'a, str>>) -> Self {
        let text = text.into();
        let lower = text.to_lowercase();
        Self {
            field,
            weight,
            text,
            lower,
            matched: false,
        }
    }
}

/// Scores one item against the query words, or returns `None` when none of
/// them appears in it and the query is not its id.
fn score_item(
    item: &Item,
    terms: &[String],
    query: &str,
    body_of: &impl Fn(&Item) -> Option<String>,
) -> Option<SearchHit> {
    // Ordered from the most to the least significant part.
    let mut parts = vec![
        Part::new(MatchedField::Id, ID_WEIGHT, item.id.as_str()),
        Part::new(MatchedField::Name, NAME_WEIGHT, item.name.as_str()),
    ];
    if let Some(description) = &item.description {
        parts.push(Part::new(
            MatchedField::Description,
            DESCRIPTION_WEIGHT,
            description.as_str(),
        ));
    }
    for (name, value) in item.attributes.iter() {
        parts.push(Part::new(
            MatchedField::Field(name.clone()),
            FIELD_WEIGHT,
            value.to_string(),
        ));
    }
    if let Some(body) = body_of(item) {
        parts.push(Part::new(MatchedField::Body, BODY_WEIGHT, body));
    }

    let mut matched_terms = 0;
    let mut score = 0;
    for term in terms {
        let mut best = None;
        for part in &mut parts {
            if part.lower.contains(term.as_str()) {
                part.matched = true;
                best = best.max(Some(part.weight));
            }
        }
        if let Some(weight) = best {
            matched_terms += 1;
            score += weight;
        }
    }

    let exact_id = item.id.as_str().to_lowercase() == query;
    if exact_id {
        score += EXACT_ID_BONUS;
    } else if matched_terms == 0 {
        return None;
    }

    let snippet = parts
        .iter()
        .filter(|part| part.matched && !matches!(part.field, MatchedField::Id | MatchedField::Name))
        .find_map(|part| snippet(&part.text, terms));
    let matched_in = parts
        .into_iter()
        .filter(|part| part.matched)
        .map(|part| part.field)
        .collect();

    Some(SearchHit {
        item: ItemSummary::from(item),
        file: item.source.full_path(),
        matched_terms,
        score,
        matched_in,
        snippet,
    })
}

/// Returns the text around the first occurrence of any term in `text`, with
/// runs of whitespace collapsed, or `None` when no term occurs.
///
/// Keeps up to [`SNIPPET_RADIUS`] characters on each side of the match and
/// marks a cut-off end with an ellipsis.
fn snippet(text: &str, terms: &[String]) -> Option<String> {
    let chars: Vec<char> = text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .collect();

    // Lowercasing may turn one character into several: remember, for each
    // lowercased character, the character it came from.
    let mut lower = Vec::with_capacity(chars.len());
    let mut origin = Vec::with_capacity(chars.len());
    for (idx, ch) in chars.iter().enumerate() {
        for low in ch.to_lowercase() {
            lower.push(low);
            origin.push(idx);
        }
    }

    let start = terms
        .iter()
        .filter_map(|term| find_chars(&lower, term))
        .min()
        .map(|pos| origin[pos])?;
    let from = start.saturating_sub(SNIPPET_RADIUS);
    let to = (start + SNIPPET_RADIUS).min(chars.len());

    let mut snippet = String::new();
    if from > 0 {
        snippet.push(ELLIPSIS);
    }
    snippet.extend(&chars[from..to]);
    if to < chars.len() {
        snippet.push(ELLIPSIS);
    }
    Some(snippet)
}

/// Returns the position of the first occurrence of `needle` in `haystack`.
fn find_chars(haystack: &[char], needle: &str) -> Option<usize> {
    let needle: Vec<char> = needle.chars().collect();
    if needle.is_empty() {
        return None;
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle.as_slice())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    use crate::graph::KnowledgeGraphBuilder;
    use crate::model::FIELD_TYPE;
    use crate::schema::builtin;
    use crate::test_utils::create_test_item_with_name;

    /// Builds an item of `item_type` with the given name and description.
    fn item(id: &str, item_type: ItemType, name: &str, description: Option<&str>) -> Item {
        let mut item = create_test_item_with_name(id, item_type, name);
        item.description = description.map(str::to_string);
        item
    }

    /// Searches `items` with the given bodies, keyed by item id.
    fn run(
        items: Vec<Item>,
        bodies: &[(&str, &str)],
        query: &str,
        types: &[ItemType],
    ) -> Vec<String> {
        let graph = KnowledgeGraphBuilder::new()
            .add_items(items)
            .build()
            .unwrap();
        let bodies: HashMap<&str, &str> = bodies.iter().copied().collect();
        search(&graph, query, types, |item| {
            bodies.get(item.id.as_str()).map(|body| (*body).to_string())
        })
        .into_iter()
        .map(|hit| hit.item.id.as_str().to_string())
        .collect()
    }

    #[test]
    fn test_search_ranks_name_match_above_body_match() {
        let hits = run(
            vec![
                item("SOL-001", builtin::SOLUTION, "Hub", None),
                item("SOL-002", builtin::SOLUTION, "Latency budget", None),
            ],
            &[("SOL-001", "Keeps latency low.")],
            "latency",
            &[],
        );

        assert_eq!(hits, ["SOL-002", "SOL-001"]);
    }

    #[test]
    fn test_search_ranks_items_covering_more_terms_first() {
        let hits = run(
            vec![
                item("SOL-001", builtin::SOLUTION, "Command center", None),
                item("SOL-002", builtin::SOLUTION, "Hub", None),
            ],
            &[("SOL-002", "Every command meets its latency target.")],
            "command latency",
            &[],
        );

        assert_eq!(hits, ["SOL-002", "SOL-001"]);
    }

    #[test]
    fn test_search_puts_exact_id_first() {
        let hits = run(
            vec![
                item("SOL-001", builtin::SOLUTION, "Refers to SOL-002", None),
                item("SOL-002", builtin::SOLUTION, "Hub", None),
            ],
            &[],
            "sol-002",
            &[],
        );

        assert_eq!(hits, ["SOL-002", "SOL-001"]);
    }

    #[test]
    fn test_search_filters_by_type() {
        let hits = run(
            vec![
                item("SOL-001", builtin::SOLUTION, "Lighting", None),
                item("UC-001", builtin::USE_CASE, "Lighting control", None),
            ],
            &[],
            "lighting",
            &[builtin::USE_CASE],
        );

        assert_eq!(hits, ["UC-001"]);
    }

    #[test]
    fn test_search_without_terms_lists_items_by_id() {
        let hits = run(
            vec![
                item("UC-001", builtin::USE_CASE, "Lighting control", None),
                item("SOL-001", builtin::SOLUTION, "Hub", None),
            ],
            &[],
            "  ",
            &[],
        );

        assert_eq!(hits, ["SOL-001", "UC-001"]);
    }

    #[test]
    fn test_search_hit_serializes_what_clients_need() {
        let graph = KnowledgeGraphBuilder::new()
            .add_item(item("SOL-001", builtin::SOLUTION, "Hub", Some("Local hub")))
            .build()
            .unwrap();

        let hits = search(&graph, "local", &[], |_| None);
        let value = serde_yaml::to_value(&hits[0]).unwrap();

        let keys: Vec<&str> = value
            .as_mapping()
            .unwrap()
            .keys()
            .map(|key| key.as_str().unwrap())
            .collect();
        assert_eq!(
            keys,
            [
                FIELD_ID,
                FIELD_TYPE,
                FIELD_NAME,
                "file",
                "matched_in",
                "snippet"
            ]
        );
        assert_eq!(
            value["matched_in"],
            serde_yaml::Value::from(vec![FIELD_DESCRIPTION])
        );
    }

    #[test]
    fn test_search_reports_where_terms_matched() {
        let graph = KnowledgeGraphBuilder::new()
            .add_item(item("SOL-001", builtin::SOLUTION, "Hub", Some("Local hub")))
            .build()
            .unwrap();

        let hits = search(&graph, "hub", &[], |_| {
            Some("The hub relays commands.".to_string())
        });

        assert_eq!(
            hits[0].matched_in,
            [
                MatchedField::Name,
                MatchedField::Description,
                MatchedField::Body
            ]
        );
    }

    #[test]
    fn test_search_snippet_surrounds_first_match() {
        let padding = "word ".repeat(40);
        let description = format!("{padding}the latency target{padding}");
        let graph = KnowledgeGraphBuilder::new()
            .add_item(item(
                "SOL-001",
                builtin::SOLUTION,
                "Hub",
                Some(&description),
            ))
            .build()
            .unwrap();

        let hits = search(&graph, "latency", &[], |_| None);
        let snippet = hits[0].snippet.as_deref().unwrap();

        assert!(
            snippet.starts_with('…') && snippet.ends_with('…'),
            "{snippet}"
        );
        assert!(snippet.contains("the latency target"), "{snippet}");
        assert!(
            snippet.chars().count() <= 2 * SNIPPET_RADIUS + 2,
            "{snippet}"
        );
    }

    #[test]
    fn test_search_ignores_case_of_accented_letters() {
        let graph = KnowledgeGraphBuilder::new()
            .add_item(item(
                "SOL-001",
                builtin::SOLUTION,
                "Hub",
                Some("Sécurité du réseau"),
            ))
            .build()
            .unwrap();

        let hits = search(&graph, "SÉCURITÉ", &[], |_| None);

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].snippet.as_deref(), Some("Sécurité du réseau"));
    }
}

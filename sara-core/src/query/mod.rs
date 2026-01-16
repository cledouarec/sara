//! Query operations for the knowledge graph.

mod traceability;

pub use traceability::{
    LookupResult, MissingParentError, QueryEngine, check_parent_exists, find_similar_ids,
    get_children, get_parents, lookup_item_or_suggest,
};

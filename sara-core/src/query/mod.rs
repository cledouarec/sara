//! Query operations for the knowledge graph.
//!
//! This module provides high-level query capabilities for exploring the
//! traceability graph and looking up items.
//!
//! # Key Types
//!
//! - [`QueryEngine`]: Main entry point for queries, wraps a `KnowledgeGraph`
//! - [`LookupResult`]: Result of item lookup (found or not found with suggestions)
//!
//! # Operations
//!
//! - Item lookup with fuzzy matching suggestions
//! - Parent/child relationship queries
//! - Upstream/downstream traceability chain traversal

mod traceability;

pub use traceability::{
    LookupResult, MissingParentError, QueryEngine, check_parent_exists, find_similar_ids,
    get_children, get_parents, lookup_item_or_suggest,
};

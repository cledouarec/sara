//! Knowledge graph operations for traceability item management.
//!
//! This module provides the core data structure for storing and navigating
//! traceability items and their relationships.
//!
//! # Key Types
//!
//! - [`KnowledgeGraph`]: The main graph data structure storing items and relationships
//! - [`GraphBuilder`]: Fluent builder for constructing graphs from parsed items
//! - [`GraphStats`]: Statistics about graph contents (item counts, relationship counts)
//!
//! # Traversal
//!
//! The [`traversal`] submodule provides functions for navigating the graph:
//! - Upstream traversal: Follow references toward Solution items
//! - Downstream traversal: Follow references toward Detailed Design items
//!
//! # Diffing
//!
//! The [`diff`] submodule compares two graphs to identify added, removed,
//! and modified items.

mod builder;
pub mod diff;
mod knowledge_graph;
mod stats;
pub mod traversal;

pub use builder::{GraphBuilder, resolve_cross_repository_refs};
pub use diff::{DiffStats, GraphDiff, ItemDiff, ItemModification, RelationshipDiff};
pub use knowledge_graph::KnowledgeGraph;
pub use stats::GraphStats;
pub use traversal::{
    TraversalDirection, TraversalNode, TraversalOptions, TraversalResult, TraversalTree,
    TraversalTreeNode, get_downstream_children, get_upstream_parents, traverse_downstream,
    traverse_upstream,
};

//! Knowledge graph operations.

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

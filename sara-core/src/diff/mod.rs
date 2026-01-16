//! Diff service for comparing knowledge graphs.
//!
//! Provides functionality to compute differences between two states of the
//! requirements knowledge graph, supporting Git reference comparisons.

mod options;
mod service;

pub use options::DiffOptions;
pub use service::{DiffError, DiffResult, DiffService};

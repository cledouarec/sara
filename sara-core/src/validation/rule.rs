//! Validation rule trait definition.

use serde::Serialize;

use crate::config::ValidationConfig;
use crate::error::ValidationError;
use crate::graph::KnowledgeGraph;
use crate::model::Item;

/// Severity level for validation issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Validation error that blocks acceptance.
    Error,
    /// Warning that doesn't block but should be addressed.
    Warning,
}

/// Trait for validation rules.
///
/// All validation rules must implement this trait. The validator orchestrates
/// running all rules and collecting results into a report.
pub trait ValidationRule: Send + Sync {
    /// Pre-validates a list of items before they are added to the graph.
    ///
    /// This enables fail-fast validation during parsing/loading. Rules that
    /// only need individual items (not relationships to other items) should
    /// implement this method.
    ///
    /// Rules that require the full graph context (e.g., cycle detection,
    /// broken references) should return an empty vector.
    fn pre_validate(&self, _items: &[Item], _config: &ValidationConfig) -> Vec<ValidationError> {
        Vec::new()
    }

    /// Validates the complete knowledge graph and returns any issues found.
    ///
    /// This method is called after the graph is fully built, providing access
    /// to all items and their relationships. Use this for validations that
    /// require graph context, such as:
    /// - Broken reference detection (target items must exist)
    /// - Cycle detection (requires full graph traversal)
    /// - Relationship type validation (needs both source and target types)
    /// - Orphan detection (requires parent relationship info)
    ///
    /// For item-level validations that don't need graph context, prefer
    /// implementing [`pre_validate`](Self::pre_validate) for fail-fast behavior.
    fn validate(
        &self,
        _graph: &KnowledgeGraph,
        _config: &ValidationConfig,
    ) -> Vec<ValidationError> {
        Vec::new()
    }

    /// The severity of issues produced by this rule.
    fn severity(&self) -> Severity {
        Severity::Error
    }
}

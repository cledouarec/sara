//! Validation rule trait definition.

use serde::Serialize;

use crate::config::ValidationConfig;
use crate::error::ValidationError;
use crate::graph::KnowledgeGraph;

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
    /// Validates the knowledge graph and returns any issues found.
    fn validate(&self, graph: &KnowledgeGraph, config: &ValidationConfig) -> Vec<ValidationError>;

    /// The severity of issues produced by this rule.
    fn severity(&self) -> Severity {
        Severity::Error
    }
}

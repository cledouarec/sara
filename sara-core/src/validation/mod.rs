//! Validation engine and rules for the knowledge graph.
//!
//! The validation system uses trait-based rules orchestrated by the [`Validator`].
//! External code should use [`validate`] or [`validate_strict`] functions.

mod report;
mod rule;
mod rules;
mod validator;

pub use report::{ValidationIssue, ValidationReport};
pub use rule::Severity;
pub use validator::{validate, validate_strict};

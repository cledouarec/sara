//! Validation engine and rules for the knowledge graph.

pub mod report;
pub mod rules;
pub mod validator;

pub use report::{Severity, ValidationIssue, ValidationReport, ValidationReportBuilder};
pub use validator::{Validator, validate, validate_strict};

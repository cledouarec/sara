//! Validation report structure.

use std::time::Duration;

use serde::Serialize;

use crate::error::ValidationError;

use super::rule::Severity;

/// A validation issue with its severity.
#[derive(Debug, Clone, Serialize)]
pub struct ValidationIssue {
    /// Severity of the issue.
    pub severity: Severity,
    /// The underlying validation error.
    pub error: ValidationError,
}

impl ValidationIssue {
    /// Creates a new error-level issue.
    pub fn error(error: ValidationError) -> Self {
        Self {
            severity: Severity::Error,
            error,
        }
    }

    /// Creates a new warning-level issue.
    pub fn warning(error: ValidationError) -> Self {
        Self {
            severity: Severity::Warning,
            error,
        }
    }
}

/// Validation report containing all issues found.
#[derive(Debug, Clone, Serialize)]
pub struct ValidationReport {
    /// All validation issues found.
    pub issues: Vec<ValidationIssue>,
    /// Number of items checked.
    pub items_checked: usize,
    /// Number of relationships checked.
    pub relationships_checked: usize,
    /// Time taken to perform validation.
    #[serde(skip)]
    pub duration: Duration,
}

impl ValidationReport {
    /// Creates a new empty validation report.
    pub fn new() -> Self {
        Self {
            issues: Vec::new(),
            items_checked: 0,
            relationships_checked: 0,
            duration: Duration::ZERO,
        }
    }

    /// Returns true if validation passed (no errors).
    pub fn is_valid(&self) -> bool {
        !self.issues.iter().any(|i| i.severity == Severity::Error)
    }

    /// Returns the number of errors.
    pub fn error_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == Severity::Error)
            .count()
    }

    /// Returns the number of warnings.
    pub fn warning_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == Severity::Warning)
            .count()
    }

    /// Returns all errors.
    pub fn errors(&self) -> Vec<&ValidationError> {
        self.issues
            .iter()
            .filter(|i| i.severity == Severity::Error)
            .map(|i| &i.error)
            .collect()
    }

    /// Returns all warnings.
    pub fn warnings(&self) -> Vec<&ValidationError> {
        self.issues
            .iter()
            .filter(|i| i.severity == Severity::Warning)
            .map(|i| &i.error)
            .collect()
    }
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating validation reports.
#[derive(Debug, Default)]
pub struct ValidationReportBuilder {
    report: ValidationReport,
}

impl ValidationReportBuilder {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the number of items checked.
    pub fn items_checked(mut self, count: usize) -> Self {
        self.report.items_checked = count;
        self
    }

    /// Sets the number of relationships checked.
    pub fn relationships_checked(mut self, count: usize) -> Self {
        self.report.relationships_checked = count;
        self
    }

    /// Sets the duration.
    pub fn duration(mut self, duration: Duration) -> Self {
        self.report.duration = duration;
        self
    }

    /// Adds errors.
    pub fn errors(mut self, errors: impl IntoIterator<Item = ValidationError>) -> Self {
        for error in errors {
            self.report.issues.push(ValidationIssue::error(error));
        }
        self
    }

    /// Adds multiple warnings.
    pub fn warnings(mut self, warnings: impl IntoIterator<Item = ValidationError>) -> Self {
        for warning in warnings {
            self.report.issues.push(ValidationIssue::warning(warning));
        }
        self
    }

    /// Builds the report.
    pub fn build(self) -> ValidationReport {
        self.report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ItemId;

    #[test]
    fn test_empty_report_is_valid() {
        let report = ValidationReport::new();
        assert!(report.is_valid());
        assert_eq!(report.error_count(), 0);
        assert_eq!(report.warning_count(), 0);
    }

    #[test]
    fn test_report_with_error() {
        let report = ValidationReportBuilder::new()
            .errors([ValidationError::BrokenReference {
                from: ItemId::new_unchecked("A"),
                to: ItemId::new_unchecked("B"),
            }])
            .build();

        assert!(!report.is_valid());
        assert_eq!(report.error_count(), 1);
        assert_eq!(report.warning_count(), 0);
    }

    #[test]
    fn test_report_with_warning() {
        let report = ValidationReportBuilder::new()
            .warnings([ValidationError::OrphanItem {
                id: ItemId::new_unchecked("A"),
                item_type: crate::model::ItemType::UseCase,
            }])
            .build();

        assert!(report.is_valid());
        assert_eq!(report.error_count(), 0);
        assert_eq!(report.warning_count(), 1);
    }

    #[test]
    fn test_builder() {
        let report = ValidationReportBuilder::new()
            .items_checked(10)
            .relationships_checked(15)
            .errors([ValidationError::BrokenReference {
                from: ItemId::new_unchecked("A"),
                to: ItemId::new_unchecked("B"),
            }])
            .build();

        assert_eq!(report.items_checked, 10);
        assert_eq!(report.relationships_checked, 15);
        assert_eq!(report.error_count(), 1);
    }
}

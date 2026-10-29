//! Validation pipeline shared by every caller that checks a set of items.
//!
//! [`check_items`] runs the two validation passes of `sara check` over
//! items already parsed, whether they come from the working tree or from a
//! Git reference. Scanning the repositories and deciding how to treat the
//! paths skipped during the scan stay with the caller.

use crate::error::SaraError;
use crate::graph::{KnowledgeGraph, KnowledgeGraphBuilder};
use crate::model::Item;
use crate::validation::{ValidationReport, pre_validate, validate};

/// Result of checking a set of items.
#[derive(Debug)]
pub struct CheckOutcome {
    /// Issues and counters of the validation passes.
    pub report: ValidationReport,
    /// The knowledge graph, built only when pre-validation passed.
    pub graph: Option<KnowledgeGraph>,
}

/// Checks parsed items the way `sara check` does.
///
/// Pre-validation runs on the items first: it catches problems the graph
/// cannot represent, such as two items sharing an identifier. Only when it
/// passes is the graph built and validated; both reports are then merged.
///
/// # Errors
///
/// Returns an error when the knowledge graph cannot be built.
pub fn check_items(items: Vec<Item>, strict: bool) -> Result<CheckOutcome, SaraError> {
    let pre_report = pre_validate(&items, strict);
    if !pre_report.is_valid() {
        return Ok(CheckOutcome {
            report: pre_report,
            graph: None,
        });
    }

    let graph = KnowledgeGraphBuilder::new().add_items(items).build()?;
    let mut report = validate(&graph, strict);
    report.merge(pre_report);

    Ok(CheckOutcome {
        report,
        graph: Some(graph),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::error::SaraError;
    use crate::model::{ItemId, Relationship};
    use crate::schema::builtin;
    use crate::test_utils::{create_test_item, create_test_item_with_relationships};

    #[test]
    fn test_check_items_reports_duplicate_ids_without_graph() {
        let items = vec![
            create_test_item("SOL-001", builtin::SOLUTION),
            create_test_item("SOL-001", builtin::SOLUTION),
        ];

        let outcome = check_items(items, false).unwrap();

        assert!(outcome.graph.is_none());
        assert!(
            outcome
                .report
                .errors()
                .iter()
                .any(|error| matches!(error, SaraError::DuplicateIdentifier { .. }))
        );
    }

    #[test]
    fn test_check_items_builds_graph_when_items_are_valid() {
        let items = vec![
            create_test_item("SOL-001", builtin::SOLUTION),
            create_test_item_with_relationships(
                "UC-001",
                builtin::USE_CASE,
                vec![Relationship::new(
                    ItemId::new_unchecked("SOL-001"),
                    builtin::REFINES,
                )],
            ),
        ];

        let outcome = check_items(items, false).unwrap();

        assert!(outcome.report.is_valid());
        assert_eq!(outcome.report.items_checked, 2);
        assert_eq!(outcome.graph.map(|graph| graph.item_count()), Some(2));
    }
}

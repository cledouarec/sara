//! Identifier format conformance rule.

use crate::config::ValidationConfig;
use crate::error::SaraError;
use crate::graph::KnowledgeGraph;
use crate::model::Item;
use crate::schema::{self, IdFormat};
use crate::validation::rule::{Severity, ValidationRule};

/// Identifier format conformance rule.
///
/// Warns when an item's id does not match the `id_format` its type declares
/// in the active schema. Temporal and unique placeholders are matched by
/// shape, so an id generated under a previous period (or any UUID) stays
/// conformant.
///
/// Default severity is Warning, but in strict mode all warnings become errors.
pub struct IdFormatRule;

impl ValidationRule for IdFormatRule {
    fn validate(&self, graph: &KnowledgeGraph, _config: &ValidationConfig) -> Vec<SaraError> {
        graph.items().filter_map(check_item).collect()
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }
}

/// Checks one item's id against its type's format.
fn check_item(item: &Item) -> Option<SaraError> {
    let def = schema::item_type_def(item.item_type.as_str())?;
    let format = IdFormat::parse(&def.id_format).ok()?;
    if format.matches(item.id.as_str(), &def.prefix, &def.id) {
        return None;
    }
    Some(SaraError::InvalidId {
        id: item.id.as_str().to_string(),
        reason: format!(
            "does not match the id_format \"{}\" of type {}",
            def.id_format, def.display_name
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::graph::KnowledgeGraphBuilder;
    use crate::schema::builtin;
    use crate::test_utils::create_test_item;

    #[test]
    fn test_conforming_and_unpadded_ids_pass() {
        let graph = KnowledgeGraphBuilder::new()
            .add_item(create_test_item("SOL-001", builtin::SOLUTION))
            .add_item(create_test_item("SOL-7", builtin::SOLUTION))
            .build()
            .unwrap();

        let rule = IdFormatRule;
        assert!(
            rule.validate(&graph, &ValidationConfig::default())
                .is_empty()
        );
    }

    #[test]
    fn test_hand_written_id_is_flagged() {
        let graph = KnowledgeGraphBuilder::new()
            .add_item(create_test_item("SOL-LOGIN", builtin::SOLUTION))
            .build()
            .unwrap();

        let rule = IdFormatRule;
        let errors = rule.validate(&graph, &ValidationConfig::default());
        assert_eq!(errors.len(), 1);
        let message = errors[0].to_string();
        assert!(message.contains("SOL-LOGIN"), "got: {message}");
        assert!(
            message.contains("does not match the id_format \"{prefix}-{seq:03}\" of type Solution"),
            "got: {message}"
        );
    }

    #[test]
    fn test_severity_is_warning() {
        assert_eq!(IdFormatRule.severity(), Severity::Warning);
    }
}

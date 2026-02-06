//! Duplicate identifier detection validation rule.

use std::collections::HashMap;

use crate::config::ValidationConfig;
use crate::error::SaraError;
use crate::model::Item;
use crate::validation::rule::ValidationRule;

/// Duplicate identifier detection rule.
///
/// Each item ID must be unique across all repositories.
/// This rule only implements pre-validation since the graph itself prevents
/// duplicates by using a HashMap. Pre-validation catches duplicates before
/// items are added to the graph.
pub struct DuplicatesRule;

impl ValidationRule for DuplicatesRule {
    fn pre_validate(&self, items: &[Item], _config: &ValidationConfig) -> Vec<SaraError> {
        // Count occurrences of each ID
        let mut id_counts: HashMap<&str, usize> = HashMap::new();

        for item in items {
            *id_counts.entry(item.id.as_str()).or_default() += 1;
        }

        // Report duplicates (IDs with more than one occurrence)
        id_counts
            .into_iter()
            .filter(|(_, count)| *count > 1)
            .map(|(id, _)| SaraError::DuplicateIdentifier {
                id: crate::model::ItemId::new_unchecked(id),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ItemType;
    use crate::test_utils::create_test_item_at;

    #[test]
    fn test_no_duplicates() {
        let items = vec![
            create_test_item_at("SOL-001", ItemType::Solution, "sol1.md"),
            create_test_item_at("SOL-002", ItemType::Solution, "sol2.md"),
        ];

        let rule = DuplicatesRule;
        let errors = rule.pre_validate(&items, &ValidationConfig::default());
        assert!(errors.is_empty());
    }

    #[test]
    fn test_duplicate_detected() {
        let items = vec![
            create_test_item_at("SOL-001", ItemType::Solution, "sol1.md"),
            create_test_item_at("SOL-001", ItemType::Solution, "sol2.md"), // Duplicate ID
        ];

        let rule = DuplicatesRule;
        let errors = rule.pre_validate(&items, &ValidationConfig::default());
        assert_eq!(errors.len(), 1);

        if let SaraError::DuplicateIdentifier { id } = &errors[0] {
            assert_eq!(id.as_str(), "SOL-001");
        } else {
            panic!("Expected DuplicateIdentifier error");
        }
    }

    #[test]
    fn test_multiple_duplicates_same_id() {
        let items = vec![
            create_test_item_at("SOL-001", ItemType::Solution, "sol1.md"),
            create_test_item_at("SOL-001", ItemType::Solution, "sol2.md"),
            create_test_item_at("SOL-001", ItemType::Solution, "sol3.md"),
        ];

        let rule = DuplicatesRule;
        let errors = rule.pre_validate(&items, &ValidationConfig::default());
        assert_eq!(errors.len(), 1, "Should be one error for one duplicate ID");
    }

    #[test]
    fn test_multiple_different_duplicates() {
        let items = vec![
            create_test_item_at("SOL-001", ItemType::Solution, "sol1.md"),
            create_test_item_at("SOL-001", ItemType::Solution, "sol2.md"), // Duplicate of SOL-001
            create_test_item_at("SOL-002", ItemType::Solution, "sol3.md"),
            create_test_item_at("SOL-002", ItemType::Solution, "sol4.md"), // Duplicate of SOL-002
        ];

        let rule = DuplicatesRule;
        let errors = rule.pre_validate(&items, &ValidationConfig::default());
        assert_eq!(errors.len(), 2, "Should detect two different duplicate IDs");
    }
}

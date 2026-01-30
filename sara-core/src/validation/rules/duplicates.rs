//! Duplicate identifier detection validation rule.

use std::collections::HashMap;

use crate::config::ValidationConfig;
use crate::error::ValidationError;
use crate::model::{Item, SourceLocation};
use crate::validation::rule::ValidationRule;

/// Duplicate identifier detection rule.
///
/// Each item ID must be unique across all repositories.
/// This rule only implements pre-validation since the graph itself prevents
/// duplicates by using a HashMap. Pre-validation catches duplicates before
/// items are added to the graph.
pub struct DuplicatesRule;

impl ValidationRule for DuplicatesRule {
    fn pre_validate(&self, items: &[Item], _config: &ValidationConfig) -> Vec<ValidationError> {
        // Group items by ID to find duplicates
        let mut id_locations: HashMap<&str, Vec<SourceLocation>> = HashMap::new();

        for item in items {
            id_locations
                .entry(item.id.as_str())
                .or_default()
                .push(item.source.clone());
        }

        // Report duplicates (IDs with more than one location)
        id_locations
            .into_iter()
            .filter(|(_, locations)| locations.len() > 1)
            .map(|(id, locations)| ValidationError::DuplicateIdentifier {
                id: crate::model::ItemId::new_unchecked(id),
                locations,
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

        if let ValidationError::DuplicateIdentifier { id, locations } = &errors[0] {
            assert_eq!(id.as_str(), "SOL-001");
            assert_eq!(locations.len(), 2);
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

        if let ValidationError::DuplicateIdentifier { locations, .. } = &errors[0] {
            assert_eq!(locations.len(), 3, "Should have all three locations");
        } else {
            panic!("Expected DuplicateIdentifier error");
        }
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

//! Application services for file I/O operations.
//!
//! This module provides stateless service functions that combine domain logic
//! with file I/O operations. These functions bridge the gap between the pure
//! domain layer (`model/`) and the CLI/application layer.

use crate::model::{ItemId, Relationship, RelationshipType};

pub mod diff;
pub mod edit;
pub mod init;

/// Converts string IDs into [`Relationship`] values of the given type.
fn ids_to_relationships(ids: &[String], rel_type: RelationshipType) -> Vec<Relationship> {
    ids.iter()
        .map(|id| Relationship::new(ItemId::new_unchecked(id), rel_type))
        .collect()
}

// Diff service exports
pub use diff::{DiffError, DiffOptions, DiffResult, DiffService};
// Edit service exports
pub use edit::{EditOptions, EditResult, EditService, EditedValues, ItemContext};
// Init service exports
pub use init::{InitError, InitOptions, InitResult, InitService, TypeConfig, parse_item_type};

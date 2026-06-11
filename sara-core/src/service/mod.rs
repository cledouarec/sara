//! Application services for file I/O operations.
//!
//! This module provides stateless service functions that combine domain logic
//! with file I/O operations. These functions bridge the gap between the pure
//! domain layer (`model/`) and the CLI/application layer.

use std::path::PathBuf;

use crate::error::SaraError;
use crate::graph::{KnowledgeGraph, KnowledgeGraphBuilder};
use crate::model::{ItemId, Relationship, RelationshipType};
use crate::repository::parse_repositories;

pub mod diff;
pub mod edit;
pub mod init;

/// Parses the given repository paths and builds the knowledge graph from
/// every item found.
pub fn load_graph(paths: &[PathBuf]) -> Result<KnowledgeGraph, SaraError> {
    let items = parse_repositories(paths)?;
    KnowledgeGraphBuilder::new().add_items(items).build()
}

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
pub use init::{
    FieldInput, InitError, InitOptions, InitResult, InitService, TypeConfig, parse_item_type,
};

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_load_graph_from_repository() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("solution.md"),
            r#"---
id: "SOL-001"
type: solution
name: "Test Solution"
---
# Solution
"#,
        )
        .unwrap();

        let graph = load_graph(&[temp_dir.path().to_path_buf()]).unwrap();

        assert_eq!(graph.item_count(), 1);
    }
}

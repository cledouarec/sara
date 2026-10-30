//! Application services for file I/O operations.
//!
//! This module provides stateless service functions that combine domain logic
//! with file I/O operations. These functions bridge the gap between the pure
//! domain layer (`model/`) and the CLI/application layer.

use std::path::PathBuf;
use std::{fs, io};

use crate::error::SaraError;
use crate::graph::{KnowledgeGraph, KnowledgeGraphBuilder};
use crate::model::{Item, ItemId, Relationship, RelationshipType};
use crate::parser::extract_body;
use crate::repository::{ScanWarning, parse_repositories};

pub mod check;
pub mod diff;
pub mod edit;
pub mod init;

/// Parses the given repository paths and builds the knowledge graph from
/// every item found.
///
/// Warnings for paths skipped during the scan are returned alongside the
/// graph so callers can report them.
pub fn load_graph(paths: &[PathBuf]) -> Result<(KnowledgeGraph, Vec<ScanWarning>), SaraError> {
    let scan = parse_repositories(paths);
    let graph = KnowledgeGraphBuilder::new().add_items(scan.items).build()?;
    Ok((graph, scan.warnings))
}

/// Reads the Markdown body of an item from its source file.
///
/// The body is everything after the frontmatter block, read from the file
/// the item was parsed from, so it reflects the file as it is now.
///
/// # Errors
///
/// Returns [`SaraError::Io`], naming the file, when it cannot be read.
pub fn read_body(item: &Item) -> Result<String, SaraError> {
    let path = item.source.full_path();
    let content = fs::read_to_string(&path)
        .map_err(|e| io::Error::new(e.kind(), format!("{}: {e}", path.display())))?;
    Ok(extract_body(&content))
}

/// Converts string IDs into [`Relationship`] values of the given type.
fn ids_to_relationships(ids: &[String], rel_type: RelationshipType) -> Vec<Relationship> {
    ids.iter()
        .map(|id| Relationship::new(ItemId::new_unchecked(id), rel_type))
        .collect()
}

// Check service exports
pub use check::{CheckOutcome, check_items};
// Diff service exports
pub use diff::{DiffError, DiffOptions, DiffResult, DiffService};
// Edit service exports
pub use edit::{EditOptions, EditResult, EditService, EditedValues, ItemContext, KeptLink};
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

        let (graph, warnings) = load_graph(&[temp_dir.path().to_path_buf()]).unwrap();

        assert_eq!(graph.item_count(), 1);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_read_body_returns_content_after_frontmatter() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("solution.md"),
            "---\nid: \"SOL-001\"\ntype: solution\nname: \"Test\"\n---\n# Solution\n\nBody text.\n",
        )
        .unwrap();
        let (graph, _) = load_graph(&[temp_dir.path().to_path_buf()]).unwrap();
        let item = graph.get(&ItemId::new_unchecked("SOL-001")).unwrap();

        assert_eq!(read_body(item).unwrap(), "# Solution\n\nBody text.");
    }

    #[test]
    fn test_read_body_error_names_the_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let file = temp_dir.path().join("solution.md");
        fs::write(
            &file,
            "---\nid: \"SOL-001\"\ntype: solution\nname: \"Test\"\n---\n",
        )
        .unwrap();
        let (graph, _) = load_graph(&[temp_dir.path().to_path_buf()]).unwrap();
        fs::remove_file(&file).unwrap();
        let item = graph.get(&ItemId::new_unchecked("SOL-001")).unwrap();

        let error = read_body(item).unwrap_err().to_string();

        assert!(error.contains("solution.md"), "{error}");
    }
}

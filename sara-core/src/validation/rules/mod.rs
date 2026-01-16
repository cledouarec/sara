//! Individual validation rules for the knowledge graph.

mod broken_refs;
mod cycles;
mod duplicates;
mod metadata;
mod orphans;
mod redundant;
mod relationships;

pub use broken_refs::{check_broken_references, find_referencing_items};
pub use cycles::{check_cycles, would_create_cycle};
pub use duplicates::{check_duplicate_items, check_duplicates, would_be_duplicate};
pub use metadata::{check_custom_fields, check_metadata, known_fields, validate_specification};
pub use orphans::{check_orphans, is_orphan_error};
pub use redundant::check_redundant_relationships;
pub use relationships::check_relationships;

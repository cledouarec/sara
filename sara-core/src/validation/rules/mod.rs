//! Individual validation rules for the knowledge graph.
//!
//! Each rule is a struct implementing the [`ValidationRule`](super::rule::ValidationRule) trait.
//! The validator orchestrates running all rules and collecting results.

mod broken_refs;
mod cycles;
mod duplicates;
mod metadata;
mod orphans;
mod redundant;
mod relationships;

// Export rule structs for the validator
pub use broken_refs::BrokenReferencesRule;
pub use cycles::CyclesRule;
pub use duplicates::DuplicatesRule;
pub use metadata::MetadataRule;
pub use orphans::OrphansRule;
pub use redundant::RedundantRelationshipsRule;
pub use relationships::RelationshipsRule;

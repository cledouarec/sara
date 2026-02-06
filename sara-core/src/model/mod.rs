//! Domain model entities for the knowledge graph.

mod edit;
mod field;
mod item;
mod metadata;
mod relationship;

pub use edit::{EditSummary, EditUpdates, FieldChange, TraceabilityLinks};
pub use field::FieldName;
pub use item::{
    AdrStatus, Item, ItemAttributes, ItemBuilder, ItemId, ItemType, TraceabilityConfig,
};
pub use metadata::SourceLocation;
pub use relationship::{Relationship, RelationshipRules, RelationshipType};

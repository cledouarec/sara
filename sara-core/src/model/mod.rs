//! Domain model entities for the knowledge graph.

mod builder;
mod edit;
mod field;
mod item;
mod metadata;
mod relationship;

pub use builder::ItemBuilder;
pub use edit::{EditSummary, FieldChange, TraceabilityLinks};
pub use field::FieldValue;
pub use item::{
    FIELD_DESCRIPTION, FIELD_ID, FIELD_NAME, FIELD_TYPE, Item, ItemAttributes, ItemId, ItemType,
    TraceabilityConfig,
};
pub use metadata::SourceLocation;
pub use relationship::{Relationship, RelationshipRules, RelationshipType};

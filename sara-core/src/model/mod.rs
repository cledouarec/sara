//! Domain model entities for the knowledge graph.

mod edit;
mod field;
mod item;
mod metadata;
mod relationship;

pub use edit::{EditSummary, EditUpdates, FieldChange, TraceabilityLinks};
pub use field::FieldName;
pub use item::{
    DownstreamRefs, Item, ItemAttributes, ItemBuilder, ItemId, ItemType, TraceabilityConfig,
    UpstreamRefs,
};
pub use metadata::SourceLocation;
pub use relationship::{Relationship, RelationshipRules, RelationshipType};

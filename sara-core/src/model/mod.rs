//! Domain model entities for the knowledge graph.

mod adr;
mod builder;
mod edit;
mod field;
mod item;
mod metadata;
mod relationship;

pub use adr::AdrStatus;
pub use builder::ItemBuilder;
pub use edit::{EditSummary, EditUpdates, FieldChange, TraceabilityLinks};
pub use field::FieldName;
pub use item::{Item, ItemAttributes, ItemId, ItemType, TraceabilityConfig};
pub use metadata::SourceLocation;
pub use relationship::{Relationship, RelationshipRules, RelationshipType};

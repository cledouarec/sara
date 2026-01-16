//! Edit service for modifying existing document metadata.
//!
//! Provides functionality for editing requirement items (FR-054 through FR-066).

mod options;
mod service;

pub use options::{EditOptions, EditedValues};
pub use service::{EditResult, EditService, ItemContext};

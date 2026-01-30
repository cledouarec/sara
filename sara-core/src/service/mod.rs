//! Service layer for file I/O operations.
//!
//! This module provides stateless service functions that combine domain logic
//! with file I/O operations. These functions bridge the gap between the pure
//! domain layer (model/) and the CLI/application layer.

mod diff;
mod edit;
mod init;

// Diff service exports
pub use diff::{DiffError, DiffOptions, DiffResult, diff};

// Edit service exports
pub use edit::{
    EditOptions, EditResult, EditedValues, ItemContext, apply_changes, build_change_summary,
    edit_item, get_item_for_edit,
};

// Init service exports
pub use init::{InitError, InitFileOptions, InitResult, create_item, suggest_next_id};

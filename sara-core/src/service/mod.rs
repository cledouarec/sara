//! Application services for file I/O operations.
//!
//! This module provides stateless service functions that combine domain logic
//! with file I/O operations. These functions bridge the gap between the pure
//! domain layer (`model/`) and the CLI/application layer.

pub mod diff;
pub mod edit;
pub mod init;

// Diff service exports
pub use diff::{DiffError, DiffOptions, DiffResult, DiffService};

// Edit service exports
pub use edit::{EditOptions, EditResult, EditService, EditedValues, ItemContext};

// Init service exports
pub use init::{InitError, InitOptions, InitResult, InitService, TypeConfig, parse_item_type};

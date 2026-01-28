//! Service layer for file I/O operations.
//!
//! This module provides stateless service functions that combine domain logic
//! with file I/O operations. These functions bridge the gap between the pure
//! domain layer (model/) and the CLI/application layer.

mod init;

pub use init::{InitError, InitFileOptions, InitResult, create_item, update_item};

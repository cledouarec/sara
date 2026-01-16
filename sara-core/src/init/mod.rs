//! Item initialization service.
//!
//! Provides functionality to initialize new requirement items or add frontmatter
//! to existing documents.

mod options;
pub mod service;

pub use options::InitOptions;
pub use service::{InitError, InitResult, InitService, parse_item_type};

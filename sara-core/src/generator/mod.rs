//! Output adapters for generating Markdown documents and YAML frontmatter.
//!
//! This module provides format-specific generation from domain types:
//! - `yaml` - YAML frontmatter generation
//! - `markdown` - Complete Markdown document generation

pub mod markdown;
pub mod yaml;

// Convenience re-exports
pub use markdown::generate_document;
pub use yaml::{generate_edit_frontmatter, generate_frontmatter};

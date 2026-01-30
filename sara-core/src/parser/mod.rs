//! Input adapters for parsing Markdown documents and YAML frontmatter.
//!
//! This module provides format-specific parsing into domain types:
//! - `frontmatter` - Frontmatter extraction from Markdown files
//! - `yaml` - YAML frontmatter parsing to domain types
//! - `markdown` - Complete Markdown document parsing

mod frontmatter;
mod markdown;
pub mod yaml;

pub use frontmatter::{
    ExtractedFrontmatter, extract_body, extract_frontmatter, has_frontmatter, update_frontmatter,
};
pub use markdown::{ParsedDocument, RawFrontmatter, parse_document, parse_markdown_file};
pub use yaml::parse_frontmatter as parse_yaml_frontmatter;

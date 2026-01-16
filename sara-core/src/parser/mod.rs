//! Markdown and metadata parsing.

mod frontmatter;
mod markdown;

pub use frontmatter::{
    ExtractedFrontmatter, extract_body, extract_frontmatter, has_frontmatter, update_frontmatter,
};
pub use markdown::{ParsedDocument, RawFrontmatter, parse_document, parse_markdown_file};

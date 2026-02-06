//! Markdown and metadata parsing.

mod frontmatter;
mod markdown;

pub use frontmatter::{
    ExtractedFrontmatter, extract_body, extract_frontmatter, has_frontmatter, update_frontmatter,
};
pub use markdown::{
    ParsedDocument, RawFrontmatter, extract_name_from_content, parse_document, parse_markdown_file,
};

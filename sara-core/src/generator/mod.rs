//! Output adapters for generating Markdown documents and YAML frontmatter.
//!
//! This module provides format-specific generation from domain types:
//! - `yaml` - YAML frontmatter generation
//! - `markdown` - Complete Markdown document generation

pub mod markdown;
pub mod yaml;

// Convenience re-exports
pub use markdown::generate_document;
pub use yaml::generate_frontmatter;

/// Extracts a name from a markdown file's first heading.
pub fn extract_name_from_content(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(heading) = trimmed.strip_prefix("# ") {
            return Some(heading.trim().to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_name_from_content() {
        let content = "# My Document\n\nSome content here.";
        assert_eq!(
            extract_name_from_content(content),
            Some("My Document".to_string())
        );

        let content_no_heading = "No heading here";
        assert_eq!(extract_name_from_content(content_no_heading), None);
    }
}

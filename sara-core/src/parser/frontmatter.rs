//! YAML frontmatter extraction from Markdown files.

use std::path::Path;

use crate::error::SaraError;

/// Line that opens and closes a frontmatter block.
const DELIMITER: &str = "---";

/// Returns the index of the closing delimiter line, searching after the
/// opening delimiter at index 0. Returns `None` when the block is unclosed.
fn closing_delimiter_index(lines: &[&str]) -> Option<usize> {
    lines
        .iter()
        .enumerate()
        .skip(1)
        .find(|(_, line)| line.trim() == DELIMITER)
        .map(|(i, _)| i)
}

/// Extracts YAML frontmatter from Markdown content.
///
/// Frontmatter must be at the start of the file, enclosed by `---` delimiters.
/// Returns the raw YAML string between the delimiters.
///
/// # Example
/// ```text
/// ---
/// id: "SOL-001"
/// type: solution
/// name: "My Solution"
/// ---
/// # Markdown content here
/// ```
pub fn extract_frontmatter(content: &str, file: &Path) -> Result<String, SaraError> {
    let lines: Vec<&str> = content.lines().collect();

    if lines.first().map(|line| line.trim()) != Some(DELIMITER) {
        return Err(SaraError::MissingFrontmatter {
            file: file.to_path_buf(),
        });
    }

    let end_idx = closing_delimiter_index(&lines).ok_or_else(|| SaraError::InvalidFrontmatter {
        file: file.to_path_buf(),
        reason: "Missing closing `---` delimiter".to_string(),
    })?;

    Ok(lines[1..end_idx].join("\n"))
}

/// Checks if content has frontmatter (starts with `---`).
pub fn has_frontmatter(content: &str) -> bool {
    content.trim_start().starts_with(DELIMITER)
}

/// Extracts just the body content after the frontmatter (FR-064).
///
/// Returns the body content without the frontmatter delimiters.
/// If no frontmatter is present, returns the original content.
pub fn extract_body(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();

    if lines.first().map(|line| line.trim()) != Some(DELIMITER) {
        return content.to_string();
    }

    match closing_delimiter_index(&lines) {
        Some(end_idx) => lines[end_idx + 1..].join("\n"),
        // No closing delimiter found, return original
        None => content.to_string(),
    }
}

/// Updates the YAML frontmatter in content while preserving the body (FR-064).
///
/// The `new_frontmatter` must include the `---` delimiters.
/// Returns the updated content with new frontmatter and preserved body.
pub fn update_frontmatter(content: &str, new_frontmatter: &str) -> String {
    let body = extract_body(content);
    let frontmatter_trimmed = new_frontmatter.trim_end();

    if body.is_empty() {
        format!("{}\n", frontmatter_trimmed)
    } else {
        format!("{}\n{}", frontmatter_trimmed, body)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_extract_frontmatter_valid() {
        let content = r#"---
id: "SOL-001"
type: solution
name: "Test"
---
# Body content"#;

        let yaml = extract_frontmatter(content, &PathBuf::from("test.md")).unwrap();
        assert!(yaml.contains("id: \"SOL-001\""));
        assert!(yaml.contains("type: solution"));
    }

    #[test]
    fn test_extract_frontmatter_no_body() {
        let content = r#"---
id: "SOL-001"
---"#;

        let yaml = extract_frontmatter(content, &PathBuf::from("test.md")).unwrap();
        assert!(yaml.contains("id: \"SOL-001\""));
    }

    #[test]
    fn test_extract_frontmatter_missing() {
        let content = "# Just markdown";
        let result = extract_frontmatter(content, &PathBuf::from("test.md"));
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_frontmatter_unclosed() {
        let content = r#"---
id: "SOL-001"
# No closing delimiter"#;

        let result = extract_frontmatter(content, &PathBuf::from("test.md"));
        assert!(result.is_err());
    }

    #[test]
    fn test_has_frontmatter() {
        assert!(has_frontmatter("---\nid: test\n---"));
        assert!(has_frontmatter("  ---\nid: test\n---"));
        assert!(!has_frontmatter("# No frontmatter"));
    }

    #[test]
    fn test_extract_frontmatter_empty() {
        let content = "";
        let result = extract_frontmatter(content, &PathBuf::from("test.md"));
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_body_with_frontmatter() {
        let content = r#"---
id: "SOL-001"
type: solution
---
# Body Content

Some markdown here."#;

        let body = extract_body(content);
        assert_eq!(body, "# Body Content\n\nSome markdown here.");
    }

    #[test]
    fn test_extract_body_no_frontmatter() {
        let content = "# Just markdown\n\nNo frontmatter here.";
        let body = extract_body(content);
        assert_eq!(body, content);
    }

    #[test]
    fn test_extract_body_empty_body() {
        let content = "---\nid: test\n---";
        let body = extract_body(content);
        assert!(body.is_empty());
    }

    #[test]
    fn test_update_frontmatter() {
        let content = r#"---
id: "SOL-001"
type: solution
name: "Old Name"
---
# Body Content

Some markdown here."#;

        let new_frontmatter = r#"---
id: "SOL-001"
type: solution
name: "New Name"
---"#;

        let updated = update_frontmatter(content, new_frontmatter);

        assert!(updated.starts_with("---\n"));
        assert!(updated.contains("name: \"New Name\""));
        assert!(updated.contains("# Body Content"));
        assert!(updated.contains("Some markdown here."));
    }

    #[test]
    fn test_update_frontmatter_no_body() {
        let content = "---\nid: test\n---";
        let new_frontmatter = "---\nid: test\nname: Updated\n---";

        let updated = update_frontmatter(content, new_frontmatter);

        assert_eq!(updated, "---\nid: test\nname: Updated\n---\n");
    }
}

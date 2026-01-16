//! YAML frontmatter extraction from Markdown files.

use std::path::Path;

use crate::error::ParseError;

/// Represents extracted frontmatter content.
#[derive(Debug, Clone)]
pub struct ExtractedFrontmatter {
    /// The raw YAML content between the `---` delimiters.
    pub yaml: String,
    /// Line number where the frontmatter starts (1-indexed, at the opening `---`).
    pub start_line: usize,
    /// Line number where the frontmatter ends (at the closing `---`).
    pub end_line: usize,
    /// The remaining Markdown content after the frontmatter.
    pub body: String,
}

/// Extracts YAML frontmatter from Markdown content.
///
/// Frontmatter must be at the start of the file, enclosed by `---` delimiters.
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
pub fn extract_frontmatter(content: &str, file: &Path) -> Result<ExtractedFrontmatter, ParseError> {
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() {
        return Err(ParseError::MissingFrontmatter {
            file: file.to_path_buf(),
        });
    }

    // Check for opening delimiter
    if lines[0].trim() != "---" {
        return Err(ParseError::MissingFrontmatter {
            file: file.to_path_buf(),
        });
    }

    // Find closing delimiter
    let mut end_idx = None;
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "---" {
            end_idx = Some(i);
            break;
        }
    }

    let end_idx = end_idx.ok_or_else(|| ParseError::InvalidFrontmatter {
        file: file.to_path_buf(),
        line: 1,
        reason: "Missing closing `---` delimiter".to_string(),
    })?;

    // Extract YAML content (lines between delimiters)
    let yaml_lines: Vec<&str> = lines[1..end_idx].to_vec();
    let yaml = yaml_lines.join("\n");

    // Extract body (everything after closing delimiter)
    let body_lines: Vec<&str> = if end_idx + 1 < lines.len() {
        lines[end_idx + 1..].to_vec()
    } else {
        Vec::new()
    };
    let body = body_lines.join("\n");

    Ok(ExtractedFrontmatter {
        yaml,
        start_line: 1,
        end_line: end_idx + 1, // 1-indexed
        body,
    })
}

/// Checks if content has frontmatter (starts with `---`).
pub fn has_frontmatter(content: &str) -> bool {
    content.trim_start().starts_with("---")
}

/// Extracts just the body content after the frontmatter (FR-064).
///
/// Returns the body content without the frontmatter delimiters.
/// If no frontmatter is present, returns the original content.
pub fn extract_body(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() || lines[0].trim() != "---" {
        // No frontmatter, return original content
        return content.to_string();
    }

    // Find closing delimiter
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "---" {
            // Return everything after the closing delimiter
            if i + 1 < lines.len() {
                return lines[i + 1..].join("\n");
            } else {
                return String::new();
            }
        }
    }

    // No closing delimiter found, return original
    content.to_string()
}

/// Updates the YAML frontmatter in content while preserving the body (FR-064).
///
/// The new_yaml should NOT include the `---` delimiters.
/// Returns the updated content with new frontmatter and preserved body.
pub fn update_frontmatter(content: &str, new_yaml: &str) -> String {
    let body = extract_body(content);

    // Ensure trailing newline in YAML
    let yaml_trimmed = new_yaml.trim_end();

    if body.is_empty() {
        format!("---\n{}\n---\n", yaml_trimmed)
    } else {
        format!("---\n{}\n---\n{}", yaml_trimmed, body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_extract_frontmatter_valid() {
        let content = r#"---
id: "SOL-001"
type: solution
name: "Test"
---
# Body content"#;

        let result = extract_frontmatter(content, &PathBuf::from("test.md")).unwrap();
        assert!(result.yaml.contains("id: \"SOL-001\""));
        assert!(result.yaml.contains("type: solution"));
        assert_eq!(result.start_line, 1);
        assert_eq!(result.end_line, 5);
        assert_eq!(result.body.trim(), "# Body content");
    }

    #[test]
    fn test_extract_frontmatter_no_body() {
        let content = r#"---
id: "SOL-001"
---"#;

        let result = extract_frontmatter(content, &PathBuf::from("test.md")).unwrap();
        assert!(result.yaml.contains("id: \"SOL-001\""));
        assert!(result.body.is_empty());
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

        let new_yaml = r#"id: "SOL-001"
type: solution
name: "New Name""#;

        let updated = update_frontmatter(content, new_yaml);

        assert!(updated.starts_with("---\n"));
        assert!(updated.contains("name: \"New Name\""));
        assert!(updated.contains("# Body Content"));
        assert!(updated.contains("Some markdown here."));
    }

    #[test]
    fn test_update_frontmatter_no_body() {
        let content = "---\nid: test\n---";
        let new_yaml = "id: test\nname: Updated";

        let updated = update_frontmatter(content, new_yaml);

        assert_eq!(updated, "---\nid: test\nname: Updated\n---\n");
    }
}

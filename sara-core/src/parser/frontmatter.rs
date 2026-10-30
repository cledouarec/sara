//! YAML frontmatter extraction from Markdown files.

use std::ops::Range;
use std::path::Path;

use serde_yaml::Mapping;

use crate::error::SaraError;

/// Line that opens and closes a frontmatter block.
const DELIMITER: &str = "---";

/// Whether `lines` open with a frontmatter delimiter.
fn opens_frontmatter(lines: &[&str]) -> bool {
    lines.first().is_some_and(|line| line.trim() == DELIMITER)
}

/// Returns the index of the line closing the frontmatter block that opens
/// `lines`, or `None` when they open no block or the block is unclosed.
fn closing_delimiter_index(lines: &[&str]) -> Option<usize> {
    if !opens_frontmatter(lines) {
        return None;
    }
    lines
        .iter()
        .enumerate()
        .skip(1)
        .find(|(_, line)| line.trim() == DELIMITER)
        .map(|(i, _)| i)
}

/// Returns the index of the line closing the frontmatter block that opens
/// `lines`, read from `file`.
///
/// # Errors
///
/// Returns [`SaraError::MissingFrontmatter`] when `lines` open no block, and
/// [`SaraError::InvalidFrontmatter`] when the block is unclosed.
fn frontmatter_end(lines: &[&str], file: &Path) -> Result<usize, SaraError> {
    if !opens_frontmatter(lines) {
        return Err(SaraError::MissingFrontmatter {
            file: file.to_path_buf(),
        });
    }
    closing_delimiter_index(lines).ok_or_else(|| SaraError::InvalidFrontmatter {
        file: file.to_path_buf(),
        reason: "Missing closing `---` delimiter".to_string(),
    })
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
    let end_idx = frontmatter_end(&lines, file)?;
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

    match closing_delimiter_index(&lines) {
        Some(end_idx) => lines[end_idx + 1..].join("\n"),
        // No frontmatter block, or an unclosed one: return the original
        None => content.to_string(),
    }
}

/// One change to a top-level key of a frontmatter block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrontmatterEdit {
    /// Writes `entry` in place of the key's current entry, or appends it
    /// at the end of the block when the key is absent.
    Set {
        /// The top-level key.
        key: String,
        /// The rendered entry: its `key:` line and the lines of its value,
        /// separated by `\n`.
        entry: String,
    },
    /// Removes the key and its value; does nothing when the key is absent.
    Remove {
        /// The top-level key.
        key: String,
    },
}

/// Applies `edits` to top-level keys of the frontmatter in `content`.
///
/// Every byte outside the edited entries is kept as is: other keys,
/// comments, blank lines, quoting style, line endings and the body. An
/// entry spans its `key:` line and the indented or `-` lines that follow;
/// blank lines and column-0 comments belong to it only when more such lines
/// follow them. Any other column-0 line, such as the next key, ends it.
/// Inserted lines use the line ending of the opening delimiter. Edits
/// apply in order.
///
/// # Errors
///
/// Returns [`SaraError::MissingFrontmatter`] when `content` does not start
/// with a frontmatter block, and [`SaraError::InvalidFrontmatter`] when the
/// block is not closed.
pub fn patch_frontmatter(
    content: &str,
    edits: &[FrontmatterEdit],
    file: &Path,
) -> Result<String, SaraError> {
    let lines: Vec<&str> = content.split_inclusive('\n').collect();
    let end_idx = frontmatter_end(&lines, file)?;

    let line_ending = line_ending(content);
    let mut entries: Vec<String> = lines[1..end_idx]
        .iter()
        .map(|line| (*line).to_string())
        .collect();

    for edit in edits {
        match edit {
            FrontmatterEdit::Set { key, entry } => {
                let rendered = entry.lines().map(|line| format!("{line}{line_ending}"));
                match entry_span(&entries, key) {
                    Some(span) => {
                        entries.splice(span, rendered);
                    }
                    None => entries.extend(rendered),
                }
            }
            FrontmatterEdit::Remove { key } => {
                if let Some(span) = entry_span(&entries, key) {
                    entries.drain(span);
                }
            }
        }
    }

    let mut patched = String::with_capacity(content.len());
    patched.push_str(lines[0]);
    entries.iter().for_each(|line| patched.push_str(line));
    lines[end_idx..]
        .iter()
        .for_each(|line| patched.push_str(line));

    let edited: Vec<&str> = edits.iter().map(FrontmatterEdit::key).collect();
    let changed = keys_changed_outside(content, &patched, &edited, file)?;
    if !changed.is_empty() {
        return Err(SaraError::InvalidFrontmatter {
            file: file.to_path_buf(),
            reason: format!(
                "editing {} would also change {}",
                edited.join(", "),
                changed.join(", ")
            ),
        });
    }
    Ok(patched)
}

impl FrontmatterEdit {
    /// Returns the key this edit applies to.
    #[must_use]
    pub fn key(&self) -> &str {
        match self {
            Self::Set { key, .. } | Self::Remove { key } => key,
        }
    }
}

/// Returns the keys outside `edited` whose value differs between the
/// frontmatter of `original` and that of `patched`, compared as YAML values
/// so that a mere change of quoting does not count.
///
/// # Errors
///
/// Returns an error when either frontmatter is not a valid YAML mapping.
fn keys_changed_outside(
    original: &str,
    patched: &str,
    edited: &[&str],
    file: &Path,
) -> Result<Vec<String>, SaraError> {
    let before = frontmatter_mapping(original, file)?;
    let after = frontmatter_mapping(patched, file)?;

    let mut changed = Vec::new();
    for key in before.keys().chain(after.keys()) {
        let name = key
            .as_str()
            .map_or_else(|| format!("{key:?}"), str::to_string);
        let outside = !edited.contains(&name.as_str());
        if outside && before.get(key) != after.get(key) && !changed.contains(&name) {
            changed.push(name);
        }
    }
    Ok(changed)
}

/// Parses the frontmatter of `content` as a YAML mapping; an empty block
/// is an empty mapping.
fn frontmatter_mapping(content: &str, file: &Path) -> Result<Mapping, SaraError> {
    let yaml = extract_frontmatter(content, file)?;
    serde_yaml::from_str::<Option<Mapping>>(&yaml)
        .map(Option::unwrap_or_default)
        .map_err(|e| SaraError::InvalidYaml {
            file: file.to_path_buf(),
            reason: e.to_string(),
        })
}

/// Returns the top-level entry of `key` in the frontmatter of `content`.
///
/// The entry is the `key:` line and the lines of its value, as delimited by
/// [`patch_frontmatter`], joined with `\n` and without line terminators, so
/// it can be passed back as a [`FrontmatterEdit::Set`] entry. Returns `None`
/// when `content` has no closed frontmatter block or the key is absent.
#[must_use]
pub fn frontmatter_entry(content: &str, key: &str) -> Option<String> {
    let lines: Vec<&str> = content.split_inclusive('\n').collect();
    let end_idx = closing_delimiter_index(&lines)?;
    let entries = &lines[1..end_idx];
    let span = entry_span(entries, key)?;
    let entry: Vec<&str> = entries[span]
        .iter()
        .map(|line| line.trim_end_matches(['\r', '\n']))
        .collect();
    Some(entry.join("\n"))
}

/// Returns the range of `lines` holding the entry of `key`, if present.
fn entry_span<S: AsRef<str>>(lines: &[S], key: &str) -> Option<Range<usize>> {
    let start = lines
        .iter()
        .position(|line| top_level_key(line.as_ref()) == Some(key))?;

    // Blank lines and column-0 comments belong to the entry only when more of
    // its lines follow them, as with a list item commented out.
    let mut end = start + 1;
    for (idx, line) in lines.iter().enumerate().skip(start + 1) {
        let line = line.as_ref();
        if continues_entry(line) {
            end = idx + 1;
        } else if !line.trim().is_empty() && !line.starts_with('#') {
            break;
        }
    }

    Some(start..end)
}

/// Returns the key of a line that opens a top-level `key:` entry.
fn top_level_key(line: &str) -> Option<&str> {
    if line.starts_with([' ', '\t', '#', '-']) {
        return None;
    }

    let (key, rest) = line.split_once(':')?;
    let value_follows = rest.is_empty() || rest.starts_with([' ', '\t', '\r', '\n']);
    value_follows.then_some(key.trim_end())
}

/// Whether a line belongs to the value of the entry above it: it is
/// indented, or it is a list item at column 0.
fn continues_entry(line: &str) -> bool {
    !line.trim().is_empty() && line.starts_with([' ', '\t', '-'])
}

/// Returns the line ending of the first line of `content`: `\r\n` when it
/// ends so, `\n` otherwise.
pub(crate) fn line_ending(content: &str) -> &'static str {
    let first = content.split_inclusive('\n').next().unwrap_or_default();
    if first.ends_with("\r\n") {
        "\r\n"
    } else {
        "\n"
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

    /// Sets `key` to the given rendered entry.
    fn set(key: &str, entry: &str) -> FrontmatterEdit {
        FrontmatterEdit::Set {
            key: key.to_string(),
            entry: entry.to_string(),
        }
    }

    /// Patches `content` with the given edits, panicking on error.
    fn patch(content: &str, edits: &[FrontmatterEdit]) -> String {
        patch_frontmatter(content, edits, &PathBuf::from("test.md")).unwrap()
    }

    #[test]
    fn test_patch_replaces_only_the_edited_key() {
        let content = r#"---
id: "SCEN-001"
type: scenario
name: "Adjust Living Room Dimmer"
description: >
  Folded text that spans
  two lines
refines:
  - "UC-001"
# Declared on this side on purpose
derives:
  - "SYSREQ-001"
owner: "alice"
---
# Body
"#;

        let patched = patch(content, &[set("name", "name: \"Dim the lights\"")]);

        assert_eq!(
            patched,
            content.replace(
                "name: \"Adjust Living Room Dimmer\"",
                "name: \"Dim the lights\""
            )
        );
    }

    #[test]
    fn test_patch_replaces_a_block_scalar_with_its_lines() {
        let content = "---\nid: x\ndescription: |\n  first\n\n  second\nname: y\n---\n";

        let patched = patch(content, &[set("description", "description: \"short\"")]);

        assert_eq!(
            patched,
            "---\nid: x\ndescription: \"short\"\nname: y\n---\n"
        );
    }

    #[test]
    fn test_patch_keeps_blank_line_after_replaced_key() {
        let content = "---\nid: x\n\nname: y\n---\n";

        let patched = patch(content, &[set("id", "id: \"z\"")]);

        assert_eq!(patched, "---\nid: \"z\"\n\nname: y\n---\n");
    }

    #[test]
    fn test_patch_replaces_list_items_at_column_zero() {
        let content = "---\nderives:\n- \"A\"\n- \"B\"\nname: y\n---\n";

        let patched = patch(content, &[set("derives", "derives:\n  - \"C\"")]);

        assert_eq!(patched, "---\nderives:\n  - \"C\"\nname: y\n---\n");
    }

    #[test]
    fn test_patch_removes_list_interrupted_by_a_comment() {
        // A list item commented out at column 0 must not end the list: the
        // items after it would otherwise be left to the previous key.
        let content = "---\ntags:\n  - \"safety\"\ndepends_on:\n  - \"SYSREQ-001\"\n#  - \"SYSREQ-999\"\n  - \"SYSREQ-002\"\nname: y\n---\n";

        let patched = patch(
            content,
            &[FrontmatterEdit::Remove {
                key: "depends_on".to_string(),
            }],
        );

        assert_eq!(patched, "---\ntags:\n  - \"safety\"\nname: y\n---\n");
    }

    #[test]
    fn test_patch_keeps_comment_before_the_next_key() {
        let content = "---\nrefines:\n  - \"UC-001\"\n# About derives\nderives:\n  - \"A\"\n---\n";

        let patched = patch(content, &[set("refines", "refines:\n  - \"UC-002\"")]);

        assert_eq!(
            patched,
            "---\nrefines:\n  - \"UC-002\"\n# About derives\nderives:\n  - \"A\"\n---\n"
        );
    }

    #[test]
    fn test_patch_appends_missing_key_before_closing_delimiter() {
        let content = "---\nid: x\n---\nBody\n";

        let patched = patch(content, &[set("status", "status: \"draft\"")]);

        assert_eq!(patched, "---\nid: x\nstatus: \"draft\"\n---\nBody\n");
    }

    #[test]
    fn test_patch_removes_key_with_its_list() {
        let content = "---\nid: x\nderives:\n  - \"A\"\nname: y\n---\n";

        let patched = patch(
            content,
            &[FrontmatterEdit::Remove {
                key: "derives".to_string(),
            }],
        );

        assert_eq!(patched, "---\nid: x\nname: y\n---\n");
    }

    #[test]
    fn test_patch_keeps_crlf_line_endings() {
        let content = "---\r\nid: x\r\nname: y\r\n---\r\nBody\r\n";

        let patched = patch(
            content,
            &[
                set("name", "name: \"z\""),
                set("derives", "derives:\n  - \"A\""),
            ],
        );

        assert_eq!(
            patched,
            "---\r\nid: x\r\nname: \"z\"\r\nderives:\r\n  - \"A\"\r\n---\r\nBody\r\n"
        );
    }

    #[test]
    fn test_patch_requires_frontmatter() {
        let result = patch_frontmatter(
            "# Just markdown",
            &[set("name", "name: x")],
            &PathBuf::from("test.md"),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_keys_changed_outside_reports_other_keys() {
        let changed = keys_changed_outside(
            "---\na: 1\nb: [x]\n---\n",
            "---\na: 9\nb: [x, y]\n---\n",
            &["a"],
            &PathBuf::from("test.md"),
        )
        .unwrap();

        assert_eq!(changed, ["b"]);
    }

    #[test]
    fn test_keys_changed_outside_ignores_edited_and_restyled_keys() {
        // A value spelled differently but equal as YAML is unchanged.
        let changed = keys_changed_outside(
            "---\na: 1\nb: \"x\"\n---\n",
            "---\na: 9\nb: x\nc: 2\n---\n",
            &["a", "c"],
            &PathBuf::from("test.md"),
        )
        .unwrap();

        assert!(changed.is_empty(), "{changed:?}");
    }

    #[test]
    fn test_frontmatter_entry_returns_scalar_entry() {
        let content = "---\nid: \"X\"\nname: \"N\"\n---\nBody\n";

        assert_eq!(
            frontmatter_entry(content, "name").as_deref(),
            Some("name: \"N\"")
        );
    }

    #[test]
    fn test_frontmatter_entry_returns_list_with_its_items() {
        let content = "---\nid: \"X\"\nderives:\n  - \"A\"\n  - \"B\"\nname: \"N\"\n---\n";

        assert_eq!(
            frontmatter_entry(content, "derives").as_deref(),
            Some("derives:\n  - \"A\"\n  - \"B\"")
        );
    }

    #[test]
    fn test_frontmatter_entry_strips_crlf() {
        let content = "---\r\nname: \"N\"\r\n---\r\n";

        assert_eq!(
            frontmatter_entry(content, "name").as_deref(),
            Some("name: \"N\"")
        );
    }

    #[test]
    fn test_frontmatter_entry_absent_key() {
        assert_eq!(frontmatter_entry("---\nid: \"X\"\n---\n", "name"), None);
        assert_eq!(frontmatter_entry("# No frontmatter", "name"), None);
    }
}

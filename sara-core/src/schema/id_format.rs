//! Bidirectional engine for `id_format` identifier templates.
//!
//! A format is literal text plus `{placeholder[:spec]}` segments (`{{` and
//! `}}` escape literal braces). The same compiled template renders new
//! identifiers and recognizes existing ones, so generation, the sequence
//! scan and the conformance check stay consistent by construction.

use std::fmt::Write as _;

use chrono::{Datelike, Utc};
use uuid::Uuid;

/// Hyphen positions in a canonical hyphenated UUID.
const UUID_HYPHENS: [usize; 4] = [8, 13, 18, 23];
/// Length of a canonical hyphenated UUID.
const UUID_LEN: usize = 36;
/// Index of the version nibble in a canonical hyphenated UUID.
const UUID_VERSION_INDEX: usize = 14;
/// Index of the variant nibble in a canonical hyphenated UUID.
const UUID_VARIANT_INDEX: usize = 19;

/// How non-constant placeholders bind when matching an existing id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Binding {
    /// Temporals bound to today's values — the sequence scope of the
    /// rendered pattern, used when suggesting the next id.
    Scope,
    /// Temporals matched by shape only, used by the conformance check.
    Any,
}

/// One parsed segment of an identifier template.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Segment {
    /// Literal text between placeholders.
    Literal(String),
    /// `{prefix}` — the type's `prefix` key.
    Prefix,
    /// `{id}` — the type's `id` key.
    TypeId,
    /// `{seq}` / `{seq:0N}` — the per-type counter, zero-padded to `width`
    /// (0 means unpadded).
    Seq {
        /// Zero-padding width used when rendering.
        width: usize,
    },
    /// `{year}` — UTC year on 4 digits, frozen at creation.
    Year,
    /// `{month}` — UTC month on 2 digits.
    Month,
    /// `{day}` — UTC day on 2 digits.
    Day,
    /// `{uuid4}` — a random UUID.
    Uuid4,
    /// `{uuid7}` — a time-ordered UUID.
    Uuid7,
}

/// A compiled `id_format` template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IdFormat {
    /// Segments in template order.
    segments: Vec<Segment>,
}

impl IdFormat {
    /// Parses and structurally validates an identifier template.
    ///
    /// # Errors
    ///
    /// Returns a human-readable reason when the template has unbalanced
    /// braces, an unknown placeholder, an invalid spec, duplicate unique
    /// placeholders, no uniqueness source, or an ambiguous segment after
    /// `{seq}`.
    pub(crate) fn parse(format: &str) -> Result<Self, String> {
        let segments = scan_segments(format)?;
        validate_structure(&segments)?;
        Ok(Self { segments })
    }

    /// Renders a new identifier for the given type constants and sequence.
    ///
    /// Temporal placeholders freeze today's UTC date; UUID placeholders draw
    /// a fresh value on every call. `seq` is ignored by formats without a
    /// `{seq}` placeholder.
    #[must_use]
    pub(crate) fn render(&self, prefix: &str, type_id: &str, seq: u32) -> String {
        let (year, month, day) = utc_today();
        let mut id = String::new();
        for segment in &self.segments {
            match segment {
                Segment::Literal(text) => id.push_str(text),
                Segment::Prefix => id.push_str(prefix),
                Segment::TypeId => id.push_str(type_id),
                Segment::Seq { width } => {
                    let width = *width;
                    let _ = write!(id, "{seq:0width$}");
                }
                Segment::Year => {
                    let _ = write!(id, "{year:04}");
                }
                Segment::Month => {
                    let _ = write!(id, "{month:02}");
                }
                Segment::Day => {
                    let _ = write!(id, "{day:02}");
                }
                Segment::Uuid4 => {
                    let _ = write!(id, "{}", Uuid::new_v4());
                }
                Segment::Uuid7 => {
                    let _ = write!(id, "{}", Uuid::now_v7());
                }
            }
        }
        id
    }

    /// Returns whether the template contains a `{seq}` placeholder.
    #[must_use]
    pub(crate) fn has_seq(&self) -> bool {
        self.segments
            .iter()
            .any(|s| matches!(s, Segment::Seq { .. }))
    }

    /// Checks whether an existing id conforms to the template shape.
    ///
    /// Temporal, sequence and UUID placeholders match by shape only, so an
    /// id generated under a previous date stays conformant.
    #[must_use]
    pub(crate) fn matches(&self, id: &str, prefix: &str, type_id: &str) -> bool {
        self.walk(id, prefix, type_id, Binding::Any).is_some()
    }

    /// Extracts the sequence number from an id within the current scope.
    ///
    /// Temporals are bound to today's values, so ids rendered under another
    /// period do not count. Returns `None` when the id does not match, when
    /// the matched sequence overflows `u32`, or when the template has no
    /// `{seq}` placeholder — pair with [`IdFormat::has_seq`].
    #[must_use]
    pub(crate) fn extract_seq(&self, id: &str, prefix: &str, type_id: &str) -> Option<u32> {
        self.walk(id, prefix, type_id, Binding::Scope)?
    }

    /// Walks the segments over `id`; `Some(seq)` on a full match.
    fn walk(&self, id: &str, prefix: &str, type_id: &str, binding: Binding) -> Option<Option<u32>> {
        let (year, month, day) = utc_today();
        let mut rest = id;
        let mut seq = None;

        for segment in &self.segments {
            rest = match segment {
                Segment::Literal(text) => rest.strip_prefix(text.as_str())?,
                Segment::Prefix => rest.strip_prefix(prefix)?,
                Segment::TypeId => rest.strip_prefix(type_id)?,
                Segment::Seq { .. } => {
                    let digits = leading_digits(rest);
                    if digits.is_empty() {
                        return None;
                    }
                    if binding == Binding::Scope {
                        seq = Some(digits.parse::<u32>().ok()?);
                    }
                    &rest[digits.len()..]
                }
                Segment::Year => strip_number(rest, 4, year as u32, binding)?,
                Segment::Month => strip_number(rest, 2, month, binding)?,
                Segment::Day => strip_number(rest, 2, day, binding)?,
                Segment::Uuid4 => strip_uuid(rest, '4')?,
                Segment::Uuid7 => strip_uuid(rest, '7')?,
            };
        }
        rest.is_empty().then_some(seq)
    }
}

/// Splits the raw template into literal and placeholder segments.
fn scan_segments(format: &str) -> Result<Vec<Segment>, String> {
    let mut segments = Vec::new();
    let mut literal = String::new();
    let mut chars = format.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '{' if chars.peek() == Some(&'{') => {
                chars.next();
                literal.push('{');
            }
            '}' if chars.peek() == Some(&'}') => {
                chars.next();
                literal.push('}');
            }
            '}' => return Err("unbalanced '}'".to_string()),
            '{' => {
                let mut body = String::new();
                loop {
                    match chars.next() {
                        Some('}') => break,
                        Some(inner) => body.push(inner),
                        None => return Err("unclosed '{'".to_string()),
                    }
                }
                if !literal.is_empty() {
                    segments.push(Segment::Literal(std::mem::take(&mut literal)));
                }
                push_placeholder(&body, &mut segments)?;
            }
            other => literal.push(other),
        }
    }
    if !literal.is_empty() {
        segments.push(Segment::Literal(literal));
    }
    Ok(segments)
}

/// Resolves one `{...}` body into segments (`{date}` desugars to three).
fn push_placeholder(body: &str, segments: &mut Vec<Segment>) -> Result<(), String> {
    let (name, spec) = match body.split_once(':') {
        Some((name, spec)) => (name, Some(spec)),
        None => (body, None),
    };
    if let Some(spec) = spec {
        if name != "seq" {
            return Err(format!("':{spec}' is only valid on {{seq}}"));
        }
        let width = spec
            .strip_prefix('0')
            .and_then(|w| w.parse::<usize>().ok())
            .filter(|w| *w >= 1)
            .ok_or_else(|| format!("invalid spec ':{spec}' — expected zero-padding like ':03'"))?;
        segments.push(Segment::Seq { width });
        return Ok(());
    }
    match name {
        "prefix" => segments.push(Segment::Prefix),
        "id" => segments.push(Segment::TypeId),
        "seq" => segments.push(Segment::Seq { width: 0 }),
        "year" => segments.push(Segment::Year),
        "month" => segments.push(Segment::Month),
        "day" => segments.push(Segment::Day),
        "date" => segments.extend([Segment::Year, Segment::Month, Segment::Day]),
        "uuid4" => segments.push(Segment::Uuid4),
        "uuid7" => segments.push(Segment::Uuid7),
        unknown => return Err(format!("unknown placeholder '{{{unknown}}}'")),
    }
    Ok(())
}

/// Enforces the structural rules that keep matching unambiguous.
fn validate_structure(segments: &[Segment]) -> Result<(), String> {
    if segments.is_empty() {
        return Err("the format is empty".to_string());
    }

    if segments
        .iter()
        .filter(|s| matches!(s, Segment::Seq { .. }))
        .count()
        > 1
    {
        return Err("at most one {seq} is allowed".to_string());
    }
    if segments
        .iter()
        .filter(|s| matches!(s, Segment::Uuid4))
        .count()
        > 1
    {
        return Err("at most one {uuid4} is allowed".to_string());
    }
    if segments
        .iter()
        .filter(|s| matches!(s, Segment::Uuid7))
        .count()
        > 1
    {
        return Err("at most one {uuid7} is allowed".to_string());
    }

    let unique = |s: &Segment| matches!(s, Segment::Seq { .. } | Segment::Uuid4 | Segment::Uuid7);
    if !segments.iter().any(unique) {
        return Err("at least one of {seq}, {uuid4} or {uuid7} is required".to_string());
    }

    if let Some(pos) = segments
        .iter()
        .position(|s| matches!(s, Segment::Seq { .. }))
    {
        let unambiguous = match segments.get(pos + 1) {
            None => true,
            Some(Segment::Literal(text)) => !text.starts_with(|c: char| c.is_ascii_digit()),
            Some(_) => false,
        };
        if !unambiguous {
            return Err(
                "{seq} must be followed by the end of the format or a literal \
                        that does not start with a digit"
                    .to_string(),
            );
        }
    }
    Ok(())
}

/// Returns the longest ASCII-digit prefix of `s`.
fn leading_digits(s: &str) -> &str {
    let len = s.bytes().take_while(u8::is_ascii_digit).count();
    &s[..len]
}

/// Strips a fixed-width number; under `Scope` it must equal `bound`.
fn strip_number(s: &str, len: usize, bound: u32, binding: Binding) -> Option<&str> {
    let (head, rest) = s.split_at_checked(len)?;
    if !head.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    if binding == Binding::Scope && head.parse::<u32>().ok()? != bound {
        return None;
    }
    Some(rest)
}

/// Strips a canonical lowercase hyphenated UUID of the given version.
fn strip_uuid(s: &str, version: char) -> Option<&str> {
    let (head, rest) = s.split_at_checked(UUID_LEN)?;
    let valid = head.char_indices().all(|(i, c)| {
        if UUID_HYPHENS.contains(&i) {
            c == '-'
        } else if i == UUID_VERSION_INDEX {
            c == version
        } else if i == UUID_VARIANT_INDEX {
            matches!(c, '8' | '9' | 'a' | 'b')
        } else {
            c.is_ascii_hexdigit() && !c.is_ascii_uppercase()
        }
    });
    valid.then_some(rest)
}

/// Returns today's UTC civil date as `(year, month, day)`.
fn utc_today() -> (i32, u32, u32) {
    let now = Utc::now();
    (now.year(), now.month(), now.day())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_format_parses() {
        assert!(IdFormat::parse("{prefix}-{seq:03}").is_ok());
    }

    #[test]
    fn test_every_placeholder_parses() {
        assert!(IdFormat::parse("{id}_{year}{month}{day}-{seq}").is_ok());
        assert!(IdFormat::parse("{prefix}-{date}-{seq:02}").is_ok());
        assert!(IdFormat::parse("{prefix}-{uuid4}").is_ok());
        assert!(IdFormat::parse("{prefix}-{uuid7}").is_ok());
    }

    #[test]
    fn test_escaped_braces_are_literals() {
        assert!(IdFormat::parse("{{v}}-{seq}").is_ok());
    }

    #[test]
    fn test_wide_padding_parses() {
        assert!(IdFormat::parse("{prefix}-{seq:010}").is_ok());
    }

    #[test]
    fn test_empty_format_is_rejected() {
        assert!(IdFormat::parse("").unwrap_err().contains("empty"));
    }

    #[test]
    fn test_missing_uniqueness_source_is_rejected() {
        let reason = IdFormat::parse("{prefix}").unwrap_err();
        assert!(reason.contains("at least one of"), "got: {reason}");
    }

    #[test]
    fn test_duplicate_unique_placeholders_are_rejected() {
        assert!(IdFormat::parse("{seq}-{seq}").is_err());
        assert!(IdFormat::parse("{uuid4}-{uuid4}").is_err());
        assert!(IdFormat::parse("{uuid7}-{uuid7}").is_err());
    }

    #[test]
    fn test_ambiguous_segment_after_seq_is_rejected() {
        assert!(IdFormat::parse("{seq}1").is_err());
        assert!(IdFormat::parse("{seq}{year}").is_err());
        assert!(IdFormat::parse("{seq}{uuid4}").is_err());
        assert!(IdFormat::parse("{prefix}-{seq}-x").is_ok());
    }

    #[test]
    fn test_invalid_specs_are_rejected() {
        let reason = IdFormat::parse("{seq:3}").unwrap_err();
        assert!(reason.contains("zero-padding"), "got: {reason}");
        assert!(IdFormat::parse("{seq:}").is_err());
        assert!(IdFormat::parse("{seq:0}").is_err());
        assert!(IdFormat::parse("{prefix:03}-{seq}").is_err());
    }

    #[test]
    fn test_unknown_placeholder_is_rejected() {
        let reason = IdFormat::parse("{bogus}-{seq}").unwrap_err();
        assert!(reason.contains("unknown placeholder"), "got: {reason}");
    }

    #[test]
    fn test_unbalanced_braces_are_rejected() {
        assert!(IdFormat::parse("{seq").unwrap_err().contains("unclosed"));
        assert!(
            IdFormat::parse("}x{seq}")
                .unwrap_err()
                .contains("unbalanced")
        );
    }

    const UUID_TAIL_LEN: usize = 36;

    /// Asserts `tail` is a canonical lowercase hyphenated UUID of `version`.
    fn assert_uuid_shape(tail: &str, version: char) {
        assert_eq!(tail.len(), UUID_TAIL_LEN, "got: {tail}");
        for (i, c) in tail.char_indices() {
            match i {
                8 | 13 | 18 | 23 => assert_eq!(c, '-', "got: {tail}"),
                14 => assert_eq!(c, version, "got: {tail}"),
                19 => assert!(matches!(c, '8' | '9' | 'a' | 'b'), "got: {tail}"),
                _ => assert!(
                    c.is_ascii_hexdigit() && !c.is_ascii_uppercase(),
                    "got: {tail}"
                ),
            }
        }
    }

    #[test]
    fn test_render_builtin_format() {
        let format = IdFormat::parse("{prefix}-{seq:03}").unwrap();
        assert_eq!(format.render("SOL", "solution", 1), "SOL-001");
        assert_eq!(format.render("SOL", "solution", 1234), "SOL-1234");
    }

    #[test]
    fn test_render_unpadded_and_type_id() {
        let format = IdFormat::parse("{id}-{seq}").unwrap();
        assert_eq!(format.render("SOL", "solution", 7), "solution-7");
    }

    #[test]
    fn test_render_escaped_braces() {
        let format = IdFormat::parse("{{{seq}}}").unwrap();
        assert_eq!(format.render("SOL", "solution", 1), "{1}");
    }

    #[test]
    fn test_render_temporals_use_utc_today() {
        let format = IdFormat::parse("{date}-{seq}").unwrap();
        let (year, month, day) = utc_today();
        assert_eq!(
            format.render("SOL", "solution", 9),
            format!("{year:04}{month:02}{day:02}-9")
        );
    }

    #[test]
    fn test_render_uuid_shapes_and_uniqueness() {
        let v4 = IdFormat::parse("{prefix}-{uuid4}").unwrap();
        let first = v4.render("SOL", "solution", 1);
        let second = v4.render("SOL", "solution", 1);
        assert_uuid_shape(first.strip_prefix("SOL-").unwrap(), '4');
        assert_ne!(first, second);

        let v7 = IdFormat::parse("{prefix}-{uuid7}").unwrap();
        let id = v7.render("SOL", "solution", 1);
        assert_uuid_shape(id.strip_prefix("SOL-").unwrap(), '7');
    }

    #[test]
    fn test_extract_seq_accepts_any_padding() {
        let format = IdFormat::parse("{prefix}-{seq:03}").unwrap();
        assert_eq!(format.extract_seq("SOL-042", "SOL", "solution"), Some(42));
        assert_eq!(format.extract_seq("SOL-7", "SOL", "solution"), Some(7));
        assert_eq!(format.extract_seq("SOL-0007", "SOL", "solution"), Some(7));
    }

    #[test]
    fn test_extract_seq_rejects_non_matching_ids() {
        let format = IdFormat::parse("{prefix}-{seq:03}").unwrap();
        assert_eq!(format.extract_seq("UC-001", "SOL", "solution"), None);
        assert_eq!(format.extract_seq("SOL-", "SOL", "solution"), None);
        assert_eq!(format.extract_seq("SOL-001X", "SOL", "solution"), None);
        assert_eq!(format.extract_seq("SOL-LOGIN", "SOL", "solution"), None);
    }

    #[test]
    fn test_scope_binds_temporals_but_any_does_not() {
        let format = IdFormat::parse("{prefix}-{year}-{seq:02}").unwrap();
        let (year, _, _) = utc_today();
        let current = format!("TKT-{year:04}-07");
        assert_eq!(format.extract_seq(&current, "TKT", "ticket"), Some(7));
        assert_eq!(format.extract_seq("TKT-1999-07", "TKT", "ticket"), None);
        assert!(format.matches("TKT-1999-07", "TKT", "ticket"));
        assert!(!format.matches("TKT-19X9-07", "TKT", "ticket"));
    }

    #[test]
    fn test_matches_validates_uuid_shape() {
        let format = IdFormat::parse("{prefix}-{uuid4}").unwrap();
        let id = format.render("SOL", "solution", 1);
        assert!(format.matches(&id, "SOL", "solution"));

        let wrong_version = id.replacen("SOL-", "", 1);
        let mut bytes = wrong_version.into_bytes();
        bytes[14] = b'7';
        let tampered = format!("SOL-{}", String::from_utf8(bytes).unwrap());
        assert!(!format.matches(&tampered, "SOL", "solution"));

        let uppercase = id.to_uppercase();
        assert!(!format.matches(&uppercase, "SOL", "solution"));
    }

    #[test]
    fn test_seq_overflow_is_shape_conformant_but_not_scannable() {
        let format = IdFormat::parse("{prefix}-{seq}").unwrap();
        let huge = "SOL-99999999999999999999";
        assert!(format.matches(huge, "SOL", "solution"));
        assert_eq!(format.extract_seq(huge, "SOL", "solution"), None);
    }

    #[test]
    fn test_has_seq() {
        assert!(IdFormat::parse("{prefix}-{seq}").unwrap().has_seq());
        assert!(!IdFormat::parse("{prefix}-{uuid4}").unwrap().has_seq());
    }

    #[test]
    fn test_extract_seq_through_the_type_id_placeholder() {
        let format = IdFormat::parse("{id}-{seq}").unwrap();
        assert_eq!(format.extract_seq("solution-7", "SOL", "solution"), Some(7));
        assert_eq!(format.extract_seq("SOL-7", "SOL", "solution"), None);
    }
}

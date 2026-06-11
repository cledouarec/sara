//! Verifies that installing a custom [`Schema`] overrides the values
//! returned by [`ItemType`] and [`RelationshipRules`] methods.
//!
//! Lives as a dedicated integration-test binary so it owns its own
//! `OnceLock` for the active schema — installing here cannot leak into
//! sibling tests, which would otherwise race against the singleton.

use std::path::Path;

use sara_core::model::RelationshipRules;
use sara_core::schema::builtin;
use sara_core::schema::{self, Schema};

/// Builds a schema based on the built-in default but with a custom prefix
/// for [`builtin::SOLUTION`]. Round-trips through YAML to also exercise
/// the public loading path.
fn modified_builtin() -> Schema {
    let yaml = Schema::builtin()
        .to_yaml()
        .expect("serialize builtin")
        .replace("prefix: SOL", "prefix: CUSTOM");
    Schema::from_yaml_str(&yaml, Path::new("<test>")).expect("re-parse modified schema")
}

/// Installing a custom schema must change the values returned by
/// [`ItemType::prefix`] and [`ItemType::generate_id`], while leaving
/// untouched types and the relationship matrix on the built-in fallback.
///
/// Bundled into a single `#[test]` because `Schema::install` uses a
/// process-wide [`std::sync::OnceLock`] that only accepts one install per
/// binary; splitting the assertions would race against parallel test
/// threads observing the singleton before installation.
#[test]
fn active_schema_overrides_propagate_to_domain_enums() {
    schema::install(modified_builtin()).expect("install once at start of test");

    // Overridden type sees the custom prefix everywhere.
    assert_eq!(
        builtin::SOLUTION.prefix(),
        "CUSTOM",
        "prefix() must reflect the active schema"
    );
    assert_eq!(
        builtin::SOLUTION.generate_id(Some(7)),
        "CUSTOM-007",
        "generate_id must compose with the active schema's prefix"
    );

    // Unmodified types still return their built-in values.
    assert_eq!(builtin::USE_CASE.prefix(), "UC");
    assert_eq!(
        builtin::SYSTEM_REQUIREMENT.display_name(),
        "System Requirement"
    );

    // The relationship matrix is unchanged: UseCase still refines Solution
    // and the reverse direction is still invalid.
    assert!(RelationshipRules::is_valid_relationship(
        builtin::USE_CASE,
        builtin::SOLUTION,
        builtin::REFINES,
    ));
    assert!(!RelationshipRules::is_valid_relationship(
        builtin::SOLUTION,
        builtin::USE_CASE,
        builtin::REFINES,
    ));
}

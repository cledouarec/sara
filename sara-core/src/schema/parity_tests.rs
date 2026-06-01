//! Parity tests: [`Schema::builtin`] must reproduce the hard-coded model exactly.

use std::path::Path;

use super::{RelationDirection, Schema};
use crate::model::{ItemType, RelationshipRules, RelationshipType};

/// All relationship types (no `all()` exists on the legacy enum).
const ALL_RELATIONSHIPS: &[RelationshipType] = &[
    RelationshipType::Refines,
    RelationshipType::IsRefinedBy,
    RelationshipType::Derives,
    RelationshipType::DerivesFrom,
    RelationshipType::Satisfies,
    RelationshipType::IsSatisfiedBy,
    RelationshipType::DependsOn,
    RelationshipType::IsRequiredBy,
    RelationshipType::Justifies,
    RelationshipType::IsJustifiedBy,
    RelationshipType::Supersedes,
    RelationshipType::IsSupersededBy,
];

#[test]
fn builtin_covers_every_item_type() {
    let schema = Schema::builtin();
    for item_type in ItemType::all() {
        assert!(
            schema.item_type(item_type.as_str()).is_some(),
            "missing schema definition for {}",
            item_type.as_str()
        );
    }
    assert_eq!(schema.item_types.len(), ItemType::all().len());
}

#[test]
fn prefix_parity() {
    let schema = Schema::builtin();
    for item_type in ItemType::all() {
        let def = schema.item_type(item_type.as_str()).unwrap();
        assert_eq!(def.prefix, item_type.prefix(), "prefix for {item_type}");
    }
}

#[test]
fn display_name_parity() {
    let schema = Schema::builtin();
    for item_type in ItemType::all() {
        let def = schema.item_type(item_type.as_str()).unwrap();
        assert_eq!(
            def.display_name,
            item_type.display_name(),
            "display_name for {item_type}"
        );
    }
}

#[test]
fn parent_type_parity() {
    let schema = Schema::builtin();
    for item_type in ItemType::all() {
        let def = schema.item_type(item_type.as_str()).unwrap();
        let legacy = item_type.required_parent_type().map(|p| p.as_str());
        let derived = def.parent_types.first().map(String::as_str);
        assert_eq!(legacy, derived, "parent type for {item_type}");
        assert!(def.parent_types.len() <= 1, "builtin parents are single");
    }
}

#[test]
fn traceability_configs_parity() {
    let schema = Schema::builtin();
    for item_type in ItemType::all() {
        let def = schema.item_type(item_type.as_str()).unwrap();

        // Legacy traceability = upstream primary relations + the `depends_on`
        // peer relation, in declaration order (matches the legacy `match`).
        let derived: Vec<(String, String)> = def
            .allowed_targets
            .iter()
            .filter(|t| {
                let rel = schema.relation(&t.relation).unwrap();
                rel.direction == RelationDirection::Upstream || t.relation == "depends_on"
            })
            .flat_map(|t| {
                t.targets
                    .iter()
                    .map(move |target| (t.relation.clone(), target.clone()))
            })
            .collect();

        let legacy: Vec<(String, String)> = item_type
            .traceability_configs()
            .iter()
            .map(|c| {
                (
                    c.relationship_field.as_str().to_string(),
                    c.target_type.as_str().to_string(),
                )
            })
            .collect();

        assert_eq!(legacy, derived, "traceability configs for {item_type}");
    }
}

#[test]
fn relationship_matrix_parity() {
    let schema = Schema::builtin();
    for from in ItemType::all() {
        for to in ItemType::all() {
            for rel in ALL_RELATIONSHIPS {
                let legacy = RelationshipRules::is_valid_relationship(*from, *to, *rel);
                let relation_id = rel.field_name().as_str();
                let derived = schema.is_valid_relationship(from.as_str(), to.as_str(), relation_id);
                assert_eq!(
                    legacy, derived,
                    "mismatch: {from} -{relation_id}-> {to} (legacy={legacy}, derived={derived})"
                );
            }
        }
    }
}

#[test]
fn yaml_round_trip_is_lossless() {
    let schema = Schema::builtin();
    let yaml = schema.to_yaml().expect("serialize builtin schema");
    let parsed = Schema::from_yaml_str(&yaml, Path::new("<builtin>"))
        .expect("re-parse serialized builtin schema");
    assert_eq!(schema, parsed);
}

#[test]
fn builtin_passes_validation() {
    let schema = Schema::builtin();
    schema
        .validate(Path::new("<builtin>"))
        .expect("builtin schema must be internally consistent");
}

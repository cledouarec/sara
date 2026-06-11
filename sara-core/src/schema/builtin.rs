//! The default schema shipped with Sara.
//!
//! Designed to be complete but simple: ten item types, twelve relations and a
//! hierarchy matrix that cover the needs of most projects out of the box.
//! Users only need to provide a custom schema when their domain calls for it.
//!
//! The module also exposes one [`ItemType`] / [`RelationshipType`] handle per
//! built-in id, the single place those ids are spelled out: the default
//! schema is built from the same handles. Against a custom schema that drops
//! an id, the corresponding handle resolves to no metadata by design.

use super::{
    AllowedTarget, FieldDef, FieldType, ItemTypeDef, RelationDef, RelationDirection, Schema,
};
use crate::model::{ItemType, RelationshipType};

/// The built-in solution type.
pub const SOLUTION: ItemType = ItemType::from_static("solution");
/// The built-in use case type.
pub const USE_CASE: ItemType = ItemType::from_static("use_case");
/// The built-in scenario type.
pub const SCENARIO: ItemType = ItemType::from_static("scenario");
/// The built-in system requirement type.
pub const SYSTEM_REQUIREMENT: ItemType = ItemType::from_static("system_requirement");
/// The built-in system architecture type.
pub const SYSTEM_ARCHITECTURE: ItemType = ItemType::from_static("system_architecture");
/// The built-in hardware requirement type.
pub const HARDWARE_REQUIREMENT: ItemType = ItemType::from_static("hardware_requirement");
/// The built-in software requirement type.
pub const SOFTWARE_REQUIREMENT: ItemType = ItemType::from_static("software_requirement");
/// The built-in hardware detailed design type.
pub const HARDWARE_DETAILED_DESIGN: ItemType = ItemType::from_static("hardware_detailed_design");
/// The built-in software detailed design type.
pub const SOFTWARE_DETAILED_DESIGN: ItemType = ItemType::from_static("software_detailed_design");
/// The built-in architecture decision record type.
pub const ARCHITECTURE_DECISION_RECORD: ItemType =
    ItemType::from_static("architecture_decision_record");

/// Refinement: child refines parent (Scenario refines Use Case).
pub const REFINES: RelationshipType = RelationshipType::from_static("refines");
/// Inverse of refines: parent is refined by child.
pub const IS_REFINED_BY: RelationshipType = RelationshipType::from_static("is_refined_by");
/// Derivation: parent derives child (Scenario derives System Requirement).
pub const DERIVES: RelationshipType = RelationshipType::from_static("derives");
/// Inverse of derives: child derives from parent.
pub const DERIVES_FROM: RelationshipType = RelationshipType::from_static("derives_from");
/// Satisfaction: child satisfies parent.
pub const SATISFIES: RelationshipType = RelationshipType::from_static("satisfies");
/// Inverse of satisfies: parent is satisfied by child.
pub const IS_SATISFIED_BY: RelationshipType = RelationshipType::from_static("is_satisfied_by");
/// Dependency: an item depends on a peer of the same type.
pub const DEPENDS_ON: RelationshipType = RelationshipType::from_static("depends_on");
/// Inverse of depends_on: an item is required by a peer.
pub const IS_REQUIRED_BY: RelationshipType = RelationshipType::from_static("is_required_by");
/// Justification: an ADR justifies a design artifact.
pub const JUSTIFIES: RelationshipType = RelationshipType::from_static("justifies");
/// Inverse of justifies: a design artifact is justified by an ADR.
pub const IS_JUSTIFIED_BY: RelationshipType = RelationshipType::from_static("justified_by");
/// Supersession: a newer ADR supersedes an older ADR.
pub const SUPERSEDES: RelationshipType = RelationshipType::from_static("supersedes");
/// Inverse of supersedes: an older ADR is superseded by a newer one.
pub const IS_SUPERSEDED_BY: RelationshipType = RelationshipType::from_static("superseded_by");

/// Convenience constructor for a relation definition.
fn relation(
    id: RelationshipType,
    display_name: &str,
    inverse: RelationshipType,
    direction: RelationDirection,
    primary: bool,
) -> RelationDef {
    RelationDef {
        id: id.as_str().to_string(),
        display_name: display_name.to_string(),
        inverse: inverse.as_str().to_string(),
        direction,
        primary,
    }
}

/// Convenience constructor for an allowed-target entry.
fn allowed(relation: RelationshipType, targets: &[ItemType]) -> AllowedTarget {
    AllowedTarget {
        relation: relation.as_str().to_string(),
        targets: targets.iter().map(|t| t.as_str().to_string()).collect(),
    }
}

/// Convenience constructor for a `specification` field (requirement types).
fn specification_field() -> FieldDef {
    FieldDef {
        name: "specification".to_string(),
        display_name: "Specification".to_string(),
        field_type: FieldType::Text,
        required: true,
        placeholder: Some("The system SHALL <describe the requirement>.".to_string()),
    }
}

impl Schema {
    /// Returns the default schema.
    ///
    /// Meant to cover the typical needs of a systems-engineering project
    /// without configuration: ten item types in hierarchy order, twelve
    /// relations (each with its inverse) and a complete validity matrix.
    /// Custom schemas are only needed for domain-specific extensions.
    #[must_use]
    pub fn builtin() -> Self {
        let relations = vec![
            relation(
                REFINES,
                "Refines",
                IS_REFINED_BY,
                RelationDirection::Upstream,
                true,
            ),
            relation(
                IS_REFINED_BY,
                "Is refined by",
                REFINES,
                RelationDirection::Downstream,
                false,
            ),
            relation(
                DERIVES_FROM,
                "Derives from",
                DERIVES,
                RelationDirection::Upstream,
                true,
            ),
            relation(
                DERIVES,
                "Derives",
                DERIVES_FROM,
                RelationDirection::Downstream,
                false,
            ),
            relation(
                SATISFIES,
                "Satisfies",
                IS_SATISFIED_BY,
                RelationDirection::Upstream,
                true,
            ),
            relation(
                IS_SATISFIED_BY,
                "Is satisfied by",
                SATISFIES,
                RelationDirection::Downstream,
                false,
            ),
            relation(
                DEPENDS_ON,
                "Depends on",
                IS_REQUIRED_BY,
                RelationDirection::Peer,
                true,
            ),
            relation(
                IS_REQUIRED_BY,
                "Is required by",
                DEPENDS_ON,
                RelationDirection::Peer,
                false,
            ),
            relation(
                JUSTIFIES,
                "Justifies",
                IS_JUSTIFIED_BY,
                RelationDirection::Upstream,
                true,
            ),
            relation(
                IS_JUSTIFIED_BY,
                "Justified by",
                JUSTIFIES,
                RelationDirection::Downstream,
                false,
            ),
            relation(
                SUPERSEDES,
                "Supersedes",
                IS_SUPERSEDED_BY,
                RelationDirection::Peer,
                true,
            ),
            relation(
                IS_SUPERSEDED_BY,
                "Superseded by",
                SUPERSEDES,
                RelationDirection::Peer,
                false,
            ),
        ];

        let item_types = vec![
            ItemTypeDef {
                id: SOLUTION.as_str().to_string(),
                display_name: "Solution".to_string(),
                prefix: "SOL".to_string(),
                id_format: "{prefix}-{seq:03}".to_string(),
                parent_types: vec![],
                fields: vec![],
                allowed_targets: vec![],
            },
            ItemTypeDef {
                id: USE_CASE.as_str().to_string(),
                display_name: "Use Case".to_string(),
                prefix: "UC".to_string(),
                id_format: "{prefix}-{seq:03}".to_string(),
                parent_types: vec![SOLUTION.as_str().to_string()],
                fields: vec![],
                allowed_targets: vec![allowed(REFINES, &[SOLUTION])],
            },
            ItemTypeDef {
                id: SCENARIO.as_str().to_string(),
                display_name: "Scenario".to_string(),
                prefix: "SCEN".to_string(),
                id_format: "{prefix}-{seq:03}".to_string(),
                parent_types: vec![USE_CASE.as_str().to_string()],
                fields: vec![],
                allowed_targets: vec![allowed(REFINES, &[USE_CASE])],
            },
            ItemTypeDef {
                id: SYSTEM_REQUIREMENT.as_str().to_string(),
                display_name: "System Requirement".to_string(),
                prefix: "SYSREQ".to_string(),
                id_format: "{prefix}-{seq:03}".to_string(),
                parent_types: vec![SCENARIO.as_str().to_string()],
                fields: vec![specification_field()],
                allowed_targets: vec![
                    allowed(DERIVES_FROM, &[SCENARIO]),
                    allowed(DEPENDS_ON, &[SYSTEM_REQUIREMENT]),
                ],
            },
            ItemTypeDef {
                id: SYSTEM_ARCHITECTURE.as_str().to_string(),
                display_name: "System Architecture".to_string(),
                prefix: "SYSARCH".to_string(),
                id_format: "{prefix}-{seq:03}".to_string(),
                parent_types: vec![SYSTEM_REQUIREMENT.as_str().to_string()],
                fields: vec![FieldDef {
                    name: "platform".to_string(),
                    display_name: "Platform".to_string(),
                    field_type: FieldType::Text,
                    required: false,
                    placeholder: None,
                }],
                allowed_targets: vec![allowed(SATISFIES, &[SYSTEM_REQUIREMENT])],
            },
            ItemTypeDef {
                id: HARDWARE_REQUIREMENT.as_str().to_string(),
                display_name: "Hardware Requirement".to_string(),
                prefix: "HWREQ".to_string(),
                id_format: "{prefix}-{seq:03}".to_string(),
                parent_types: vec![SYSTEM_ARCHITECTURE.as_str().to_string()],
                fields: vec![specification_field()],
                allowed_targets: vec![
                    allowed(DERIVES_FROM, &[SYSTEM_ARCHITECTURE]),
                    allowed(DEPENDS_ON, &[HARDWARE_REQUIREMENT]),
                ],
            },
            ItemTypeDef {
                id: SOFTWARE_REQUIREMENT.as_str().to_string(),
                display_name: "Software Requirement".to_string(),
                prefix: "SWREQ".to_string(),
                id_format: "{prefix}-{seq:03}".to_string(),
                parent_types: vec![SYSTEM_ARCHITECTURE.as_str().to_string()],
                fields: vec![specification_field()],
                allowed_targets: vec![
                    allowed(DERIVES_FROM, &[SYSTEM_ARCHITECTURE]),
                    allowed(DEPENDS_ON, &[SOFTWARE_REQUIREMENT]),
                ],
            },
            ItemTypeDef {
                id: HARDWARE_DETAILED_DESIGN.as_str().to_string(),
                display_name: "Hardware Detailed Design".to_string(),
                prefix: "HWDD".to_string(),
                id_format: "{prefix}-{seq:03}".to_string(),
                parent_types: vec![HARDWARE_REQUIREMENT.as_str().to_string()],
                fields: vec![],
                allowed_targets: vec![allowed(SATISFIES, &[HARDWARE_REQUIREMENT])],
            },
            ItemTypeDef {
                id: SOFTWARE_DETAILED_DESIGN.as_str().to_string(),
                display_name: "Software Detailed Design".to_string(),
                prefix: "SWDD".to_string(),
                id_format: "{prefix}-{seq:03}".to_string(),
                parent_types: vec![SOFTWARE_REQUIREMENT.as_str().to_string()],
                fields: vec![],
                allowed_targets: vec![allowed(SATISFIES, &[SOFTWARE_REQUIREMENT])],
            },
            ItemTypeDef {
                id: ARCHITECTURE_DECISION_RECORD.as_str().to_string(),
                display_name: "Architecture Decision Record".to_string(),
                prefix: "ADR".to_string(),
                id_format: "{prefix}-{seq:03}".to_string(),
                parent_types: vec![],
                fields: vec![
                    FieldDef {
                        name: "status".to_string(),
                        display_name: "Status".to_string(),
                        field_type: FieldType::Enum {
                            values: vec![
                                "proposed".to_string(),
                                "accepted".to_string(),
                                "deprecated".to_string(),
                                "superseded".to_string(),
                            ],
                        },
                        required: true,
                        placeholder: Some("proposed".to_string()),
                    },
                    FieldDef {
                        name: "deciders".to_string(),
                        display_name: "Deciders".to_string(),
                        field_type: FieldType::List(Box::new(FieldType::Text)),
                        required: true,
                        placeholder: Some("TBD".to_string()),
                    },
                ],
                allowed_targets: vec![
                    allowed(
                        JUSTIFIES,
                        &[
                            SYSTEM_ARCHITECTURE,
                            SOFTWARE_DETAILED_DESIGN,
                            HARDWARE_DETAILED_DESIGN,
                        ],
                    ),
                    allowed(SUPERSEDES, &[ARCHITECTURE_DECISION_RECORD]),
                ],
            },
        ];

        Self {
            item_types,
            relations,
        }
    }
}

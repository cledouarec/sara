//! The default schema shipped with Sara.
//!
//! Designed to be complete but simple: ten item types, twelve relations and a
//! hierarchy matrix that cover the needs of most projects out of the box.
//! Users only need to provide a custom schema when their domain calls for it.

use super::{
    AllowedTarget, FieldDef, FieldType, ItemTypeDef, RelationDef, RelationDirection, Schema,
};

/// Convenience constructor for a relation definition.
fn relation(
    id: &str,
    display_name: &str,
    inverse: &str,
    direction: RelationDirection,
    primary: bool,
) -> RelationDef {
    RelationDef {
        id: id.to_string(),
        display_name: display_name.to_string(),
        inverse: inverse.to_string(),
        direction,
        primary,
    }
}

/// Convenience constructor for an allowed-target entry.
fn allowed(relation: &str, targets: &[&str]) -> AllowedTarget {
    AllowedTarget {
        relation: relation.to_string(),
        targets: targets.iter().map(|t| (*t).to_string()).collect(),
    }
}

/// Convenience constructor for a `specification` field (requirement types).
fn specification_field() -> FieldDef {
    FieldDef {
        name: "specification".to_string(),
        display_name: "Specification".to_string(),
        field_type: FieldType::Text,
        required: true,
    }
}

/// Convenience constructor for a `depends_on` peer-reference list field.
fn depends_on_field() -> FieldDef {
    FieldDef {
        name: "depends_on".to_string(),
        display_name: "Depends on".to_string(),
        field_type: FieldType::List(Box::new(FieldType::ItemRef)),
        required: false,
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
                "refines",
                "Refines",
                "is_refined_by",
                RelationDirection::Upstream,
                true,
            ),
            relation(
                "is_refined_by",
                "Is refined by",
                "refines",
                RelationDirection::Downstream,
                false,
            ),
            relation(
                "derives_from",
                "Derives from",
                "derives",
                RelationDirection::Upstream,
                true,
            ),
            relation(
                "derives",
                "Derives",
                "derives_from",
                RelationDirection::Downstream,
                false,
            ),
            relation(
                "satisfies",
                "Satisfies",
                "is_satisfied_by",
                RelationDirection::Upstream,
                true,
            ),
            relation(
                "is_satisfied_by",
                "Is satisfied by",
                "satisfies",
                RelationDirection::Downstream,
                false,
            ),
            relation(
                "depends_on",
                "Depends on",
                "is_required_by",
                RelationDirection::Peer,
                true,
            ),
            relation(
                "is_required_by",
                "Is required by",
                "depends_on",
                RelationDirection::Peer,
                false,
            ),
            relation(
                "justifies",
                "Justifies",
                "justified_by",
                RelationDirection::Upstream,
                true,
            ),
            relation(
                "justified_by",
                "Justified by",
                "justifies",
                RelationDirection::Downstream,
                false,
            ),
            relation(
                "supersedes",
                "Supersedes",
                "superseded_by",
                RelationDirection::Peer,
                true,
            ),
            relation(
                "superseded_by",
                "Superseded by",
                "supersedes",
                RelationDirection::Peer,
                false,
            ),
        ];

        let item_types = vec![
            ItemTypeDef {
                id: "solution".to_string(),
                display_name: "Solution".to_string(),
                prefix: "SOL".to_string(),
                id_format: "{prefix}-{seq:03}".to_string(),
                parent_types: vec![],
                fields: vec![],
                allowed_targets: vec![],
            },
            ItemTypeDef {
                id: "use_case".to_string(),
                display_name: "Use Case".to_string(),
                prefix: "UC".to_string(),
                id_format: "{prefix}-{seq:03}".to_string(),
                parent_types: vec!["solution".to_string()],
                fields: vec![],
                allowed_targets: vec![allowed("refines", &["solution"])],
            },
            ItemTypeDef {
                id: "scenario".to_string(),
                display_name: "Scenario".to_string(),
                prefix: "SCEN".to_string(),
                id_format: "{prefix}-{seq:03}".to_string(),
                parent_types: vec!["use_case".to_string()],
                fields: vec![],
                allowed_targets: vec![allowed("refines", &["use_case"])],
            },
            ItemTypeDef {
                id: "system_requirement".to_string(),
                display_name: "System Requirement".to_string(),
                prefix: "SYSREQ".to_string(),
                id_format: "{prefix}-{seq:03}".to_string(),
                parent_types: vec!["scenario".to_string()],
                fields: vec![specification_field(), depends_on_field()],
                allowed_targets: vec![
                    allowed("derives_from", &["scenario"]),
                    allowed("depends_on", &["system_requirement"]),
                ],
            },
            ItemTypeDef {
                id: "system_architecture".to_string(),
                display_name: "System Architecture".to_string(),
                prefix: "SYSARCH".to_string(),
                id_format: "{prefix}-{seq:03}".to_string(),
                parent_types: vec!["system_requirement".to_string()],
                fields: vec![FieldDef {
                    name: "platform".to_string(),
                    display_name: "Platform".to_string(),
                    field_type: FieldType::Text,
                    required: false,
                }],
                allowed_targets: vec![allowed("satisfies", &["system_requirement"])],
            },
            ItemTypeDef {
                id: "hardware_requirement".to_string(),
                display_name: "Hardware Requirement".to_string(),
                prefix: "HWREQ".to_string(),
                id_format: "{prefix}-{seq:03}".to_string(),
                parent_types: vec!["system_architecture".to_string()],
                fields: vec![specification_field(), depends_on_field()],
                allowed_targets: vec![
                    allowed("derives_from", &["system_architecture"]),
                    allowed("depends_on", &["hardware_requirement"]),
                ],
            },
            ItemTypeDef {
                id: "software_requirement".to_string(),
                display_name: "Software Requirement".to_string(),
                prefix: "SWREQ".to_string(),
                id_format: "{prefix}-{seq:03}".to_string(),
                parent_types: vec!["system_architecture".to_string()],
                fields: vec![specification_field(), depends_on_field()],
                allowed_targets: vec![
                    allowed("derives_from", &["system_architecture"]),
                    allowed("depends_on", &["software_requirement"]),
                ],
            },
            ItemTypeDef {
                id: "hardware_detailed_design".to_string(),
                display_name: "Hardware Detailed Design".to_string(),
                prefix: "HWDD".to_string(),
                id_format: "{prefix}-{seq:03}".to_string(),
                parent_types: vec!["hardware_requirement".to_string()],
                fields: vec![],
                allowed_targets: vec![allowed("satisfies", &["hardware_requirement"])],
            },
            ItemTypeDef {
                id: "software_detailed_design".to_string(),
                display_name: "Software Detailed Design".to_string(),
                prefix: "SWDD".to_string(),
                id_format: "{prefix}-{seq:03}".to_string(),
                parent_types: vec!["software_requirement".to_string()],
                fields: vec![],
                allowed_targets: vec![allowed("satisfies", &["software_requirement"])],
            },
            ItemTypeDef {
                id: "architecture_decision_record".to_string(),
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
                    },
                    FieldDef {
                        name: "deciders".to_string(),
                        display_name: "Deciders".to_string(),
                        field_type: FieldType::List(Box::new(FieldType::Text)),
                        required: true,
                    },
                    FieldDef {
                        name: "supersedes".to_string(),
                        display_name: "Supersedes".to_string(),
                        field_type: FieldType::List(Box::new(FieldType::ItemRef)),
                        required: false,
                    },
                ],
                allowed_targets: vec![
                    allowed(
                        "justifies",
                        &[
                            "system_architecture",
                            "software_detailed_design",
                            "hardware_detailed_design",
                        ],
                    ),
                    allowed("supersedes", &["architecture_decision_record"]),
                ],
            },
        ];

        Self {
            item_types,
            relations,
        }
    }
}

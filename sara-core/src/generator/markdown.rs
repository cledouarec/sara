//! Markdown document generation using Tera templates.
//!
//! Renders full Markdown documents (frontmatter + body) from core `Item`
//! structures. The Tera templates are embedded at compile time from
//! `sara-core/templates/*.tera`.

use std::sync::OnceLock;

use tera::{Context, Tera};

use crate::model::{FieldName, Item, ItemAttributes, ItemType, RelationshipType};

/// Embedded templates compiled into the binary.
const SOLUTION_TEMPLATE: &str = include_str!("../../templates/solution.tera");
const USE_CASE_TEMPLATE: &str = include_str!("../../templates/use_case.tera");
const SCENARIO_TEMPLATE: &str = include_str!("../../templates/scenario.tera");
const SYSTEM_REQUIREMENT_TEMPLATE: &str = include_str!("../../templates/system_requirement.tera");
const HARDWARE_REQUIREMENT_TEMPLATE: &str =
    include_str!("../../templates/hardware_requirement.tera");
const SOFTWARE_REQUIREMENT_TEMPLATE: &str =
    include_str!("../../templates/software_requirement.tera");
const SYSTEM_ARCHITECTURE_TEMPLATE: &str =
    include_str!("../../templates/system_architecture.tera");
const HARDWARE_DETAILED_DESIGN_TEMPLATE: &str =
    include_str!("../../templates/hardware_detailed_design.tera");
const SOFTWARE_DETAILED_DESIGN_TEMPLATE: &str =
    include_str!("../../templates/software_detailed_design.tera");
const ADR_TEMPLATE: &str = include_str!("../../templates/adr.tera");

/// Global Tera instance, lazily initialized.
static TERA: OnceLock<Tera> = OnceLock::new();

/// Returns the global Tera instance, initializing on first call.
fn get_tera() -> &'static Tera {
    TERA.get_or_init(|| {
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![
            ("solution.tera", SOLUTION_TEMPLATE),
            ("use_case.tera", USE_CASE_TEMPLATE),
            ("scenario.tera", SCENARIO_TEMPLATE),
            ("system_requirement.tera", SYSTEM_REQUIREMENT_TEMPLATE),
            ("hardware_requirement.tera", HARDWARE_REQUIREMENT_TEMPLATE),
            ("software_requirement.tera", SOFTWARE_REQUIREMENT_TEMPLATE),
            ("system_architecture.tera", SYSTEM_ARCHITECTURE_TEMPLATE),
            (
                "hardware_detailed_design.tera",
                HARDWARE_DETAILED_DESIGN_TEMPLATE,
            ),
            (
                "software_detailed_design.tera",
                SOFTWARE_DETAILED_DESIGN_TEMPLATE,
            ),
            ("adr.tera", ADR_TEMPLATE),
        ])
        .expect("Failed to load embedded templates");
        tera
    })
}

/// Generates a complete Markdown document (frontmatter + body) from an `Item`.
#[must_use]
pub fn generate_document(item: &Item) -> String {
    let tera = get_tera();
    let context = build_context(item);
    let template_name = template_name_for(item.item_type);
    tera.render(template_name, &context)
        .expect("Failed to render document template")
}

/// Builds a Tera context from an `Item`, populating all fields the templates expect.
fn build_context(item: &Item) -> Context {
    let mut context = Context::new();

    // Required fields
    context.insert(FieldName::Id.as_str(), item.id.as_str());
    context.insert(FieldName::Type.as_str(), item.item_type.as_str());
    context.insert(FieldName::Name.as_str(), &escape_yaml_string(&item.name));

    if let Some(ref desc) = item.description {
        context.insert(FieldName::Description.as_str(), &escape_yaml_string(desc));
    }

    // Relationship-based fields
    insert_relationship_ids(&mut context, item, RelationshipType::Refines, FieldName::Refines);
    insert_relationship_ids(
        &mut context,
        item,
        RelationshipType::DerivesFrom,
        FieldName::DerivesFrom,
    );
    insert_relationship_ids(
        &mut context,
        item,
        RelationshipType::Satisfies,
        FieldName::Satisfies,
    );
    insert_relationship_ids(
        &mut context,
        item,
        RelationshipType::Justifies,
        FieldName::Justifies,
    );

    // Type-specific attributes
    match &item.attributes {
        ItemAttributes::Solution
        | ItemAttributes::UseCase
        | ItemAttributes::Scenario
        | ItemAttributes::SoftwareDetailedDesign
        | ItemAttributes::HardwareDetailedDesign => {}

        ItemAttributes::SystemRequirement {
            specification,
            depends_on,
        }
        | ItemAttributes::SoftwareRequirement {
            specification,
            depends_on,
        }
        | ItemAttributes::HardwareRequirement {
            specification,
            depends_on,
        } => {
            if !depends_on.is_empty() {
                let ids: Vec<&str> = depends_on.iter().map(|id| id.as_str()).collect();
                context.insert(FieldName::DependsOn.as_str(), &ids);
            }
            context.insert(
                FieldName::Specification.as_str(),
                &escape_yaml_string(specification),
            );
        }

        ItemAttributes::SystemArchitecture { platform } => {
            if let Some(plat) = platform {
                context.insert(FieldName::Platform.as_str(), &escape_yaml_string(plat));
            }
        }

        ItemAttributes::Adr {
            status,
            deciders,
            supersedes,
        } => {
            context.insert(FieldName::Status.as_str(), status.as_str());
            if !deciders.is_empty() {
                context.insert(FieldName::Deciders.as_str(), deciders);
            }
            if !supersedes.is_empty() {
                let ids: Vec<&str> = supersedes.iter().map(|id| id.as_str()).collect();
                context.insert(FieldName::Supersedes.as_str(), &ids);
            }
        }
    }

    context
}

/// Inserts relationship target IDs into the Tera context if any exist.
fn insert_relationship_ids(
    context: &mut Context,
    item: &Item,
    rel_type: RelationshipType,
    field: FieldName,
) {
    let ids: Vec<&str> = item
        .relationship_ids(rel_type)
        .map(|id| id.as_str())
        .collect();
    if !ids.is_empty() {
        context.insert(field.as_str(), &ids);
    }
}

/// Returns the Tera template name for the given item type.
const fn template_name_for(item_type: ItemType) -> &'static str {
    match item_type {
        ItemType::Solution => "solution.tera",
        ItemType::UseCase => "use_case.tera",
        ItemType::Scenario => "scenario.tera",
        ItemType::SystemRequirement => "system_requirement.tera",
        ItemType::HardwareRequirement => "hardware_requirement.tera",
        ItemType::SoftwareRequirement => "software_requirement.tera",
        ItemType::SystemArchitecture => "system_architecture.tera",
        ItemType::HardwareDetailedDesign => "hardware_detailed_design.tera",
        ItemType::SoftwareDetailedDesign => "software_detailed_design.tera",
        ItemType::ArchitectureDecisionRecord => "adr.tera",
    }
}

/// Escapes a string for safe embedding in YAML quoted values.
fn escape_yaml_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        AdrStatus, ItemAttributes, ItemBuilder, ItemId, Relationship, RelationshipType,
    };
    use crate::model::SourceLocation;
    use std::path::PathBuf;

    fn test_source() -> SourceLocation {
        SourceLocation {
            repository: PathBuf::from("/repo"),
            file_path: PathBuf::from("docs/test.md"),
            git_ref: None,
        }
    }

    #[test]
    fn test_generate_document_solution() {
        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("SOL-001"))
            .item_type(ItemType::Solution)
            .name("Test Solution")
            .source(test_source())
            .build()
            .unwrap();

        let doc = generate_document(&item);

        assert!(doc.contains("# Solution: Test Solution"));
        assert!(doc.contains("## Overview"));
        assert!(doc.contains("## Goals & KPIs"));
    }

    #[test]
    fn test_generate_document_use_case_with_refines() {
        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("UC-001"))
            .item_type(ItemType::UseCase)
            .name("Test Use Case")
            .source(test_source())
            .relationships(vec![Relationship::new(
                ItemId::new_unchecked("SOL-001"),
                RelationshipType::Refines,
            )])
            .build()
            .unwrap();

        let doc = generate_document(&item);

        assert!(doc.contains("# Use Case: Test Use Case"));
        assert!(doc.contains("## Actor(s)"));
        assert!(doc.contains("refines:"));
        assert!(doc.contains("SOL-001"));
    }

    #[test]
    fn test_generate_document_system_architecture_with_platform() {
        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("SYSARCH-001"))
            .item_type(ItemType::SystemArchitecture)
            .name("Web Platform Architecture")
            .source(test_source())
            .platform("AWS Lambda")
            .relationships(vec![Relationship::new(
                ItemId::new_unchecked("SYSREQ-001"),
                RelationshipType::Satisfies,
            )])
            .build()
            .unwrap();

        let doc = generate_document(&item);

        assert!(doc.contains("id: \"SYSARCH-001\""));
        assert!(doc.contains("type: system_architecture"));
        assert!(doc.contains("platform: \"AWS Lambda\""));
        assert!(doc.contains("satisfies:"));
        assert!(doc.contains("SYSREQ-001"));
    }

    #[test]
    fn test_generate_document_adr() {
        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("ADR-001"))
            .item_type(ItemType::ArchitectureDecisionRecord)
            .name("Use Microservices Architecture")
            .description("Decision to adopt microservices")
            .source(test_source())
            .status(AdrStatus::Proposed)
            .deciders(vec![
                "Alice Smith".to_string(),
                "Bob Jones".to_string(),
            ])
            .relationships(vec![Relationship::new(
                ItemId::new_unchecked("SYSARCH-001"),
                RelationshipType::Justifies,
            )])
            .build()
            .unwrap();

        let doc = generate_document(&item);

        assert!(doc.contains("id: \"ADR-001\""));
        assert!(doc.contains("type: architecture_decision_record"));
        assert!(doc.contains("status: proposed"));
        assert!(doc.contains("deciders:"));
        assert!(doc.contains("Alice Smith"));
        assert!(doc.contains("Bob Jones"));
        assert!(doc.contains("justifies:"));
        assert!(doc.contains("SYSARCH-001"));
        assert!(doc.contains("# Architecture Decision: Use Microservices Architecture"));
        assert!(doc.contains("## Context and problem statement"));
        assert!(doc.contains("## Considered options"));
        assert!(doc.contains("## Decision Outcome"));
    }

    #[test]
    fn test_generate_document_system_requirement() {
        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("SYSREQ-001"))
            .item_type(ItemType::SystemRequirement)
            .name("Performance Requirement")
            .source(test_source())
            .specification("The system SHALL respond within 200ms.")
            .relationships(vec![Relationship::new(
                ItemId::new_unchecked("SCEN-001"),
                RelationshipType::DerivesFrom,
            )])
            .build()
            .unwrap();

        let doc = generate_document(&item);

        assert!(doc.contains("id: \"SYSREQ-001\""));
        assert!(doc.contains("type: system_requirement"));
        assert!(doc.contains("specification:"));
        assert!(doc.contains("derives_from:"));
        assert!(doc.contains("SCEN-001"));
    }
}

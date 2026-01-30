//! Markdown document generation using Tera templates.
//!
//! Generates complete Markdown documents from domain types using the
//! templates in `templates/*.tera`.

use std::sync::OnceLock;

use tera::{Context, Tera};

use crate::model::{FieldName, Item, ItemId, ItemType, RelationshipType};

/// Embedded templates - compiled into the binary.
const SOLUTION_TEMPLATE: &str = include_str!("../../templates/solution.tera");
const USE_CASE_TEMPLATE: &str = include_str!("../../templates/use_case.tera");
const SCENARIO_TEMPLATE: &str = include_str!("../../templates/scenario.tera");
const SYSTEM_REQUIREMENT_TEMPLATE: &str = include_str!("../../templates/system_requirement.tera");
const HARDWARE_REQUIREMENT_TEMPLATE: &str =
    include_str!("../../templates/hardware_requirement.tera");
const SOFTWARE_REQUIREMENT_TEMPLATE: &str =
    include_str!("../../templates/software_requirement.tera");
const SYSTEM_ARCHITECTURE_TEMPLATE: &str = include_str!("../../templates/system_architecture.tera");
const HARDWARE_DETAILED_DESIGN_TEMPLATE: &str =
    include_str!("../../templates/hardware_detailed_design.tera");
const SOFTWARE_DETAILED_DESIGN_TEMPLATE: &str =
    include_str!("../../templates/software_detailed_design.tera");
const ADR_TEMPLATE: &str = include_str!("../../templates/adr.tera");

/// Global Tera instance, lazily initialized.
static TERA: OnceLock<Tera> = OnceLock::new();

/// Gets or initializes the global Tera instance.
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

/// Generate a complete Markdown document from an Item.
pub fn generate_document(item: &Item) -> String {
    let tera = get_tera();
    let context = build_context(item);
    let template_name = template_name_for_type(item.item_type);

    tera.render(template_name, &context)
        .expect("Failed to render document template")
}

/// Returns the template name for the given item type.
fn template_name_for_type(item_type: ItemType) -> &'static str {
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

/// Inserts a list of IDs into the context if non-empty.
fn insert_ids_if_present(context: &mut Context, field: FieldName, ids: &[ItemId]) {
    if !ids.is_empty() {
        context.insert(field.as_str(), &ItemId::slice_to_strs(ids));
    }
}

/// Builds a Tera context from an Item.
fn build_context(item: &Item) -> Context {
    let mut context = Context::new();

    // Core fields
    context.insert(FieldName::Id.as_str(), item.id.as_str());
    context.insert(FieldName::Type.as_str(), item.item_type.as_str());
    context.insert(FieldName::Name.as_str(), &escape_yaml_string(&item.name));

    if let Some(desc) = &item.description {
        context.insert(FieldName::Description.as_str(), &escape_yaml_string(desc));
    }

    // Extract relationship IDs by type
    let refines: Vec<&str> = item
        .relationship_ids(RelationshipType::Refines)
        .map(ItemId::as_str)
        .collect();
    if !refines.is_empty() {
        context.insert(FieldName::Refines.as_str(), &refines);
    }

    let derives_from: Vec<&str> = item
        .relationship_ids(RelationshipType::DerivesFrom)
        .map(ItemId::as_str)
        .collect();
    if !derives_from.is_empty() {
        context.insert(FieldName::DerivesFrom.as_str(), &derives_from);
    }

    let satisfies: Vec<&str> = item
        .relationship_ids(RelationshipType::Satisfies)
        .map(ItemId::as_str)
        .collect();
    if !satisfies.is_empty() {
        context.insert(FieldName::Satisfies.as_str(), &satisfies);
    }

    let justifies: Vec<&str> = item
        .relationship_ids(RelationshipType::Justifies)
        .map(ItemId::as_str)
        .collect();
    if !justifies.is_empty() {
        context.insert(FieldName::Justifies.as_str(), &justifies);
    }

    // Type-specific fields from attributes
    if let Some(depends_on) = item.attributes.depends_on_as_option() {
        insert_ids_if_present(&mut context, FieldName::DependsOn, depends_on);
    }

    if let Some(spec) = item.attributes.specification() {
        context.insert(FieldName::Specification.as_str(), &escape_yaml_string(spec));
    }

    if let Some(platform) = item.attributes.platform() {
        context.insert(FieldName::Platform.as_str(), &escape_yaml_string(platform));
    }

    if let Some(status) = item.attributes.status() {
        context.insert(FieldName::Status.as_str(), status.as_str());
    }

    let deciders = item.attributes.deciders();
    if !deciders.is_empty() {
        context.insert(FieldName::Deciders.as_str(), deciders);
    }

    insert_ids_if_present(
        &mut context,
        FieldName::Supersedes,
        item.attributes.supersedes(),
    );

    context
}

/// Escapes a string for YAML.
fn escape_yaml_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AdrStatus, ItemAttributes, ItemBuilder, ItemId, Relationship, SourceLocation};
    use std::path::PathBuf;

    fn test_source() -> SourceLocation {
        SourceLocation::new(PathBuf::from("/test"), "test.md")
    }

    #[test]
    fn test_generate_document_solution() {
        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("SOL-001"))
            .item_type(ItemType::Solution)
            .name("Test Solution")
            .description("A test solution")
            .source(test_source())
            .attributes(ItemAttributes::Solution)
            .build()
            .unwrap();

        let doc = generate_document(&item);

        assert!(doc.contains("---"));
        assert!(doc.contains("id: \"SOL-001\""));
        assert!(doc.contains("# Solution: Test Solution"));
        assert!(doc.contains("## Overview"));
        assert!(doc.contains("## Goals & KPIs"));
    }

    #[test]
    fn test_generate_document_use_case() {
        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("UC-001"))
            .item_type(ItemType::UseCase)
            .name("Test Use Case")
            .source(test_source())
            .relationships(vec![Relationship::new(
                ItemId::new_unchecked("SOL-001"),
                RelationshipType::Refines,
            )])
            .attributes(ItemAttributes::UseCase)
            .build()
            .unwrap();

        let doc = generate_document(&item);

        assert!(doc.contains("# Use Case: Test Use Case"));
        assert!(doc.contains("refines:"));
        assert!(doc.contains("SOL-001"));
    }

    #[test]
    fn test_generate_document_requirement() {
        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("SYSREQ-001"))
            .item_type(ItemType::SystemRequirement)
            .name("Test Requirement")
            .source(test_source())
            .relationships(vec![Relationship::new(
                ItemId::new_unchecked("SCEN-001"),
                RelationshipType::DerivesFrom,
            )])
            .attributes(ItemAttributes::SystemRequirement {
                specification: "The system SHALL do X".to_string(),
                depends_on: Vec::new(),
            })
            .build()
            .unwrap();

        let doc = generate_document(&item);

        assert!(doc.contains("id: \"SYSREQ-001\""));
        assert!(doc.contains("specification:"));
        assert!(doc.contains("derives_from:"));
    }

    #[test]
    fn test_generate_document_adr() {
        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("ADR-001"))
            .item_type(ItemType::ArchitectureDecisionRecord)
            .name("Use Microservices")
            .description("Decision about architecture")
            .source(test_source())
            .attributes(ItemAttributes::Adr {
                status: AdrStatus::Proposed,
                deciders: vec!["Alice".to_string()],
                supersedes: Vec::new(),
            })
            .build()
            .unwrap();

        let doc = generate_document(&item);

        assert!(doc.contains("# Architecture Decision: Use Microservices"));
        assert!(doc.contains("status: proposed"));
        assert!(doc.contains("deciders:"));
        assert!(doc.contains("## Context and problem statement"));
        assert!(doc.contains("## Decision Outcome"));
    }
}

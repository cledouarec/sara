//! Markdown document generation using Tera templates.
//!
//! Generates complete Markdown documents from domain types using the
//! templates in `templates/*.tera`.

use std::sync::OnceLock;

use tera::{Context, Tera};

use crate::model::{FieldName, InitData, ItemType, TypeFields};

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

/// Generate a complete Markdown document from InitData.
pub fn generate_document(data: &InitData) -> String {
    let tera = get_tera();
    let context = build_context(
        data.item_type,
        &data.id,
        &data.name,
        data.description.as_deref(),
        &data.fields,
    );
    let template_name = template_name_for_type(data.item_type);

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

/// Builds a Tera context from the provided data.
fn build_context(
    item_type: ItemType,
    id: &str,
    name: &str,
    description: Option<&str>,
    fields: &TypeFields,
) -> Context {
    let mut context = Context::new();

    // Core fields
    context.insert(FieldName::Id.as_str(), id);
    context.insert(FieldName::Type.as_str(), item_type.as_str());
    context.insert(FieldName::Name.as_str(), &escape_yaml_string(name));

    if let Some(desc) = description {
        context.insert(FieldName::Description.as_str(), &escape_yaml_string(desc));
    }

    // Traceability fields
    if !fields.refines.is_empty() {
        context.insert(FieldName::Refines.as_str(), &fields.refines);
    }

    if !fields.derives_from.is_empty() {
        context.insert(FieldName::DerivesFrom.as_str(), &fields.derives_from);
    }

    if !fields.satisfies.is_empty() {
        context.insert(FieldName::Satisfies.as_str(), &fields.satisfies);
    }

    if !fields.depends_on.is_empty() {
        context.insert(FieldName::DependsOn.as_str(), &fields.depends_on);
    }

    // Requirement-specific fields
    if let Some(ref spec) = fields.specification {
        context.insert(FieldName::Specification.as_str(), &escape_yaml_string(spec));
    }

    // Architecture-specific fields
    if let Some(ref platform) = fields.platform {
        context.insert(FieldName::Platform.as_str(), &escape_yaml_string(platform));
    }

    // ADR-specific fields
    if let Some(ref status) = fields.status {
        context.insert(FieldName::Status.as_str(), status.as_str());
    }

    if !fields.deciders.is_empty() {
        context.insert(FieldName::Deciders.as_str(), &fields.deciders);
    }

    if !fields.justifies.is_empty() {
        context.insert(FieldName::Justifies.as_str(), &fields.justifies);
    }

    if !fields.supersedes.is_empty() {
        context.insert(FieldName::Supersedes.as_str(), &fields.supersedes);
    }

    if let Some(ref superseded_by) = fields.superseded_by {
        context.insert(FieldName::SupersededBy.as_str(), superseded_by);
    }

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
    use crate::model::AdrStatus;

    #[test]
    fn test_generate_document_solution() {
        let data = InitData {
            id: "SOL-001".to_string(),
            name: "Test Solution".to_string(),
            item_type: ItemType::Solution,
            description: Some("A test solution".to_string()),
            fields: TypeFields::default(),
        };

        let doc = generate_document(&data);

        assert!(doc.contains("---"));
        assert!(doc.contains("id: \"SOL-001\""));
        assert!(doc.contains("# Solution: Test Solution"));
        assert!(doc.contains("## Overview"));
        assert!(doc.contains("## Goals & KPIs"));
    }

    #[test]
    fn test_generate_document_use_case() {
        let fields = TypeFields::new().with_refines(vec!["SOL-001".to_string()]);
        let data = InitData {
            id: "UC-001".to_string(),
            name: "Test Use Case".to_string(),
            item_type: ItemType::UseCase,
            description: None,
            fields,
        };

        let doc = generate_document(&data);

        assert!(doc.contains("# Use Case: Test Use Case"));
        assert!(doc.contains("refines:"));
        assert!(doc.contains("SOL-001"));
    }

    #[test]
    fn test_generate_document_requirement() {
        let fields = TypeFields::new()
            .with_specification("The system SHALL do X")
            .with_derives_from(vec!["SCEN-001".to_string()]);

        let data = InitData {
            id: "SYSREQ-001".to_string(),
            name: "Test Requirement".to_string(),
            item_type: ItemType::SystemRequirement,
            description: None,
            fields,
        };

        let doc = generate_document(&data);

        assert!(doc.contains("id: \"SYSREQ-001\""));
        assert!(doc.contains("specification:"));
        assert!(doc.contains("derives_from:"));
    }

    #[test]
    fn test_generate_document_adr() {
        let fields = TypeFields::new()
            .with_status(AdrStatus::Proposed)
            .with_deciders(vec!["Alice".to_string()]);

        let data = InitData {
            id: "ADR-001".to_string(),
            name: "Use Microservices".to_string(),
            item_type: ItemType::ArchitectureDecisionRecord,
            description: Some("Decision about architecture".to_string()),
            fields,
        };

        let doc = generate_document(&data);

        assert!(doc.contains("# Architecture Decision: Use Microservices"));
        assert!(doc.contains("status: proposed"));
        assert!(doc.contains("deciders:"));
        assert!(doc.contains("## Context and problem statement"));
        assert!(doc.contains("## Decision Outcome"));
    }
}

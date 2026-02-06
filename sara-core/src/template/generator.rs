//! Frontmatter template generation using Tera.

use std::sync::OnceLock;

use tera::{Context, Tera};

use crate::init::TypeConfig;
use crate::model::{FieldName, ItemType};

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

/// Options for generating frontmatter.
#[derive(Debug, Clone)]
pub struct GeneratorOptions {
    /// The item ID.
    pub id: String,
    /// The item name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Type-specific attributes.
    pub type_config: TypeConfig,
}

impl GeneratorOptions {
    /// Creates new generator options with defaults for the given item type.
    pub fn new(item_type: ItemType, id: String, name: String) -> Self {
        Self {
            id,
            name,
            description: None,
            type_config: TypeConfig::from_item_type(item_type),
        }
    }

    /// Creates new generator options with specific type configuration.
    pub fn with_type_config(id: String, name: String, type_config: TypeConfig) -> Self {
        Self {
            id,
            name,
            description: None,
            type_config,
        }
    }

    /// Returns the item type for these options.
    pub fn item_type(&self) -> ItemType {
        self.type_config.item_type()
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets refines references (for UseCase, Scenario).
    pub fn with_refines(mut self, refs: Vec<String>) -> Self {
        match &mut self.type_config {
            TypeConfig::UseCase { refines } | TypeConfig::Scenario { refines } => {
                *refines = refs;
            }
            _ => {}
        }
        self
    }

    /// Sets derives_from references (for requirement types).
    pub fn with_derives_from(mut self, refs: Vec<String>) -> Self {
        match &mut self.type_config {
            TypeConfig::SystemRequirement { derives_from, .. }
            | TypeConfig::SoftwareRequirement { derives_from, .. }
            | TypeConfig::HardwareRequirement { derives_from, .. } => {
                *derives_from = refs;
            }
            _ => {}
        }
        self
    }

    /// Sets satisfies references (for architecture and design types).
    pub fn with_satisfies(mut self, refs: Vec<String>) -> Self {
        match &mut self.type_config {
            TypeConfig::SystemArchitecture { satisfies, .. }
            | TypeConfig::SoftwareDetailedDesign { satisfies }
            | TypeConfig::HardwareDetailedDesign { satisfies } => {
                *satisfies = refs;
            }
            _ => {}
        }
        self
    }

    /// Sets peer dependencies (for requirement types).
    pub fn with_depends_on(mut self, refs: Vec<String>) -> Self {
        match &mut self.type_config {
            TypeConfig::SystemRequirement { depends_on, .. }
            | TypeConfig::SoftwareRequirement { depends_on, .. }
            | TypeConfig::HardwareRequirement { depends_on, .. } => {
                *depends_on = refs;
            }
            _ => {}
        }
        self
    }

    /// Sets the specification (for requirement types).
    pub fn with_specification(mut self, spec: impl Into<String>) -> Self {
        match &mut self.type_config {
            TypeConfig::SystemRequirement { specification, .. }
            | TypeConfig::SoftwareRequirement { specification, .. }
            | TypeConfig::HardwareRequirement { specification, .. } => {
                *specification = Some(spec.into());
            }
            _ => {}
        }
        self
    }

    /// Sets the target platform (for SystemArchitecture).
    pub fn with_platform(mut self, plat: impl Into<String>) -> Self {
        if let TypeConfig::SystemArchitecture { platform, .. } = &mut self.type_config {
            *platform = Some(plat.into());
        }
        self
    }

    /// Sets the ADR status.
    pub fn with_status(mut self, stat: impl Into<String>) -> Self {
        if let TypeConfig::Adr { status, .. } = &mut self.type_config {
            *status = Some(stat.into());
        }
        self
    }

    /// Sets the ADR deciders.
    pub fn with_deciders(mut self, decs: Vec<String>) -> Self {
        if let TypeConfig::Adr { deciders, .. } = &mut self.type_config {
            *deciders = decs;
        }
        self
    }

    /// Sets the design artifacts this ADR justifies.
    pub fn with_justifies(mut self, just: Vec<String>) -> Self {
        if let TypeConfig::Adr { justifies, .. } = &mut self.type_config {
            *justifies = just;
        }
        self
    }

    /// Sets the older ADRs this decision supersedes.
    pub fn with_supersedes(mut self, sups: Vec<String>) -> Self {
        if let TypeConfig::Adr { supersedes, .. } = &mut self.type_config {
            *supersedes = sups;
        }
        self
    }

    /// Sets the newer ADR that supersedes this one.
    pub fn with_superseded_by(mut self, sup_by: impl Into<String>) -> Self {
        if let TypeConfig::Adr { superseded_by, .. } = &mut self.type_config {
            *superseded_by = Some(sup_by.into());
        }
        self
    }

    /// Builds a Tera context from the options.
    fn to_context(&self) -> Context {
        let mut context = Context::new();
        let item_type = self.item_type();

        context.insert(FieldName::Id.as_str(), &self.id);
        context.insert(FieldName::Type.as_str(), item_type.as_str());
        context.insert(FieldName::Name.as_str(), &escape_yaml_string(&self.name));

        if let Some(ref desc) = self.description {
            context.insert(FieldName::Description.as_str(), &escape_yaml_string(desc));
        }

        // Insert type-specific attributes
        match &self.type_config {
            TypeConfig::Solution => {
                // Solutions don't have type-specific attributes
            }

            TypeConfig::UseCase { refines } | TypeConfig::Scenario { refines } => {
                if !refines.is_empty() {
                    context.insert(FieldName::Refines.as_str(), refines);
                }
            }

            TypeConfig::SystemRequirement {
                specification,
                derives_from,
                depends_on,
            }
            | TypeConfig::SoftwareRequirement {
                specification,
                derives_from,
                depends_on,
            }
            | TypeConfig::HardwareRequirement {
                specification,
                derives_from,
                depends_on,
            } => {
                if !derives_from.is_empty() {
                    context.insert(FieldName::DerivesFrom.as_str(), derives_from);
                }
                if !depends_on.is_empty() {
                    context.insert(FieldName::DependsOn.as_str(), depends_on);
                }
                let spec = specification
                    .as_deref()
                    .unwrap_or("The system SHALL <describe the requirement>.");
                context.insert(FieldName::Specification.as_str(), &escape_yaml_string(spec));
            }

            TypeConfig::SystemArchitecture {
                platform,
                satisfies,
            } => {
                if !satisfies.is_empty() {
                    context.insert(FieldName::Satisfies.as_str(), satisfies);
                }
                if let Some(plat) = platform {
                    context.insert(FieldName::Platform.as_str(), &escape_yaml_string(plat));
                }
            }

            TypeConfig::SoftwareDetailedDesign { satisfies }
            | TypeConfig::HardwareDetailedDesign { satisfies } => {
                if !satisfies.is_empty() {
                    context.insert(FieldName::Satisfies.as_str(), satisfies);
                }
            }

            TypeConfig::Adr {
                status,
                deciders,
                justifies,
                supersedes,
                superseded_by,
            } => {
                if let Some(stat) = status {
                    context.insert(FieldName::Status.as_str(), stat);
                }
                if !deciders.is_empty() {
                    context.insert(FieldName::Deciders.as_str(), deciders);
                }
                if !justifies.is_empty() {
                    context.insert(FieldName::Justifies.as_str(), justifies);
                }
                if !supersedes.is_empty() {
                    context.insert(FieldName::Supersedes.as_str(), supersedes);
                }
                if let Some(sup_by) = superseded_by {
                    context.insert(FieldName::SupersededBy.as_str(), sup_by);
                }
            }
        }

        context
    }

    /// Returns the template name for the item type.
    fn template_name(&self) -> &'static str {
        match self.item_type() {
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
}

/// Generates a complete document with frontmatter and body.
pub fn generate_document(opts: &GeneratorOptions) -> String {
    let tera = get_tera();
    let context = opts.to_context();
    tera.render(opts.template_name(), &context)
        .expect("Failed to render document template")
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

    #[test]
    fn test_generate_document_solution() {
        let opts = GeneratorOptions::new(
            ItemType::Solution,
            "SOL-001".to_string(),
            "Test Solution".to_string(),
        );
        let doc = generate_document(&opts);

        assert!(doc.contains("# Solution: Test Solution"));
        assert!(doc.contains("## Overview"));
        assert!(doc.contains("## Goals & KPIs"));
    }

    #[test]
    fn test_generate_document_use_case() {
        let opts = GeneratorOptions::new(
            ItemType::UseCase,
            "UC-001".to_string(),
            "Test Use Case".to_string(),
        )
        .with_refines(vec!["SOL-001".to_string()]);

        let doc = generate_document(&opts);

        assert!(doc.contains("# Use Case: Test Use Case"));
        assert!(doc.contains("## Actor(s)"));
        assert!(doc.contains("refines:"));
        assert!(doc.contains("SOL-001"));
    }

    #[test]
    fn test_generate_frontmatter_system_architecture_with_platform() {
        let opts = GeneratorOptions::new(
            ItemType::SystemArchitecture,
            "SYSARCH-001".to_string(),
            "Web Platform Architecture".to_string(),
        )
        .with_platform("AWS Lambda")
        .with_satisfies(vec!["SYSREQ-001".to_string()]);

        let doc = generate_document(&opts);

        assert!(doc.contains("id: \"SYSARCH-001\""));
        assert!(doc.contains("type: system_architecture"));
        assert!(doc.contains("platform: \"AWS Lambda\""));
        assert!(doc.contains("satisfies:"));
        assert!(doc.contains("SYSREQ-001"));
    }

    #[test]
    fn test_generate_document_adr() {
        let opts = GeneratorOptions::new(
            ItemType::ArchitectureDecisionRecord,
            "ADR-001".to_string(),
            "Use Microservices Architecture".to_string(),
        )
        .with_status("proposed")
        .with_deciders(vec!["Alice Smith".to_string(), "Bob Jones".to_string()])
        .with_justifies(vec!["SYSARCH-001".to_string()])
        .with_description("Decision to adopt microservices".to_string());

        let doc = generate_document(&opts);

        // Check frontmatter
        assert!(doc.contains("id: \"ADR-001\""));
        assert!(doc.contains("type: architecture_decision_record"));
        assert!(doc.contains("status: proposed"));
        assert!(doc.contains("deciders:"));
        assert!(doc.contains("Alice Smith"));
        assert!(doc.contains("Bob Jones"));
        assert!(doc.contains("justifies:"));
        assert!(doc.contains("SYSARCH-001"));

        // Check body structure
        assert!(doc.contains("# Architecture Decision: Use Microservices Architecture"));
        assert!(doc.contains("## Context and problem statement"));
        assert!(doc.contains("## Considered options"));
        assert!(doc.contains("## Decision Outcome"));
    }

}

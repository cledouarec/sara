//! Markdown document generation using Tera templates.
//!
//! Renders full Markdown documents (frontmatter + body) from core `Item`
//! structures. Each [`ItemType`] maps to a [`TemplateEntry`] that pairs a
//! frontmatter partial with a full document template. Both are embedded at
//! compile time from `sara-core/templates/*.tera`.

use std::collections::HashMap;
use std::sync::OnceLock;

use tera::{Context, Tera};

use crate::model::{FieldName, Item, ItemAttributes, ItemType, RelationshipType};

/// Pairs a frontmatter partial with its full document template.
#[derive(Debug)]
struct TemplateEntry {
    /// Tera registration name for the frontmatter partial (e.g., `"adr_frontmatter.tera"`).
    frontmatter_name: &'static str,
    /// Tera registration name for the full document (e.g., `"adr.tera"`).
    document_name: &'static str,
    /// Embedded frontmatter template source.
    frontmatter: &'static str,
    /// Embedded full document template source.
    document: &'static str,
}

/// Compile-time list of all template definitions, one per [`ItemType`].
const TEMPLATE_DEFS: &[(ItemType, TemplateEntry)] = &[
    (
        ItemType::Solution,
        TemplateEntry {
            frontmatter_name: "solution_frontmatter.tera",
            document_name: "solution.tera",
            frontmatter: include_str!("../../templates/solution_frontmatter.tera"),
            document: include_str!("../../templates/solution.tera"),
        },
    ),
    (
        ItemType::UseCase,
        TemplateEntry {
            frontmatter_name: "use_case_frontmatter.tera",
            document_name: "use_case.tera",
            frontmatter: include_str!("../../templates/use_case_frontmatter.tera"),
            document: include_str!("../../templates/use_case.tera"),
        },
    ),
    (
        ItemType::Scenario,
        TemplateEntry {
            frontmatter_name: "scenario_frontmatter.tera",
            document_name: "scenario.tera",
            frontmatter: include_str!("../../templates/scenario_frontmatter.tera"),
            document: include_str!("../../templates/scenario.tera"),
        },
    ),
    (
        ItemType::SystemRequirement,
        TemplateEntry {
            frontmatter_name: "system_requirement_frontmatter.tera",
            document_name: "system_requirement.tera",
            frontmatter: include_str!("../../templates/system_requirement_frontmatter.tera"),
            document: include_str!("../../templates/system_requirement.tera"),
        },
    ),
    (
        ItemType::HardwareRequirement,
        TemplateEntry {
            frontmatter_name: "hardware_requirement_frontmatter.tera",
            document_name: "hardware_requirement.tera",
            frontmatter: include_str!("../../templates/hardware_requirement_frontmatter.tera"),
            document: include_str!("../../templates/hardware_requirement.tera"),
        },
    ),
    (
        ItemType::SoftwareRequirement,
        TemplateEntry {
            frontmatter_name: "software_requirement_frontmatter.tera",
            document_name: "software_requirement.tera",
            frontmatter: include_str!("../../templates/software_requirement_frontmatter.tera"),
            document: include_str!("../../templates/software_requirement.tera"),
        },
    ),
    (
        ItemType::SystemArchitecture,
        TemplateEntry {
            frontmatter_name: "system_architecture_frontmatter.tera",
            document_name: "system_architecture.tera",
            frontmatter: include_str!("../../templates/system_architecture_frontmatter.tera"),
            document: include_str!("../../templates/system_architecture.tera"),
        },
    ),
    (
        ItemType::HardwareDetailedDesign,
        TemplateEntry {
            frontmatter_name: "hardware_detailed_design_frontmatter.tera",
            document_name: "hardware_detailed_design.tera",
            frontmatter: include_str!("../../templates/hardware_detailed_design_frontmatter.tera"),
            document: include_str!("../../templates/hardware_detailed_design.tera"),
        },
    ),
    (
        ItemType::SoftwareDetailedDesign,
        TemplateEntry {
            frontmatter_name: "software_detailed_design_frontmatter.tera",
            document_name: "software_detailed_design.tera",
            frontmatter: include_str!("../../templates/software_detailed_design_frontmatter.tera"),
            document: include_str!("../../templates/software_detailed_design.tera"),
        },
    ),
    (
        ItemType::ArchitectureDecisionRecord,
        TemplateEntry {
            frontmatter_name: "adr_frontmatter.tera",
            document_name: "adr.tera",
            frontmatter: include_str!("../../templates/adr_frontmatter.tera"),
            document: include_str!("../../templates/adr.tera"),
        },
    ),
];

/// Holds the Tera engine and the template lookup map.
struct TemplateRegistry {
    tera: Tera,
    entries: HashMap<ItemType, &'static TemplateEntry>,
}

/// Global template registry, lazily initialized.
static REGISTRY: OnceLock<TemplateRegistry> = OnceLock::new();

/// Returns the global template registry, initializing on first call.
fn get_registry() -> &'static TemplateRegistry {
    REGISTRY.get_or_init(|| {
        let mut tera = Tera::default();
        let raw: Vec<(&str, &str)> = TEMPLATE_DEFS
            .iter()
            .flat_map(|(_, e)| {
                [
                    (e.frontmatter_name, e.frontmatter),
                    (e.document_name, e.document),
                ]
            })
            .collect();
        tera.add_raw_templates(raw)
            .expect("Failed to load embedded templates");

        let entries: HashMap<ItemType, &'static TemplateEntry> = TEMPLATE_DEFS
            .iter()
            .map(|(item_type, entry)| (*item_type, entry))
            .collect();

        TemplateRegistry { tera, entries }
    })
}

/// Generates a complete Markdown document (frontmatter + body) from an `Item`.
#[must_use]
pub fn generate_document(item: &Item) -> String {
    let registry = get_registry();
    let context = build_context(item);
    let entry = &registry.entries[&item.item_type];
    registry
        .tera
        .render(entry.document_name, &context)
        .expect("Failed to render document template")
}

/// Renders only the YAML frontmatter block from an `Item`.
#[must_use]
pub fn generate_frontmatter(item: &Item) -> String {
    let registry = get_registry();
    let context = build_context(item);
    let entry = &registry.entries[&item.item_type];
    registry
        .tera
        .render(entry.frontmatter_name, &context)
        .expect("Failed to render frontmatter template")
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
    insert_relationship_ids(
        &mut context,
        item,
        RelationshipType::Refines,
        FieldName::Refines,
    );
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

/// Escapes a string for safe embedding in YAML quoted values.
fn escape_yaml_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::SourceLocation;
    use crate::model::{AdrStatus, ItemBuilder, ItemId, Relationship, RelationshipType};
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
            .deciders(vec!["Alice Smith".to_string(), "Bob Jones".to_string()])
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
    fn test_generate_frontmatter_solution() {
        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("SOL-001"))
            .item_type(ItemType::Solution)
            .name("Test Solution")
            .source(test_source())
            .build()
            .unwrap();

        let fm = generate_frontmatter(&item);

        assert!(fm.starts_with("---"));
        assert!(fm.ends_with("---"));
        assert!(fm.contains("id: \"SOL-001\""));
        assert!(fm.contains("type: solution"));
        assert!(fm.contains("name: \"Test Solution\""));
        assert!(!fm.contains("## Overview"));
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

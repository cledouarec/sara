//! Frontmatter template generation using Tera.

use std::sync::OnceLock;

use tera::{Context, Tera};

use crate::model::{FieldName, ItemType};

/// Embedded templates - compiled into the binary.
const FRONTMATTER_TEMPLATE: &str = include_str!("../../templates/frontmatter.tera");
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

/// Global Tera instance, lazily initialized.
static TERA: OnceLock<Tera> = OnceLock::new();

/// Gets or initializes the global Tera instance.
fn get_tera() -> &'static Tera {
    TERA.get_or_init(|| {
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![
            ("frontmatter.tera", FRONTMATTER_TEMPLATE),
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
        ])
        .expect("Failed to load embedded templates");
        tera
    })
}

/// Options for generating frontmatter.
#[derive(Debug, Clone)]
pub struct GeneratorOptions {
    /// The item type.
    pub item_type: ItemType,
    /// The item ID.
    pub id: String,
    /// The item name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Upstream references (refines).
    pub refines: Vec<String>,
    /// Upstream references (derives_from).
    pub derives_from: Vec<String>,
    /// Upstream references (satisfies).
    pub satisfies: Vec<String>,
    /// Peer dependencies (for requirement types).
    pub depends_on: Vec<String>,
    /// Specification text (for requirement types).
    pub specification: Option<String>,
    /// Target platform (for system_architecture).
    pub platform: Option<String>,
}

impl GeneratorOptions {
    /// Creates new generator options with defaults.
    pub fn new(item_type: ItemType, id: String, name: String) -> Self {
        Self {
            item_type,
            id,
            name,
            description: None,
            refines: Vec::new(),
            derives_from: Vec::new(),
            satisfies: Vec::new(),
            depends_on: Vec::new(),
            specification: None,
            platform: None,
        }
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Adds a refines reference.
    pub fn with_refines(mut self, refs: Vec<String>) -> Self {
        self.refines = refs;
        self
    }

    /// Adds a derives_from reference.
    pub fn with_derives_from(mut self, refs: Vec<String>) -> Self {
        self.derives_from = refs;
        self
    }

    /// Adds a satisfies reference.
    pub fn with_satisfies(mut self, refs: Vec<String>) -> Self {
        self.satisfies = refs;
        self
    }

    /// Adds peer dependencies (for requirement types).
    pub fn with_depends_on(mut self, refs: Vec<String>) -> Self {
        self.depends_on = refs;
        self
    }

    /// Sets the specification.
    pub fn with_specification(mut self, spec: impl Into<String>) -> Self {
        self.specification = Some(spec.into());
        self
    }

    /// Sets the target platform (for system_architecture).
    pub fn with_platform(mut self, platform: impl Into<String>) -> Self {
        self.platform = Some(platform.into());
        self
    }

    /// Builds a Tera context from the options.
    fn to_context(&self) -> Context {
        let mut context = Context::new();
        context.insert(FieldName::Id.as_str(), &self.id);
        context.insert(FieldName::Type.as_str(), self.item_type.as_str());
        context.insert(FieldName::Name.as_str(), &escape_yaml_string(&self.name));

        if let Some(ref desc) = self.description {
            context.insert(FieldName::Description.as_str(), &escape_yaml_string(desc));
        }

        // Insert upstream references based on item type
        match self.item_type {
            ItemType::UseCase | ItemType::Scenario => {
                if !self.refines.is_empty() {
                    context.insert(FieldName::Refines.as_str(), &self.refines);
                }
            }
            ItemType::SystemRequirement
            | ItemType::HardwareRequirement
            | ItemType::SoftwareRequirement => {
                if !self.derives_from.is_empty() {
                    context.insert(FieldName::DerivesFrom.as_str(), &self.derives_from);
                }
                if !self.depends_on.is_empty() {
                    context.insert(FieldName::DependsOn.as_str(), &self.depends_on);
                }
            }
            ItemType::SystemArchitecture => {
                if !self.satisfies.is_empty() {
                    context.insert(FieldName::Satisfies.as_str(), &self.satisfies);
                }
                // Add platform for system architecture
                if let Some(ref platform) = self.platform {
                    context.insert(FieldName::Platform.as_str(), &escape_yaml_string(platform));
                }
            }
            ItemType::HardwareDetailedDesign | ItemType::SoftwareDetailedDesign => {
                if !self.satisfies.is_empty() {
                    context.insert(FieldName::Satisfies.as_str(), &self.satisfies);
                }
            }
            ItemType::Solution => {
                // Solutions don't have upstream references
            }
        }

        // Add specification for requirement types
        if self.item_type.requires_specification() {
            let spec = self
                .specification
                .as_deref()
                .unwrap_or("The system SHALL <describe the requirement>.");
            context.insert(FieldName::Specification.as_str(), &escape_yaml_string(spec));
        }

        context
    }

    /// Returns the template name for the item type.
    fn template_name(&self) -> &'static str {
        match self.item_type {
            ItemType::Solution => "solution.tera",
            ItemType::UseCase => "use_case.tera",
            ItemType::Scenario => "scenario.tera",
            ItemType::SystemRequirement => "system_requirement.tera",
            ItemType::HardwareRequirement => "hardware_requirement.tera",
            ItemType::SoftwareRequirement => "software_requirement.tera",
            ItemType::SystemArchitecture => "system_architecture.tera",
            ItemType::HardwareDetailedDesign => "hardware_detailed_design.tera",
            ItemType::SoftwareDetailedDesign => "software_detailed_design.tera",
        }
    }
}

/// Generates YAML frontmatter for an item.
pub fn generate_frontmatter(opts: &GeneratorOptions) -> String {
    let tera = get_tera();
    let context = opts.to_context();
    tera.render("frontmatter.tera", &context)
        .expect("Failed to render frontmatter template")
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

/// Generates a new ID for the given type, optionally using a sequence number.
pub fn generate_id(item_type: ItemType, sequence: Option<u32>) -> String {
    let prefix = item_type.prefix();
    let num = sequence.unwrap_or(1);
    format!("{}-{:03}", prefix, num)
}

/// Suggests the next ID based on existing items in the graph (FR-044).
///
/// Finds the highest existing ID for the given type and returns the next sequential ID.
/// If no graph is provided or no items exist, returns the first ID (e.g., "SOL-001").
pub fn suggest_next_id(
    item_type: ItemType,
    graph: Option<&crate::graph::KnowledgeGraph>,
) -> String {
    let Some(graph) = graph else {
        return generate_id(item_type, None);
    };

    let prefix = item_type.prefix();
    let max_num = graph
        .items()
        .filter(|item| item.item_type == item_type)
        .filter_map(|item| {
            item.id
                .as_str()
                .strip_prefix(prefix)
                .and_then(|suffix| suffix.trim_start_matches('-').parse::<u32>().ok())
        })
        .max()
        .unwrap_or(0);

    format!("{}-{:03}", prefix, max_num + 1)
}

/// Extracts a name from a markdown file's first heading.
pub fn extract_name_from_content(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(heading) = trimmed.strip_prefix("# ") {
            return Some(heading.trim().to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_frontmatter_solution() {
        let opts = GeneratorOptions::new(
            ItemType::Solution,
            "SOL-001".to_string(),
            "Test Solution".to_string(),
        );
        let frontmatter = generate_frontmatter(&opts);

        assert!(frontmatter.contains("id: \"SOL-001\""));
        assert!(frontmatter.contains("type: solution"));
        assert!(frontmatter.contains("name: \"Test Solution\""));
    }

    #[test]
    fn test_generate_frontmatter_requirement() {
        let opts = GeneratorOptions::new(
            ItemType::SystemRequirement,
            "SYSREQ-001".to_string(),
            "Test Requirement".to_string(),
        )
        .with_specification("The system SHALL do something.");

        let frontmatter = generate_frontmatter(&opts);

        assert!(frontmatter.contains("specification:"));
        assert!(frontmatter.contains("The system SHALL do something."));
    }

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
    fn test_generate_id() {
        assert_eq!(generate_id(ItemType::Solution, Some(1)), "SOL-001");
        assert_eq!(generate_id(ItemType::UseCase, Some(42)), "UC-042");
        assert_eq!(generate_id(ItemType::SystemRequirement, None), "SYSREQ-001");
    }

    #[test]
    fn test_extract_name_from_content() {
        let content = "# My Document\n\nSome content here.";
        assert_eq!(
            extract_name_from_content(content),
            Some("My Document".to_string())
        );
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

        let frontmatter = generate_frontmatter(&opts);

        assert!(frontmatter.contains("id: \"SYSARCH-001\""));
        assert!(frontmatter.contains("type: system_architecture"));
        assert!(frontmatter.contains("platform: \"AWS Lambda\""));
        assert!(frontmatter.contains("satisfies:"));
        assert!(frontmatter.contains("SYSREQ-001"));
    }
}

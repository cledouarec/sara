//! Markdown document generation using Tera templates.
//!
//! Rendering is driven by the active [`crate::schema::Schema`]: a single
//! generic frontmatter template renders the fields and relations declared
//! for the item's type, in declaration order, and the document body is
//! looked up by type id. Bodies for the built-in types are embedded at
//! compile time; types without a dedicated body fall back to a generic
//! body listing the declared text fields. Projects can override a type's
//! body (or supply one for a new type) with `.tera` files discovered from
//! [`TemplatesConfig::paths`] and installed via [`install_overrides`].

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde::Serialize;
use tera::{Context, Tera};

use crate::config::TemplatesConfig;
use crate::error::SaraError;
use crate::model::{FieldName, FieldValue, Item, RelationshipType};
use crate::schema::{self, FieldDef, FieldType, RelationDirection};

/// Tera registration name of the generic frontmatter partial.
const FRONTMATTER_TEMPLATE: &str = "frontmatter.tera";

/// Tera registration name of the generic document body fallback.
const GENERIC_TEMPLATE: &str = "generic.tera";

/// Embedded generic frontmatter template, included by all document templates.
const FRONTMATTER_SOURCE: &str = include_str!("../../templates/frontmatter.tera");

/// Embedded generic document template for types without a dedicated body.
const GENERIC_SOURCE: &str = include_str!("../../templates/generic.tera");

/// Embedded document bodies for the built-in item types, keyed by type id.
const BUILTIN_DOCUMENTS: &[(&str, &str)] = &[
    ("solution", include_str!("../../templates/solution.tera")),
    ("use_case", include_str!("../../templates/use_case.tera")),
    ("scenario", include_str!("../../templates/scenario.tera")),
    (
        "system_requirement",
        include_str!("../../templates/system_requirement.tera"),
    ),
    (
        "hardware_requirement",
        include_str!("../../templates/hardware_requirement.tera"),
    ),
    (
        "software_requirement",
        include_str!("../../templates/software_requirement.tera"),
    ),
    (
        "system_architecture",
        include_str!("../../templates/system_architecture.tera"),
    ),
    (
        "hardware_detailed_design",
        include_str!("../../templates/hardware_detailed_design.tera"),
    ),
    (
        "software_detailed_design",
        include_str!("../../templates/software_detailed_design.tera"),
    ),
    (
        "architecture_decision_record",
        include_str!("../../templates/adr.tera"),
    ),
];

/// A runtime-discovered Tera document template for one item type.
#[derive(Debug, Clone)]
pub struct TemplateOverride {
    /// Id of the item type whose document this template renders.
    pub type_id: String,
    /// Tera template source. May include `frontmatter.tera`.
    pub source: String,
}

/// Holds the installed template overrides, if any.
static OVERRIDES: OnceLock<Vec<TemplateOverride>> = OnceLock::new();

/// Installs runtime document template overrides.
///
/// Intended to be called once at startup, before the first document is
/// generated; overrides installed after a document has been rendered have no
/// effect. Sources must be valid Tera templates ([`discover_overrides`]
/// validates them), otherwise the first generation panics.
///
/// # Errors
///
/// Returns the supplied overrides unchanged if overrides are already
/// installed, mirroring the underlying [`OnceLock::set`] semantics.
pub fn install_overrides(overrides: Vec<TemplateOverride>) -> Result<(), Vec<TemplateOverride>> {
    OVERRIDES.set(overrides)
}

/// Discovers `.tera` document templates referenced by a templates config.
///
/// Each entry of [`TemplatesConfig::paths`] may be a `.tera` file or a
/// directory scanned (non-recursively, in name order) for `.tera` files. The
/// file stem names the item type the template renders (e.g. `use_case.tera`).
/// Entries that do not exist or do not end in `.tera` are skipped, so the
/// same configuration can also carry Markdown templates consumed elsewhere.
/// Every discovered source is compiled eagerly so that a broken template
/// surfaces here as a configuration error instead of a render panic.
///
/// # Errors
///
/// Returns [`SaraError::InvalidConfig`] if a discovered file cannot be read
/// or is not a valid Tera template.
pub fn discover_overrides(config: &TemplatesConfig) -> Result<Vec<TemplateOverride>, SaraError> {
    let mut overrides = Vec::new();
    for raw in &config.paths {
        let path = Path::new(raw);
        if path.is_dir() {
            let mut files: Vec<PathBuf> = fs::read_dir(path)
                .map_err(|e| invalid_config(path, &e.to_string()))?
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|p| is_tera_file(p) && p.is_file())
                .collect();
            files.sort();
            for file in files {
                overrides.push(load_override(&file)?);
            }
        } else if is_tera_file(path) && path.is_file() {
            overrides.push(load_override(path)?);
        }
    }
    Ok(overrides)
}

/// Returns true if the path has a `.tera` extension.
fn is_tera_file(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext == "tera")
}

/// Builds an [`SaraError::InvalidConfig`] for a template path.
fn invalid_config(path: &Path, reason: &str) -> SaraError {
    SaraError::InvalidConfig {
        path: path.to_path_buf(),
        reason: reason.to_string(),
    }
}

/// Reads and validates one override template from disk.
fn load_override(path: &Path) -> Result<TemplateOverride, SaraError> {
    let type_id = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| invalid_config(path, "file name is not valid UTF-8"))?
        .to_string();
    let source = fs::read_to_string(path).map_err(|e| invalid_config(path, &e.to_string()))?;

    // Compile against a probe engine so syntax errors carry the file path.
    let mut probe = Tera::default();
    probe
        .add_raw_template(FRONTMATTER_TEMPLATE, FRONTMATTER_SOURCE)
        .expect("Failed to load embedded frontmatter template");
    probe
        .add_raw_template(&format!("{type_id}.tera"), &source)
        .map_err(|e| invalid_config(path, &e.to_string()))?;

    Ok(TemplateOverride { type_id, source })
}

/// Holds the Tera engine and the type id to document template lookup map.
struct TemplateRegistry {
    tera: Tera,
    documents: HashMap<String, String>,
}

/// Global template registry, lazily initialized.
static REGISTRY: OnceLock<TemplateRegistry> = OnceLock::new();

/// Returns the global template registry, initializing on first call.
///
/// Built-in document bodies are registered first, then installed overrides,
/// so an override replaces the built-in body for the same type id.
fn get_registry() -> &'static TemplateRegistry {
    REGISTRY.get_or_init(|| {
        let mut tera = Tera::default();
        let mut documents = HashMap::new();

        tera.add_raw_template(FRONTMATTER_TEMPLATE, FRONTMATTER_SOURCE)
            .expect("Failed to load embedded frontmatter template");
        tera.add_raw_template(GENERIC_TEMPLATE, GENERIC_SOURCE)
            .expect("Failed to load embedded generic template");

        let builtin = BUILTIN_DOCUMENTS
            .iter()
            .map(|(type_id, source)| ((*type_id).to_string(), (*source).to_string()));
        let overridden = OVERRIDES
            .get()
            .into_iter()
            .flatten()
            .map(|o| (o.type_id.clone(), o.source.clone()));

        for (type_id, source) in builtin.chain(overridden) {
            let name = format!("{type_id}.tera");
            tera.add_raw_template(&name, &source)
                .expect("Failed to load document template");
            documents.insert(type_id, name);
        }

        TemplateRegistry { tera, documents }
    })
}

/// Generates a complete Markdown document (frontmatter + body) from an `Item`.
#[must_use]
pub fn generate_document(item: &Item) -> String {
    let registry = get_registry();
    let context = build_context(item);
    let template = registry
        .documents
        .get(item.item_type.as_str())
        .map_or(GENERIC_TEMPLATE, String::as_str);
    registry
        .tera
        .render(template, &context)
        .expect("Failed to render document template")
}

/// Renders only the YAML frontmatter block from an `Item`.
#[must_use]
pub fn generate_frontmatter(item: &Item) -> String {
    let registry = get_registry();
    let context = build_context(item);
    registry
        .tera
        .render(FRONTMATTER_TEMPLATE, &context)
        .expect("Failed to render frontmatter template")
}

/// One frontmatter line group prepared for the generic template.
///
/// `kind` selects the rendering: `"scalar"` renders `name: "value"`, `"raw"`
/// renders `name: value` (enum and date identifiers), and `"list"` renders a
/// block sequence of quoted `values`.
#[derive(Debug, Serialize)]
struct FrontmatterEntry {
    name: String,
    kind: &'static str,
    value: String,
    values: Vec<String>,
}

impl FrontmatterEntry {
    /// Creates a quoted scalar entry.
    fn scalar(name: &str, value: String) -> Self {
        Self {
            name: name.to_string(),
            kind: "scalar",
            value,
            values: Vec::new(),
        }
    }

    /// Creates an unquoted scalar entry.
    fn raw(name: &str, value: String) -> Self {
        Self {
            name: name.to_string(),
            kind: "raw",
            value,
            values: Vec::new(),
        }
    }

    /// Creates a block sequence entry.
    fn list(name: &str, values: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            kind: "list",
            value: String::new(),
            values,
        }
    }
}

/// Field metadata exposed to document templates as `fields`.
#[derive(Debug, Serialize)]
struct FieldMeta {
    name: String,
    display_name: String,
    kind: &'static str,
    required: bool,
}

/// Returns the template-facing kind identifier for a field type.
fn field_kind(field_type: &FieldType) -> &'static str {
    match field_type {
        FieldType::Text => "text",
        FieldType::Enum { .. } => "enum",
        FieldType::ItemRef => "item_ref",
        FieldType::List(_) => "list",
        FieldType::Date => "date",
    }
}

/// Builds a Tera context from an `Item`, driven by the active schema.
///
/// Inserts the core fields (`id`, `type`, `name`, `description`), then one
/// value per declared field or relation of the item's type — both at the top
/// level under its own name (for document bodies) and in the ordered
/// `entries` sequence consumed by the generic frontmatter template. Also
/// exposes `display_name` and the `fields` metadata used by the generic body.
fn build_context(item: &Item) -> Context {
    let mut context = Context::new();
    let type_id = item.item_type.as_str();

    context.insert(FieldName::Id.as_str(), item.id.as_str());
    context.insert(FieldName::Type.as_str(), type_id);
    context.insert(FieldName::Name.as_str(), &escape_yaml_string(&item.name));
    if let Some(ref desc) = item.description {
        context.insert(FieldName::Description.as_str(), &escape_yaml_string(desc));
    }

    let Some(def) = schema::item_type_def(type_id) else {
        return context;
    };

    context.insert("display_name", def.display_name.as_str());
    let field_meta: Vec<FieldMeta> = def
        .fields
        .iter()
        .map(|f| FieldMeta {
            name: f.name.clone(),
            display_name: f.display_name.clone(),
            kind: field_kind(&f.field_type),
            required: f.required,
        })
        .collect();
    context.insert("fields", &field_meta);

    let mut entries: Vec<FrontmatterEntry> = Vec::new();

    // Declared fields first, then declared relations in declaration order.
    for field in &def.fields {
        if let Some(entry) = field_entry(item, field) {
            entries.push(entry);
        }
    }

    // Upstream and peer relations read the item's relationships; downstream
    // relations are derived and never declared as primary.
    for target in &def.allowed_targets {
        let Some(rel) = schema::relation_def(&target.relation) else {
            continue;
        };
        if rel.direction == RelationDirection::Downstream {
            continue;
        }
        let Some(rel_type) = RelationshipType::from_id(&target.relation) else {
            continue;
        };
        let ids: Vec<String> = item
            .relationship_ids(rel_type)
            .map(|id| id.as_str().to_string())
            .collect();
        if !ids.is_empty() {
            entries.push(FrontmatterEntry::list(&target.relation, ids));
        }
    }

    for entry in &entries {
        if entry.kind == "list" {
            context.insert(&entry.name, &entry.values);
        } else {
            context.insert(&entry.name, &entry.value);
        }
    }
    context.insert("entries", &entries);

    context
}

/// Builds the frontmatter entry for one declared field, if the item holds a
/// renderable value for it. Empty lists are omitted.
fn field_entry(item: &Item, field: &FieldDef) -> Option<FrontmatterEntry> {
    let value = item.attributes.get(&field.name)?;
    match &field.field_type {
        FieldType::Text => value
            .as_text()
            .map(|s| FrontmatterEntry::scalar(&field.name, escape_yaml_string(s))),
        FieldType::Enum { .. } => value
            .as_enum()
            .map(|s| FrontmatterEntry::raw(&field.name, s.clone())),
        FieldType::Date => value
            .as_date()
            .map(|s| FrontmatterEntry::raw(&field.name, s.clone())),
        FieldType::ItemRef => value
            .as_item_ref()
            .map(|id| FrontmatterEntry::scalar(&field.name, id.as_str().to_string())),
        FieldType::List(_) => {
            let values: Vec<String> = value
                .as_list()?
                .iter()
                .filter_map(list_item_string)
                .collect();
            if values.is_empty() {
                None
            } else {
                Some(FrontmatterEntry::list(&field.name, values))
            }
        }
    }
}

/// Renders one list element as its YAML string form. Nested lists are not
/// representable in frontmatter and yield `None`.
fn list_item_string(value: &FieldValue) -> Option<String> {
    match value {
        FieldValue::Text(s) => Some(escape_yaml_string(s)),
        FieldValue::Enum(s) | FieldValue::Date(s) => Some(s.clone()),
        FieldValue::ItemRef(id) => Some(id.as_str().to_string()),
        FieldValue::List(_) => None,
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
    use std::path::PathBuf;

    use super::*;
    use crate::model::SourceLocation;
    use crate::model::{AdrStatus, ItemBuilder, ItemId, ItemType, Relationship, RelationshipType};

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
            .item_type(ItemType::SOLUTION)
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
            .item_type(ItemType::USE_CASE)
            .name("Test Use Case")
            .source(test_source())
            .relationships(vec![Relationship::new(
                ItemId::new_unchecked("SOL-001"),
                RelationshipType::REFINES,
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
            .item_type(ItemType::SYSTEM_ARCHITECTURE)
            .name("Web Platform Architecture")
            .source(test_source())
            .platform("AWS Lambda")
            .relationships(vec![Relationship::new(
                ItemId::new_unchecked("SYSREQ-001"),
                RelationshipType::SATISFIES,
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
            .item_type(ItemType::ARCHITECTURE_DECISION_RECORD)
            .name("Use Microservices Architecture")
            .description("Decision to adopt microservices")
            .source(test_source())
            .status(AdrStatus::Proposed)
            .deciders(vec!["Alice Smith".to_string(), "Bob Jones".to_string()])
            .relationships(vec![Relationship::new(
                ItemId::new_unchecked("SYSARCH-001"),
                RelationshipType::JUSTIFIES,
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
            .item_type(ItemType::SOLUTION)
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
            .item_type(ItemType::SYSTEM_REQUIREMENT)
            .name("Performance Requirement")
            .source(test_source())
            .specification("The system SHALL respond within 200ms.")
            .relationships(vec![Relationship::new(
                ItemId::new_unchecked("SCEN-001"),
                RelationshipType::DERIVES_FROM,
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

    #[test]
    fn test_frontmatter_entry_order_matches_legacy_layout() {
        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("SYSREQ-002"))
            .item_type(ItemType::SYSTEM_REQUIREMENT)
            .name("Ordered")
            .source(test_source())
            .specification("Spec.")
            .depends_on(ItemId::new_unchecked("SYSREQ-001"))
            .relationships(vec![Relationship::new(
                ItemId::new_unchecked("SCEN-001"),
                RelationshipType::DERIVES_FROM,
            )])
            .build()
            .unwrap();

        let fm = generate_frontmatter(&item);

        let spec = fm.find("specification:").unwrap();
        let derives = fm.find("derives_from:").unwrap();
        let depends = fm.find("depends_on:").unwrap();
        assert!(spec < derives && derives < depends);
    }

    #[test]
    fn test_generic_body_renders_declared_text_fields() {
        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("SYSREQ-003"))
            .item_type(ItemType::SYSTEM_REQUIREMENT)
            .name("Fallback")
            .source(test_source())
            .specification("Spec.")
            .build()
            .unwrap();

        let registry = get_registry();
        let doc = registry
            .tera
            .render(GENERIC_TEMPLATE, &build_context(&item))
            .unwrap();

        assert!(doc.contains("# System Requirement: Fallback"));
        assert!(doc.contains("## Overview"));
        assert!(doc.contains("## Specification"));
        assert!(doc.contains("specification: \"Spec.\""));
    }

    #[test]
    fn test_discover_overrides_from_directory() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("use_case.tera"),
            "{% include \"frontmatter.tera\" %}\n\n# Custom: {{ name }}",
        )
        .unwrap();
        std::fs::write(dir.path().join("notes.md"), "not a template").unwrap();

        let config = TemplatesConfig {
            paths: vec![dir.path().to_string_lossy().into_owned()],
        };
        let overrides = discover_overrides(&config).unwrap();

        assert_eq!(overrides.len(), 1);
        assert_eq!(overrides[0].type_id, "use_case");
        assert!(overrides[0].source.contains("# Custom:"));
    }

    #[test]
    fn test_discover_overrides_direct_file_and_missing_paths() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("scenario.tera");
        std::fs::write(&file, "# {{ name }}").unwrap();

        let config = TemplatesConfig {
            paths: vec![
                file.to_string_lossy().into_owned(),
                "does/not/exist.tera".to_string(),
                "*.md".to_string(),
            ],
        };
        let overrides = discover_overrides(&config).unwrap();

        assert_eq!(overrides.len(), 1);
        assert_eq!(overrides[0].type_id, "scenario");
    }

    #[test]
    fn test_discover_overrides_rejects_invalid_template() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("broken.tera");
        std::fs::write(&file, "{% if %}").unwrap();

        let config = TemplatesConfig {
            paths: vec![file.to_string_lossy().into_owned()],
        };

        assert!(discover_overrides(&config).is_err());
    }
}

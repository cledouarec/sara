# Data Model: Requirements Knowledge Graph CLI

**Phase**: 1 - Design
**Date**: 2026-01-11
**Branch**: `001-requirements-knowledge-graph`

## Overview

This document defines the core data structures for the Requirements Knowledge Graph. All entities are defined as Rust types with serde support for serialization.

## Entity Hierarchy

**Upstream to Downstream (is_refined_by / derives / is_satisfied_by):**
```
Solution
  └── is_refined_by → Use Case
                        └── is_refined_by → Scenario
                                              └── derives → System Requirement
                                                              └── is_satisfied_by → System Architecture
                                                                                      ├── derives → Hardware Requirement
                                                                                      │               └── is_satisfied_by → Hardware Detailed Design
                                                                                      └── derives → Software Requirement
                                                                                                      └── is_satisfied_by → Software Detailed Design
```

**Downstream to Upstream (refines / derives_from / satisfies):**
```
Solution
  └── refines ← Use Case
                  └── refines ← Scenario
                                  └── derives_from ← System Requirement
                                                       └── satisfies ← System Architecture
                                                                         ├── derives_from ← Hardware Requirement
                                                                         │                    └── satisfies ← Hardware Detailed Design
                                                                         └── derives_from ← Software Requirement
                                                                                              └── satisfies ← Software Detailed Design
```

## Core Entities

### ItemType (Enum)

Represents the type of item in the knowledge graph.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemType {
    Solution,
    UseCase,
    Scenario,
    SystemRequirement,
    SystemArchitecture,
    HardwareRequirement,
    SoftwareRequirement,
    HardwareDetailedDesign,
    SoftwareDetailedDesign,
}
```

**Validation Rules**:
- Must be one of the 9 defined types
- Type determines allowed relationships (see Relationship Rules)

---

### ItemId (Value Object)

Unique identifier for an item across all repositories.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemId(String);

impl ItemId {
    /// Creates a new ItemId, validating format
    pub fn new(id: impl Into<String>) -> Result<Self, ValidationError>;

    /// Returns the raw identifier string
    pub fn as_str(&self) -> &str;
}
```

**Validation Rules**:
- Must be non-empty
- Must contain only alphanumeric characters, hyphens, and underscores
- Recommended format: `{TYPE_PREFIX}-{NUMBER}` (e.g., "SOL-001", "UC-001", "SREQ-001")
- Must be unique across all repositories (FR-012)

**Common Prefixes** (convention, not enforced):
| Type | Prefix |
|------|--------|
| Solution | SOL |
| Use Case | UC |
| Scenario | SCEN |
| System Requirement | SREQ |
| System Architecture | SARCH |
| Hardware Requirement | HWREQ |
| Software Requirement | SWREQ |
| Hardware Detailed Design | HWDD |
| Software Detailed Design | SWDD |

---

### Item (Entity)

Represents a single document/node in the knowledge graph.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    /// Unique identifier
    pub id: ItemId,

    /// Type of this item
    pub item_type: ItemType,

    /// Human-readable name
    pub name: String,

    /// Optional description
    pub description: Option<String>,

    /// Source file location
    pub source: SourceLocation,

    /// Upstream relationships (toward Solution)
    pub upstream: UpstreamRefs,

    /// Downstream relationships (toward Detailed Designs)
    pub downstream: DownstreamRefs,

    /// Type-specific attributes
    pub attributes: ItemAttributes,
}

/// Upstream relationship references (this item points to parents)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpstreamRefs {
    /// Items this item refines (for UseCase, Scenario)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refines: Vec<ItemId>,

    /// Items this item derives from (for SystemRequirement, HW/SW Requirement)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub derives_from: Vec<ItemId>,

    /// Items this item satisfies (for SystemArchitecture, HW/SW DetailedDesign)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub satisfies: Vec<ItemId>,
}

/// Downstream relationship references (this item points to children)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DownstreamRefs {
    /// Items that refine this item (for Solution, UseCase)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub is_refined_by: Vec<ItemId>,

    /// Items derived from this item (for Scenario, SystemArchitecture)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub derives: Vec<ItemId>,

    /// Items that satisfy this item (for SystemRequirement, HW/SW Requirement)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub is_satisfied_by: Vec<ItemId>,
}
```

**Relationships**:
- **Upstream** (toward Solution): `refines`, `derives_from`, `satisfies`
- **Downstream** (toward Detailed Designs): `is_refined_by`, `derives`, `is_satisfied_by`

---

### SourceLocation (Value Object)

Tracks the file origin of an item for error reporting.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    /// Repository path (absolute)
    pub repository: PathBuf,

    /// Relative path within repository
    pub file_path: PathBuf,

    /// Line number where item definition starts (1-indexed)
    pub line: usize,

    /// Optional Git commit/branch if reading from history
    pub git_ref: Option<String>,
}

impl SourceLocation {
    /// Format as "path/to/file.md:42"
    pub fn display(&self) -> String;
}
```

---

### ItemAttributes (Type-Specific Data)

Additional fields depending on item type.

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ItemAttributes {
    /// For SystemRequirement, HardwareRequirement, SoftwareRequirement: specification statement (mandatory)
    pub specification: Option<String>,

    /// For SystemRequirement, HardwareRequirement, SoftwareRequirement: peer dependencies
    /// Stored as depends_on in YAML, inverse relationship is_required_by is computed
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub depends_on: Vec<ItemId>,

    /// For SystemArchitecture: target platform
    pub platform: Option<String>,

    /// For SystemArchitecture: reserved for future ADR (Architecture Decision Record) links
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub justified_by: Option<Vec<ItemId>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationMethod {
    Test,
    Analysis,
    Inspection,
    Demonstration,
}
```

---

### RelationshipType (Enum)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipType {
    /// Refinement: child refines parent (Scenario refines Use Case)
    Refines,
    /// Inverse of Refines: parent is refined by child (Use Case is_refined_by Scenario)
    IsRefinedBy,
    /// Derivation: parent derives child (Scenario derives System Requirement)
    Derives,
    /// Inverse of Derives: child derives from parent (System Requirement derives_from Scenario)
    DerivesFrom,
    /// Satisfaction: child satisfies parent (System Architecture satisfies System Requirement)
    Satisfies,
    /// Inverse of Satisfies: parent is satisfied by child (System Requirement is_satisfied_by System Architecture)
    IsSatisfiedBy,
    /// Dependency: Requirement depends on another Requirement of the same type
    DependsOn,
    /// Inverse of DependsOn: Requirement is required by another
    IsRequiredBy,
}

impl RelationshipType {
    /// Get the inverse relationship type
    pub fn inverse(&self) -> Self {
        match self {
            Self::Refines => Self::IsRefinedBy,
            Self::IsRefinedBy => Self::Refines,
            Self::Derives => Self::DerivesFrom,
            Self::DerivesFrom => Self::Derives,
            Self::Satisfies => Self::IsSatisfiedBy,
            Self::IsSatisfiedBy => Self::Satisfies,
            Self::DependsOn => Self::IsRequiredBy,
            Self::IsRequiredBy => Self::DependsOn,
        }
    }

    /// Check if this is an upstream relationship (toward Solution)
    pub fn is_upstream(&self) -> bool {
        matches!(self, Self::Refines | Self::DerivesFrom | Self::Satisfies)
    }

    /// Check if this is a downstream relationship (toward Detailed Designs)
    pub fn is_downstream(&self) -> bool {
        matches!(self, Self::IsRefinedBy | Self::Derives | Self::IsSatisfiedBy)
    }

    /// Check if this is a peer relationship (between items of the same type)
    pub fn is_peer(&self) -> bool {
        matches!(self, Self::DependsOn | Self::IsRequiredBy)
    }
}
```

---

### Relationship (Edge)

Represents a link between two items in the graph.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    /// Source item ID
    pub from: ItemId,

    /// Target item ID
    pub to: ItemId,

    /// Type of relationship
    pub relationship_type: RelationshipType,
}
```

---

## Relationship Rules

Valid relationships based on item types (FR-006, FR-007, FR-008):

**Downstream direction (from parent to child):**

| From Type | Relationship | To Type(s) |
|-----------|--------------|------------|
| Solution | is_refined_by | UseCase |
| UseCase | is_refined_by | Scenario |
| Scenario | derives | SystemRequirement |
| SystemRequirement | is_satisfied_by | SystemArchitecture |
| SystemArchitecture | derives | HardwareRequirement, SoftwareRequirement |
| HardwareRequirement | is_satisfied_by | HardwareDetailedDesign |
| SoftwareRequirement | is_satisfied_by | SoftwareDetailedDesign |

**Upstream direction (from child to parent):**

| From Type | Relationship | To Type(s) |
|-----------|--------------|------------|
| UseCase | refines | Solution |
| Scenario | refines | UseCase |
| SystemRequirement | derives_from | Scenario |
| SystemArchitecture | satisfies | SystemRequirement |
| HardwareRequirement | derives_from | SystemArchitecture |
| SoftwareRequirement | derives_from | SystemArchitecture |
| HardwareDetailedDesign | satisfies | HardwareRequirement |
| SoftwareDetailedDesign | satisfies | SoftwareRequirement |

**Peer relationships (between items of the same type):**

| From Type | Relationship | To Type(s) |
|-----------|--------------|------------|
| SystemRequirement | depends_on | SystemRequirement |
| HardwareRequirement | depends_on | HardwareRequirement |
| SoftwareRequirement | depends_on | SoftwareRequirement |

**Validation**:
- Relationships not in these tables are invalid
- Circular references are always errors (FR-017)
- Both directions are automatically maintained (adding `refines` also creates `is_refined_by`, adding `depends_on` creates `is_required_by`)
- Redundant declarations (both items explicitly declaring the same relationship) produce a warning (FR-067); only one side needs to declare the link

---

## Graph Structure

### KnowledgeGraph

The main graph container using Petgraph.

```rust
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

pub struct KnowledgeGraph {
    /// The underlying directed graph
    graph: DiGraph<Item, RelationshipType>,

    /// Index for O(1) lookup by ItemId
    index: HashMap<ItemId, NodeIndex>,

    /// Source repositories
    repositories: Vec<PathBuf>,

    /// Validation mode
    strict_mode: bool,
}

impl KnowledgeGraph {
    /// Get item by ID
    pub fn get(&self, id: &ItemId) -> Option<&Item>;

    /// Get all items of a specific type
    pub fn items_by_type(&self, item_type: ItemType) -> Vec<&Item>;

    /// Traverse upstream (toward Solution)
    pub fn upstream(&self, id: &ItemId) -> Vec<&Item>;

    /// Traverse downstream (toward Detailed Designs)
    pub fn downstream(&self, id: &ItemId) -> Vec<&Item>;

    /// Get direct parents (items that this item requires/realizes)
    pub fn parents(&self, id: &ItemId) -> Vec<&Item>;

    /// Get direct children (items that require/realize this item)
    pub fn children(&self, id: &ItemId) -> Vec<&Item>;

    /// Check for cycles
    pub fn has_cycles(&self) -> bool;

    /// Get all items with no upstream parents
    pub fn orphans(&self) -> Vec<&Item>;
}
```

---

## Builder Patterns

### ItemBuilder

```rust
pub struct ItemBuilder {
    id: Option<ItemId>,
    item_type: Option<ItemType>,
    name: Option<String>,
    description: Option<String>,
    source: Option<SourceLocation>,
    requires: Vec<ItemId>,
    realizes: Vec<ItemId>,
    attributes: ItemAttributes,
}

impl ItemBuilder {
    pub fn new(id: impl Into<String>, item_type: ItemType) -> Self;
    pub fn name(self, name: impl Into<String>) -> Self;
    pub fn description(self, desc: impl Into<String>) -> Self;
    pub fn source(self, source: SourceLocation) -> Self;
    pub fn requires(self, ids: Vec<impl Into<String>>) -> Self;
    pub fn realizes(self, ids: Vec<impl Into<String>>) -> Self;
    pub fn verification_method(self, method: VerificationMethod) -> Self;
    pub fn requirement_text(self, text: impl Into<String>) -> Self;
    pub fn platform(self, platform: impl Into<String>) -> Self;
    pub fn build(self) -> Result<Item, BuilderError>;
}
```

### GraphBuilder

```rust
pub struct GraphBuilder {
    repositories: Vec<PathBuf>,
    strict_mode: bool,
    git_ref: Option<String>,
}

impl GraphBuilder {
    pub fn new() -> Self;
    pub fn add_repository(self, path: impl AsRef<Path>) -> Result<Self, IoError>;
    pub fn with_strict_orphan_check(self, strict: bool) -> Self;
    pub fn at_git_ref(self, git_ref: impl Into<String>) -> Self;
    pub fn build(self) -> Result<KnowledgeGraph, BuildError>;
}
```

---

## Validation Types

### ValidationError

```rust
#[derive(Debug, Clone, Serialize)]
pub struct ValidationError {
    pub severity: Severity,
    pub code: ErrorCode,
    pub message: String,
    pub location: Option<SourceLocation>,
    pub related: Vec<ItemId>,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum Severity {
    Error,   // Blocks validation
    Warning, // Informational (e.g., orphans in permissive mode)
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum ErrorCode {
    BrokenReference,     // FR-010
    OrphanItem,          // FR-011
    DuplicateIdentifier, // FR-012
    CircularReference,   // FR-013
    InvalidMetadata,     // FR-014
    InvalidRelationship, // FR-006, FR-007, FR-008
}
```

### ValidationReport

```rust
pub struct ValidationReport {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationError>,
    pub items_checked: usize,
    pub relationships_checked: usize,
    pub duration: Duration,
}

impl ValidationReport {
    pub fn is_valid(&self) -> bool;
    pub fn error_count(&self) -> usize;
    pub fn warning_count(&self) -> usize;
}
```

---

## Configuration Types

### Config

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub repositories: RepositoryConfig,
    pub validation: ValidationConfig,
    pub output: OutputConfig,
    #[serde(default)]
    pub templates: TemplatesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryConfig {
    pub paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    #[serde(default)]
    pub strict_orphans: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    #[serde(default = "default_true")]
    pub colors: bool,
    #[serde(default = "default_true")]
    pub emojis: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TemplatesConfig {
    /// Paths to custom template files (supports glob patterns, e.g., "*.tera", "*.md")
    /// Each template must contain exactly one 'type' field in its YAML frontmatter
    /// to identify the item type it defines
    /// Custom templates override built-in Tera templates for the corresponding item type
    /// Built-in templates are embedded in the binary via `include_str!` and use the Tera engine
    #[serde(default)]
    pub paths: Vec<String>,
}
```

---

## YAML Frontmatter Schema

Expected frontmatter format in Markdown files:

```yaml
---
id: "REQ-001"
type: system_requirement
name: "Performance Requirement"
description: "Optional longer description"
requires:
  - "SCEN-001"
  - "SCEN-002"
realizes: []
verification_method: test
requirement_text: "The system SHALL respond within 100ms"
---
```

**Required Fields**:
- `id`: Unique identifier
- `type`: One of the ItemType values
- `name`: Human-readable name

**Optional Fields**:
- `description`: Extended description
- `requires`: List of required item IDs
- `realizes`: List of realized item IDs
- Type-specific fields (verification_method, requirement_text, platform)

---

## State Transitions

Items are immutable once loaded. The graph itself has the following states:

```
                    ┌──────────────────┐
                    │     Empty        │
                    └────────┬─────────┘
                             │ add_repository()
                             ▼
                    ┌──────────────────┐
                    │    Building      │◄────┐
                    └────────┬─────────┘     │ add_repository()
                             │ build()       │
                             ▼               │
                    ┌──────────────────┐     │
                    │    Validating    │─────┘ (if error, can add more repos)
                    └────────┬─────────┘
                             │ validation complete
                             ▼
              ┌──────────────┴──────────────┐
              ▼                              ▼
    ┌──────────────────┐          ┌──────────────────┐
    │      Valid       │          │     Invalid      │
    │  (ready for use) │          │ (contains errors)│
    └──────────────────┘          └──────────────────┘
```

---

## Interactive Mode Types (Added 2026-01-14)

This section documents types for FR-040 through FR-052 (Interactive Mode).

### InteractiveSession

Holds state for an interactive document creation session.

```rust
use sara_core::{ItemType, KnowledgeGraph};

/// Configuration for an interactive init session
pub struct InteractiveSession {
    /// Pre-parsed knowledge graph for traceability lookups
    pub graph: Option<KnowledgeGraph>,

    /// Output configuration (colors, emojis)
    pub output_config: OutputConfig,

    /// Pre-provided fields from CLI arguments (skip prompts for these)
    pub prefilled: PrefilledFields,
}

/// Fields pre-provided via CLI arguments (FR-050)
#[derive(Debug, Default)]
pub struct PrefilledFields {
    pub file: Option<PathBuf>,
    pub item_type: Option<ItemType>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub refines: Vec<String>,
    pub derives_from: Vec<String>,
    pub satisfies: Vec<String>,
    pub specification: Option<String>,
    pub platform: Option<String>,
}
```

---

### InteractiveResult

The result of an interactive session.

```rust
/// Result of an interactive session
pub enum InteractiveResult {
    /// User completed the session with valid input
    Completed(InteractiveInput),

    /// User cancelled (Ctrl+C or explicit cancel)
    Cancelled,

    /// Non-TTY environment detected
    NonInteractive,

    /// Required parent items don't exist
    MissingParents {
        item_type: ItemType,
        required_parent_type: ItemType
    },
}

/// Collected input from interactive session
#[derive(Debug)]
pub struct InteractiveInput {
    pub file: PathBuf,
    pub item_type: ItemType,
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub traceability: TraceabilityInput,
    pub type_specific: TypeSpecificInput,
}

/// Traceability links selected by user
#[derive(Debug, Default)]
pub struct TraceabilityInput {
    /// For UseCase/Scenario
    pub refines: Vec<String>,
    /// For Requirements
    pub derives_from: Vec<String>,
    /// For Architectures/Designs
    pub satisfies: Vec<String>,
}

/// Type-specific fields
#[derive(Debug, Default)]
pub struct TypeSpecificInput {
    /// For requirement types
    pub specification: Option<String>,
    /// For SystemArchitecture
    pub platform: Option<String>,
}
```

---

### PromptError

Error types for interactive prompts.

```rust
/// Errors that can occur during interactive prompts
#[derive(Debug, thiserror::Error)]
pub enum PromptError {
    #[error("Interactive mode requires a terminal. Use --type <TYPE> to specify the item type.")]
    NonInteractiveTerminal,

    #[error("Cannot create {item_type}: no {parent_type} items exist. Create a {parent_type} first.")]
    MissingParentItems {
        item_type: String,
        parent_type: String,
    },

    #[error("User cancelled")]
    Cancelled,

    #[error("Prompt error: {0}")]
    InquireError(#[from] inquire::InquireError),

    #[error("Failed to parse graph: {0}")]
    GraphError(String),
}
```

---

### ParentRequirements

Lookup table for parent item type requirements (FR-052).

```rust
impl ItemType {
    /// Returns the required parent item type for this type, if any.
    /// Solution has no parent (returns None).
    pub fn required_parent_type(&self) -> Option<ItemType> {
        match self {
            ItemType::Solution => None,
            ItemType::UseCase => Some(ItemType::Solution),
            ItemType::Scenario => Some(ItemType::UseCase),
            ItemType::SystemRequirement => Some(ItemType::Scenario),
            ItemType::SystemArchitecture => Some(ItemType::SystemRequirement),
            ItemType::HardwareRequirement => Some(ItemType::SystemArchitecture),
            ItemType::SoftwareRequirement => Some(ItemType::SystemArchitecture),
            ItemType::HardwareDetailedDesign => Some(ItemType::HardwareRequirement),
            ItemType::SoftwareDetailedDesign => Some(ItemType::SoftwareRequirement),
        }
    }

    /// Returns the relationship field name for upstream traceability.
    pub fn traceability_field(&self) -> Option<&'static str> {
        match self {
            ItemType::Solution => None,
            ItemType::UseCase | ItemType::Scenario => Some("refines"),
            ItemType::SystemRequirement
            | ItemType::HardwareRequirement
            | ItemType::SoftwareRequirement => Some("derives_from"),
            ItemType::SystemArchitecture
            | ItemType::HardwareDetailedDesign
            | ItemType::SoftwareDetailedDesign => Some("satisfies"),
        }
    }
}
```

---

### SelectOption

Display format for items in select lists.

```rust
/// Option displayed in Select/MultiSelect prompts
#[derive(Debug, Clone)]
pub struct SelectOption {
    /// Item ID (e.g., "SOL-001")
    pub id: String,
    /// Item name for display
    pub name: String,
    /// Item type for grouping/filtering
    pub item_type: ItemType,
}

impl std::fmt::Display for SelectOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.id, self.name)
    }
}

impl SelectOption {
    /// Create from a graph Item
    pub fn from_item(item: &Item) -> Self {
        Self {
            id: item.id.as_str().to_string(),
            name: item.name.clone(),
            item_type: item.item_type,
        }
    }
}
```

---

## Edit Command Types (Added 2026-01-14)

This section documents types for FR-054 through FR-066 (Edit Command).

### EditSession

Holds state for an interactive edit session.

```rust
/// Configuration for an interactive edit session
pub struct EditSession {
    /// The item being edited
    pub item: Item,

    /// Pre-parsed knowledge graph for traceability lookups
    pub graph: KnowledgeGraph,

    /// Output configuration (colors, emojis)
    pub output_config: OutputConfig,

    /// Fields to update from CLI arguments (non-interactive mode)
    pub updates: EditUpdates,
}

/// Fields to update (from CLI flags for non-interactive mode)
#[derive(Debug, Default)]
pub struct EditUpdates {
    pub name: Option<String>,
    pub description: Option<String>,
    pub refines: Option<Vec<String>>,
    pub derives_from: Option<Vec<String>>,
    pub satisfies: Option<Vec<String>>,
    pub specification: Option<String>,
    pub platform: Option<String>,
}

impl EditUpdates {
    /// Returns true if any field is set (non-interactive mode)
    pub fn has_updates(&self) -> bool {
        self.name.is_some()
            || self.description.is_some()
            || self.refines.is_some()
            || self.derives_from.is_some()
            || self.satisfies.is_some()
            || self.specification.is_some()
            || self.platform.is_some()
    }
}
```

---

### EditResult

The result of an edit operation.

```rust
/// Result of an edit session
pub enum EditResult {
    /// Changes applied successfully
    Applied(EditSummary),

    /// User cancelled (Ctrl+C or explicit cancel)
    Cancelled,

    /// Item not found
    NotFound {
        id: String,
        suggestions: Vec<String>,
    },

    /// Non-TTY environment detected
    NonInteractive,
}

/// Summary of changes made
#[derive(Debug)]
pub struct EditSummary {
    pub item_id: String,
    pub file_path: PathBuf,
    pub changes: Vec<FieldChange>,
}

/// A single field change
#[derive(Debug)]
pub struct FieldChange {
    pub field: String,
    pub old_value: String,
    pub new_value: String,
}

impl FieldChange {
    pub fn is_changed(&self) -> bool {
        self.old_value != self.new_value
    }
}
```

---

### EditError

Error types for edit operations.

```rust
/// Errors that can occur during edit operations
#[derive(Debug, thiserror::Error)]
pub enum EditError {
    #[error("Item not found: {id}")]
    ItemNotFound {
        id: String,
        #[source]
        suggestions: Vec<String>,
    },

    #[error("Interactive mode requires a terminal. Use modification flags (--name, --description, etc.) to edit non-interactively.")]
    NonInteractiveTerminal,

    #[error("User cancelled")]
    Cancelled,

    #[error("Invalid traceability link: {id} does not exist")]
    InvalidLink { id: String },

    #[error("Prompt error: {0}")]
    InquireError(#[from] inquire::InquireError),

    #[error("Failed to read file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse graph: {0}")]
    GraphError(String),
}

impl EditError {
    /// Format suggestions for "not found" error
    pub fn format_suggestions(&self) -> Option<String> {
        if let EditError::ItemNotFound { suggestions, .. } = self {
            if !suggestions.is_empty() {
                return Some(format!("Did you mean: {}?", suggestions.join(", ")));
            }
        }
        None
    }
}
```

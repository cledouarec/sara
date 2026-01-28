# Quickstart: ADR Management Implementation

**Feature**: 002-adr-management
**Date**: 2026-01-26

## Prerequisites

- Rust 1.75+ installed
- Familiarity with SARA codebase structure
- Read [research.md](research.md) and [data-model.md](data-model.md)

## Development Setup

```bash
# Clone and enter repo (if not already)
cd /home/christophe/Projects/sara

# Ensure on feature branch
git checkout 002-adr-management

# Build and test existing code
cargo build
cargo test
cargo clippy
```

## Implementation Order

Follow this order to minimize compilation errors and enable incremental testing:

### Step 1: Model Layer (sara-core/src/model/)

**Files**: `item.rs`, `relationship.rs`, `field.rs`

1. Add `AdrStatus` enum to `item.rs`
2. Add `AdrAttributes` struct to `item.rs`
3. Add `ArchitectureDecisionRecord` variant to `ItemType`
4. Implement `ItemType` methods for ADR (`prefix()`, `display_name()`, etc.)
5. Add `Justifies`/`IsJustifiedBy`, `Supersedes`/`IsSupersededBy` to `RelationshipType`
6. Implement `inverse()` for new relationship types
7. Add new `FieldName` variants for ADR fields

**Verify**:
```bash
cargo check -p sara-core
cargo test -p sara-core model::
```

### Step 2: Parser Layer (sara-core/src/parser/)

**Files**: `markdown.rs`

1. Extend `RawFrontmatter` with ADR optional fields
2. Add ADR field parsing in `parse_item()` function
3. Add validation for ADR-specific requirements

**Verify**:
```bash
cargo test -p sara-core parser::
```

### Step 3: Graph Layer (sara-core/src/graph/)

**Files**: `builder.rs`, `knowledge_graph.rs` (minimal)

1. Extend `GraphBuilder::add_item_relationships()` for ADR
2. Handle `justifies` → `Justifies` edges
3. Handle `supersedes` → `Supersedes` edges
4. Update `justified_by` on target items (reverse direction)

**Verify**:
```bash
cargo test -p sara-core graph::
```

### Step 4: Template Layer (sara-core/src/template/)

**Files**: `generator.rs`, `templates/adr.tera`

1. Create `templates/adr.tera` template file
2. Register ADR template in `TemplateGenerator`
3. Implement template context population for ADR

**Template content** (`templates/adr.tera`):
```tera
{% include "frontmatter.tera" %}
# Architecture Decision: {{ name }}

- Deciders: {{ deciders | join(", ") }}

## Context and problem statement

[Describe the context and problem statement]

## Key factors

- [Key factor 1]
- [Key factor 2]

## Considered options

- [Option 1]
- [Option 2]

## Decision Outcome

Chosen option: "[Option X]", because [justification].

### Positive Consequences

- [Positive consequence]

### Negative Consequences

- [Negative consequence]

## Pros and Cons of the Options

### [Option 1]

- Good, because [argument]
- Bad, because [argument]

## Links

- [Reference](url)
```

**Verify**:
```bash
cargo test -p sara-core template::
```

### Step 5: CLI Layer (sara-cli/src/commands/)

**Files**: `new.rs`, `list.rs`, `show.rs`, potentially new `adr.rs`

1. Extend `new` command with `adr` subcommand
2. Extend `list` command to handle ADR type
3. Add ADR-specific formatting in `show` command
4. Add `link adr` subcommand
5. Add `query adr` subcommand

**Verify**:
```bash
cargo build -p sara-cli
./target/debug/sara new adr --help
./target/debug/sara list adr --help
```

### Step 6: Integration Tests

**Files**: `tests/integration/adr_tests.rs`

1. Create test fixtures (sample ADR markdown files)
2. Test ADR parsing from files
3. Test graph building with ADR relationships
4. Test bidirectional traversal (Justifies/IsJustifiedBy)
5. Test supersession chain traversal
6. Test validation rules

**Verify**:
```bash
cargo test -p sara-core --test '*adr*'
cargo test -p sara-cli --test '*adr*'
```

## Key Code Patterns

### Adding a New ItemType Variant

```rust
// 1. Add to enum
pub enum ItemType {
    // ...existing...
    ArchitectureDecisionRecord,
}

// 2. Implement ALL match arms
impl ItemType {
    pub fn prefix(&self) -> &'static str {
        match self {
            // ...existing...
            Self::ArchitectureDecisionRecord => "ADR",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            // ...existing...
            Self::ArchitectureDecisionRecord => "Architecture Decision Record",
        }
    }

    pub fn required_parent_type(&self) -> Option<ItemType> {
        match self {
            // ADR has no required parent
            Self::ArchitectureDecisionRecord => None,
            // ...existing...
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            // ...existing...
            "architecture_decision_record" => Some(Self::ArchitectureDecisionRecord),
            _ => None,
        }
    }
}
```

### Adding a New Relationship Type

```rust
// 1. Add to enum
pub enum RelationshipType {
    // ...existing...
    Justifies,
    IsJustifiedBy,
}

// 2. Implement inverse
impl RelationshipType {
    pub fn inverse(&self) -> Self {
        match self {
            // ...existing...
            Self::Justifies => Self::IsJustifiedBy,
            Self::IsJustifiedBy => Self::Justifies,
        }
    }
}
```

### Extending RawFrontmatter

```rust
#[derive(Debug, Deserialize)]
pub struct RawFrontmatter {
    // ...existing fields...

    // ADR fields (all optional - non-ADR items ignore these)
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub deciders: Option<Vec<String>>,
    // ...etc...
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adr_status_from_str() {
        assert_eq!(AdrStatus::from_str("proposed"), Some(AdrStatus::Proposed));
        assert_eq!(AdrStatus::from_str("invalid"), None);
    }

    #[test]
    fn test_adr_item_type_prefix() {
        assert_eq!(ItemType::ArchitectureDecisionRecord.prefix(), "ADR");
    }

    #[test]
    fn test_justifies_inverse() {
        assert_eq!(
            RelationshipType::Justifies.inverse(),
            RelationshipType::IsJustifiedBy
        );
    }
}
```

### Integration Tests

```rust
#[test]
fn test_adr_graph_building() {
    let adr_content = r#"---
id: "ADR-001"
type: architecture_decision_record
name: "Test Decision"
status: proposed
deciders: ["Alice"]
justifies: ["SYSARCH-001"]
---
# Test
"#;

    let sysarch_content = r#"---
id: "SYSARCH-001"
type: system_architecture
name: "Test Architecture"
justified_by: ["ADR-001"]
---
# Test
"#;

    // Parse and build graph
    let graph = build_test_graph(vec![adr_content, sysarch_content]);

    // Verify relationship
    let adr = graph.get(&ItemId::new("ADR-001").unwrap()).unwrap();
    let children = graph.children(&adr.id, RelationshipType::Justifies);
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].id.as_str(), "SYSARCH-001");
}
```

## Common Pitfalls

1. **Forgetting match arms**: Rust will error if ItemType/RelationshipType matches are incomplete
2. **Bidirectional consistency**: When ADR declares `justifies`, the target should have `justified_by`
3. **Serde defaults**: Use `#[serde(default)]` for optional fields to avoid parse errors
4. **Validation order**: Validate relationships exist before building graph edges

## Checklist

- [ ] `AdrStatus` enum added and tested
- [ ] `AdrAttributes` struct added
- [ ] `ItemType::ArchitectureDecisionRecord` added with all methods
- [ ] `RelationshipType::Justifies/IsJustifiedBy` added
- [ ] `RelationshipType::Supersedes/IsSupersededBy` added
- [ ] `FieldName` variants added for ADR fields
- [ ] `RawFrontmatter` extended
- [ ] `GraphBuilder` handles ADR relationships
- [ ] `adr.tera` template created
- [ ] CLI commands extended
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] `cargo clippy` clean
- [ ] Documentation updated

# Research: ADR Management with Design Linking

**Feature**: 002-adr-management
**Date**: 2026-01-26

## Overview

This document consolidates research findings for implementing ADR (Architecture Decision Record) support in the SARA knowledge graph system. All technical unknowns have been resolved through codebase analysis and user clarification.

---

## 1. ADR Template Format

### Decision
Use a custom ADR template based on MADR (Markdown Architectural Decision Records) with YAML frontmatter for metadata.

### Rationale
- Aligns with existing SARA pattern of Markdown + YAML frontmatter
- User-provided template follows industry-standard ADR structure
- Compatible with existing parser infrastructure (pulldown-cmark + serde_yaml)

### Template Structure

```yaml
---
id: "ADR-001"
type: architecture_decision_record
name: "Decision Title"
status: proposed  # proposed | accepted | deprecated | superseded
deciders:
  - "Person 1"
  - "Person 2"
justifies:
  - "SYSARCH-001"
  - "SWDD-002"
supersedes: []
superseded_by: null
---
# Architecture Decision: {{ name }}

- Deciders: {{ deciders | join(", ") }}

## Context and problem statement

[Description of the context and problem]

## Key factors

- [Key factor 1]
- [Key factor 2]

## Considered options

- [Option 1]
- [Option 2]
- [Option 3]

## Decision Outcome

Chosen option: "[Option X]", because [justification].

### Positive Consequences

- [Positive consequence 1]

### Negative Consequences

- [Negative consequence 1]

## Pros and Cons of the Options

### [Option 1]

- Good, because [argument]
- Bad, because [argument]

### [Option 2]

- Good, because [argument]
- Bad, because [argument]

## Links

- [Reference 1](url)
```

### Alternatives Considered
- **Plain Markdown without frontmatter**: Rejected - incompatible with SARA's structured parsing approach
- **JSON metadata**: Rejected - YAML is already established in the codebase and more readable
- **Separate metadata file**: Rejected - violates single-file-per-item pattern

---

## 2. ItemType Extension Pattern

### Decision
Add `ArchitectureDecisionRecord` as a new variant to the existing `ItemType` enum with prefix "ADR".

### Rationale
- Follows established pattern for all other item types (Solution, UseCase, SystemArchitecture, etc.)
- Enables reuse of existing infrastructure (parsing, graph building, CLI)
- Maintains type safety through Rust's enum system

### Implementation Approach

```rust
// In sara-core/src/model/item.rs
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
    ArchitectureDecisionRecord,  // NEW
}

impl ItemType {
    pub fn prefix(&self) -> &'static str {
        match self {
            // ... existing ...
            Self::ArchitectureDecisionRecord => "ADR",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            // ... existing ...
            Self::ArchitectureDecisionRecord => "Architecture Decision Record",
        }
    }

    // ADR does not have a required parent - it's a peer to design artifacts
    pub fn required_parent_type(&self) -> Option<ItemType> {
        match self {
            Self::ArchitectureDecisionRecord => None,
            // ... existing logic ...
        }
    }
}
```

### Alternatives Considered
- **Separate ADR struct not in ItemType**: Rejected - would require parallel infrastructure and break graph uniformity
- **ADR as metadata on existing items**: Rejected - ADRs are first-class entities with their own lifecycle

---

## 3. Relationship Type Design

### Decision
Add two new relationship type pairs:
1. `Justifies` / `IsJustifiedBy` - ADR ↔ Design Artifact
2. `Supersedes` / `IsSupersededBy` - ADR ↔ ADR

### Rationale
- Bidirectional relationships align with existing pattern (Refines/IsRefinedBy, etc.)
- Clear semantic meaning: ADRs *justify* design choices, ADRs *supersede* older decisions
- Enables graph traversal in both directions

### Implementation Approach

```rust
// In sara-core/src/model/relationship.rs
pub enum RelationshipType {
    Refines,
    IsRefinedBy,
    Derives,
    DerivesFrom,
    Satisfies,
    IsSatisfiedBy,
    DependsOn,
    IsRequiredBy,
    Justifies,       // NEW: ADR → Design Artifact
    IsJustifiedBy,   // NEW: Design Artifact → ADR
    Supersedes,      // NEW: ADR → ADR (newer supersedes older)
    IsSupersededBy,  // NEW: ADR → ADR (older is superseded by newer)
}

impl RelationshipType {
    pub fn inverse(&self) -> Self {
        match self {
            // ... existing ...
            Self::Justifies => Self::IsJustifiedBy,
            Self::IsJustifiedBy => Self::Justifies,
            Self::Supersedes => Self::IsSupersededBy,
            Self::IsSupersededBy => Self::Supersedes,
        }
    }
}
```

### Validation Rules

```rust
// In RelationshipRules
impl RelationshipRules {
    pub fn valid_justification_targets() -> Vec<ItemType> {
        vec![
            ItemType::SystemArchitecture,
            ItemType::SoftwareDetailedDesign,
            ItemType::HardwareDetailedDesign,
        ]
    }

    pub fn is_valid_justification(from: ItemType, to: ItemType) -> bool {
        from == ItemType::ArchitectureDecisionRecord
            && Self::valid_justification_targets().contains(&to)
    }

    pub fn is_valid_supersession(from: ItemType, to: ItemType) -> bool {
        from == ItemType::ArchitectureDecisionRecord
            && to == ItemType::ArchitectureDecisionRecord
    }
}
```

### Alternatives Considered
- **Generic "RelatesTo" relationship**: Rejected - loses semantic meaning needed for queries
- **Separate relationship storage**: Rejected - breaks existing graph model uniformity

---

## 4. Refactored Refs and Attributes (Enum-Based Design)

### Decision
Transform `UpstreamRefs`, `DownstreamRefs`, and `ItemAttributes` into enum-based structures per item type, plus add a new `PeerRefs` enum for peer relationships.

### Rationale
- Current flat structs with many `Option<T>` or empty `Vec<T>` fields pollute unrelated item types
- Enum-based design enforces type safety at compile time
- Clear separation: upstream (hierarchical up), downstream (hierarchical down), peer (same-level), attributes (non-relationship data)
- Extensible for future item types

### Implementation Approach

```rust
// In sara-core/src/model/item.rs

/// ADR lifecycle status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdrStatus {
    Proposed,
    Accepted,
    Deprecated,
    Superseded,
}

/// Upstream relationship references - enum per item type category
/// Points toward root/source (Solution direction, or justification source for ADR)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(tag = "upstream_type")]
pub enum UpstreamRefs {
    /// Solution has no upstream
    #[default]
    None,

    /// UseCase, Scenario refine their parent
    Refines {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        refines: Vec<ItemId>,
    },

    /// Requirements derive from parent (Scenario or SystemArchitecture)
    DerivesFrom {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        derives_from: Vec<ItemId>,
    },

    /// Design artifacts satisfy requirements AND may be justified by ADRs
    Design {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        satisfies: Vec<ItemId>,
        /// ADRs that justify this design artifact
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        justified_by: Vec<ItemId>,
    },

    /// ADR justifies design artifacts - enables impact tracing
    /// Trace path: ADR → (justifies) → Design → (satisfies) → Req → ... → Solution
    Adr {
        /// Design artifacts this ADR justifies (inverse of Design.justified_by)
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        justifies: Vec<ItemId>,
    },
}

/// Downstream relationship references - enum per item type category
/// Points toward leaves/dependents (DetailedDesign direction)
/// Note: These are typically computed as inverses of upstream refs
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(tag = "downstream_type")]
pub enum DownstreamRefs {
    /// Leaf items (HWDD, SWDD) and ADRs have no downstream
    #[default]
    None,

    /// Solution, UseCase are refined by children (inverse of Refines)
    IsRefinedBy {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        is_refined_by: Vec<ItemId>,
    },

    /// Scenario, SystemArchitecture have items derived from them (inverse of DerivesFrom)
    Derives {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        derives: Vec<ItemId>,
    },

    /// Requirements are satisfied by design artifacts (inverse of Satisfies)
    IsSatisfiedBy {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        is_satisfied_by: Vec<ItemId>,
    },

    /// ADRs have designs that reference them (inverse of Design.justified_by)
    IsJustificationFor {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        is_justification_for: Vec<ItemId>,
    },
}

/// Peer relationship references - for same-level dependencies
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(tag = "peer_type")]
pub enum PeerRefs {
    /// No peer relationships (Solution, UseCase, Scenario, Design artifacts)
    #[default]
    None,

    /// Requirements can depend on other requirements of same type
    DependsOn {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        depends_on: Vec<ItemId>,
    },

    /// ADRs can supersede other ADRs
    Supersedes {
        /// Older ADRs this one supersedes
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        supersedes: Vec<ItemId>,
        /// Newer ADR that supersedes this one (inverse, computed)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        superseded_by: Option<ItemId>,
    },
}

/// Type-specific attributes (non-relationship data)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(tag = "attr_type")]
pub enum TypeSpecificAttributes {
    /// No type-specific attributes (Solution, UseCase, Scenario)
    #[default]
    None,

    /// Requirements have a specification statement
    Requirement {
        specification: String,
    },

    /// SystemArchitecture has platform (HWDD/SWDD don't use this)
    Design {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        platform: Option<String>,
    },

    /// ADRs have status and deciders
    Adr {
        status: AdrStatus,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        deciders: Vec<String>,
    },
}
```

### Updated Item Structure

```rust
/// Represents a single document/node in the knowledge graph.
pub struct Item {
    pub id: ItemId,
    pub item_type: ItemType,
    pub name: String,
    pub description: Option<String>,
    pub source: SourceLocation,

    /// Upstream relationships (toward Solution / justification source)
    pub upstream: UpstreamRefs,

    /// Downstream relationships (toward DetailedDesign / justified items)
    pub downstream: DownstreamRefs,

    /// Peer relationships (same-level dependencies)
    pub peers: PeerRefs,

    /// Type-specific non-relationship attributes
    pub attributes: TypeSpecificAttributes,
}
```

### Relationship Model Summary

| Category | Enum | Item Types | Fields |
|----------|------|------------|--------|
| **Upstream** | `UpstreamRefs::Refines` | UseCase, Scenario | `refines` |
| | `UpstreamRefs::DerivesFrom` | SYSREQ, HWREQ, SWREQ | `derives_from` |
| | `UpstreamRefs::Design` | SYSARCH, HWDD, SWDD | `satisfies`, `justified_by` |
| | `UpstreamRefs::Adr` | ADR | `justifies` (enables tracing to Solution) |
| **Downstream** | `DownstreamRefs::IsRefinedBy` | Solution, UseCase | `is_refined_by` |
| | `DownstreamRefs::Derives` | Scenario, SYSARCH | `derives` |
| | `DownstreamRefs::IsSatisfiedBy` | SYSREQ, HWREQ, SWREQ | `is_satisfied_by` |
| | `DownstreamRefs::IsJustificationFor` | ADR | `is_justification_for` (inverse) |
| **Peer** | `PeerRefs::DependsOn` | SYSREQ, HWREQ, SWREQ | `depends_on` |
| | `PeerRefs::Supersedes` | ADR | `supersedes`, `superseded_by` |
| **Attributes** | `TypeSpecificAttributes::Requirement` | SYSREQ, HWREQ, SWREQ | `specification` |
| | `TypeSpecificAttributes::Design` | SYSARCH | `platform` |
| | `TypeSpecificAttributes::Adr` | ADR | `status`, `deciders` |

### ADR Tracing Example

From ADR-001, trace impact to Solution:
```
ADR-001.upstream.justifies = [SYSARCH-001]
  → SYSARCH-001.upstream.satisfies = [SYSREQ-001]
    → SYSREQ-001.upstream.derives_from = [SCEN-001]
      → SCEN-001.upstream.refines = [UC-001]
        → UC-001.upstream.refines = [SOL-001]
```

### Alternatives Considered
- **Keep flat structs with Option<T>**: Rejected - pollutes unrelated item types, no compile-time safety
- **Single mega-enum for everything**: Rejected - loses clear separation of concerns

---

## 5. Field Name Extensions

### Decision
Add new `FieldName` enum variants for ADR-specific YAML frontmatter fields.

### Rationale
- Follows existing pattern for field name management
- Enables consistent serialization/deserialization
- Supports validation and error messaging

### Implementation Approach

```rust
// In sara-core/src/model/field.rs
pub enum FieldName {
    // ... existing fields ...
    Status,        // ADR status
    Deciders,      // ADR deciders list
    Justifies,     // ADR → Design artifacts
    Supersedes,    // ADR → older ADRs
    SupersededBy,  // ADR ← newer ADR
}

impl FieldName {
    pub fn as_str(&self) -> &'static str {
        match self {
            // ... existing ...
            Self::Status => "status",
            Self::Deciders => "deciders",
            Self::Justifies => "justifies",
            Self::Supersedes => "supersedes",
            Self::SupersededBy => "superseded_by",
        }
    }
}
```

---

## 6. Graph Builder Extension

### Decision
Extend `GraphBuilder` to handle ADR relationships during two-pass construction.

### Rationale
- ADR relationships follow same bidirectional pattern as existing relationships
- No architectural changes needed - just additional match arms

### Implementation Approach

```rust
// In sara-core/src/graph/builder.rs
impl GraphBuilder {
    fn add_item_relationships(&mut self, item: &Item) -> Result<(), GraphError> {
        // Handle upstream relationships
        match &item.upstream {
            UpstreamRefs::Refines { refines } => {
                for target_id in refines {
                    self.add_relationship(item.id.clone(), target_id.clone(), RelationshipType::Refines)?;
                }
            }
            UpstreamRefs::DerivesFrom { derives_from } => {
                for target_id in derives_from {
                    self.add_relationship(item.id.clone(), target_id.clone(), RelationshipType::DerivesFrom)?;
                }
            }
            UpstreamRefs::Design { satisfies, justified_by } => {
                for target_id in satisfies {
                    self.add_relationship(item.id.clone(), target_id.clone(), RelationshipType::Satisfies)?;
                }
                for adr_id in justified_by {
                    self.add_relationship(item.id.clone(), adr_id.clone(), RelationshipType::IsJustifiedBy)?;
                }
            }
            UpstreamRefs::Adr { justifies } => {
                // ADR → Design: enables tracing from ADR to Solution
                for target_id in justifies {
                    self.add_relationship(item.id.clone(), target_id.clone(), RelationshipType::Justifies)?;
                }
            }
            UpstreamRefs::None => {}
        }

        // Downstream relationships are computed as inverses during graph building
        // (IsRefinedBy, Derives, IsSatisfiedBy, IsJustificationFor)

        // Handle peer relationships
        match &item.peers {
            PeerRefs::DependsOn { depends_on } => {
                for target_id in depends_on {
                    self.add_relationship(item.id.clone(), target_id.clone(), RelationshipType::DependsOn)?;
                }
            }
            PeerRefs::Supersedes { supersedes, .. } => {
                for target_id in supersedes {
                    self.add_relationship(item.id.clone(), target_id.clone(), RelationshipType::Supersedes)?;
                }
            }
            PeerRefs::None => {}
        }

        Ok(())
    }
}
```

---

## 7. Parser Extension

### Decision
Extend `RawFrontmatter` struct to include ADR-specific fields with serde deserialization.

### Rationale
- Serde handles optional fields gracefully
- Existing parser infrastructure handles the rest
- No changes to parsing logic - just struct extension

### Implementation Approach

```rust
// In sara-core/src/parser/markdown.rs
#[derive(Debug, Deserialize)]
pub struct RawFrontmatter {
    // ... existing fields ...

    // ADR-specific fields (all optional for non-ADR items)
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub deciders: Option<Vec<String>>,
    #[serde(default)]
    pub justifies: Option<Vec<String>>,
    #[serde(default)]
    pub supersedes: Option<Vec<String>>,
    #[serde(default)]
    pub superseded_by: Option<String>,
}
```

---

## 8. Best Practices Summary

| Area | Pattern | Source |
|------|---------|--------|
| ItemType extension | Add enum variant + implement all trait methods | Existing ItemType implementations |
| Relationship types | Bidirectional pairs with inverse() method | Existing RelationshipType pattern |
| Attributes | Enum-based TypeSpecificAttributes for type safety | Refactored ItemAttributes |
| Frontmatter parsing | Serde with #[serde(default)] for optional fields | Existing RawFrontmatter |
| Graph building | Two-pass: nodes first, then edges | Existing GraphBuilder |
| Templates | Tera with include for shared frontmatter | Existing templates/ |
| Testing | Unit tests for types, integration for graph | Existing test structure |

---

## Conclusion

All technical unknowns resolved. Implementation follows established patterns with minimal code changes:

1. **Model layer**: ~120 lines (ItemType, RelationshipType, TypeSpecificAttributes refactor, AdrStatus, FieldName)
2. **Parser layer**: ~30 lines (RawFrontmatter extension)
3. **Graph layer**: ~40 lines (GraphBuilder ADR handling)
4. **Template layer**: ~50 lines (adr.tera template)
5. **Tests**: ~200 lines (unit + integration)

Total estimated new code: ~420 lines of Rust

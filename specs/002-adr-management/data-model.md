# Data Model: ADR Management with Design Linking

**Feature**: 002-adr-management
**Date**: 2026-01-26

## Overview

This document defines the data model for Architecture Decision Records (ADRs) and their relationships to design artifacts within the SARA knowledge graph system.

---

## Entities

### 1. ArchitectureDecisionRecord (ADR)

**Description**: A document capturing an architectural decision, its context, alternatives considered, and the rationale for the chosen approach.

**Identity**: Sequential number with "ADR" prefix (e.g., ADR-001, ADR-002)

#### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | ItemId | Yes | Unique identifier (ADR-nnn format) |
| `type` | ItemType | Yes | Always `architecture_decision_record` |
| `name` | String | Yes | Decision title (human-readable) |
| `description` | Option<String> | No | Extended description (optional) |
| `status` | AdrStatus | Yes | Current lifecycle status |
| `deciders` | Vec<String> | Yes | List of people involved in the decision |
| `justifies` | Vec<ItemId> | Yes | Design artifacts this ADR justifies (can be empty) |
| `supersedes` | Vec<ItemId> | No | Older ADRs this decision supersedes |
| `superseded_by` | Option<ItemId> | No | Newer ADR that supersedes this one |
| `source` | SourceLocation | Yes | File path and optional Git ref |

> **Note**: Date and ticket information are not stored in frontmatter. Git history provides creation/modification dates, and ticket references can be included in the markdown body under the Links section.

#### Status Values (AdrStatus)

| Value | Description |
|-------|-------------|
| `proposed` | Decision is under consideration, not yet finalized |
| `accepted` | Decision has been approved and is in effect |
| `deprecated` | Decision is no longer recommended but not replaced |
| `superseded` | Decision has been replaced by a newer ADR |

#### Validation Rules

- `id` MUST match pattern `^ADR-\d{3,}$`
- `status` MUST be one of the defined AdrStatus values
- `deciders` MUST contain at least one entry
- `justifies` entries MUST reference valid SYSARCH, SWDD, or HWDD items
- `supersedes` entries MUST reference valid ADR items
- `superseded_by` MUST reference a valid ADR item if present
- When `status` is `superseded`, `superseded_by` SHOULD be populated

---

### 2. AdrStatus (Enum)

**Description**: Lifecycle status of an Architecture Decision Record.

```rust
pub enum AdrStatus {
    Proposed,    // Decision under consideration
    Accepted,    // Decision approved and in effect
    Deprecated,  // No longer recommended, not replaced
    Superseded,  // Replaced by newer ADR
}
```

#### State Transitions

No enforced transition rules - any status can change to any other status (flexibility for corrections).

```
┌──────────┐     ┌──────────┐     ┌────────────┐
│ Proposed │────▶│ Accepted │────▶│ Deprecated │
└──────────┘     └──────────┘     └────────────┘
      │                │                 │
      │                │                 │
      ▼                ▼                 ▼
┌────────────────────────────────────────────┐
│              Superseded                     │
└────────────────────────────────────────────┘
```

---

### 3. AdrAttributes (Struct)

**Description**: ADR-specific attributes stored within ItemAttributes.

```rust
pub struct AdrAttributes {
    pub status: AdrStatus,
    pub deciders: Vec<String>,
    pub justifies: Vec<ItemId>,
    pub supersedes: Vec<ItemId>,
    pub superseded_by: Option<ItemId>,
}
```

---

## Relationships

### 1. Justifies / IsJustifiedBy

**Description**: Links an ADR to the design artifacts it provides rationale for.

| Aspect | Value |
|--------|-------|
| **From** | ArchitectureDecisionRecord |
| **To** | SystemArchitecture, SoftwareDetailedDesign, HardwareDetailedDesign |
| **Cardinality** | Many-to-Many |
| **Direction** | ADR → Design Artifact (Justifies) |
| **Inverse** | Design Artifact → ADR (IsJustifiedBy) |

**Graph Representation**:
```
ADR-001 ──Justifies──▶ SYSARCH-001
ADR-001 ──Justifies──▶ SWDD-001
SYSARCH-001 ◀──IsJustifiedBy── ADR-001
```

**Storage**:
- ADR stores `justifies: [SYSARCH-001, SWDD-001]` in frontmatter
- Design artifacts store `justified_by: [ADR-001]` (uses reserved field)

---

### 2. Supersedes / IsSupersededBy

**Description**: Links a newer ADR to older ADRs it replaces.

| Aspect | Value |
|--------|-------|
| **From** | ArchitectureDecisionRecord |
| **To** | ArchitectureDecisionRecord |
| **Cardinality** | Many-to-Many |
| **Direction** | Newer ADR → Older ADR (Supersedes) |
| **Inverse** | Older ADR → Newer ADR (IsSupersededBy) |

**Graph Representation**:
```
ADR-005 ──Supersedes──▶ ADR-001
ADR-005 ──Supersedes──▶ ADR-002
ADR-001 ◀──IsSupersededBy── ADR-005
```

**Storage**:
- Newer ADR stores `supersedes: [ADR-001, ADR-002]` in frontmatter
- Older ADR stores `superseded_by: ADR-005` in frontmatter

---

## Entity Relationship Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                      SARA Knowledge Graph                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────┐                                                     │
│  │   SOL   │                                                     │
│  └────┬────┘                                                     │
│       │ Refines                                                  │
│       ▼                                                          │
│  ┌─────────┐                                                     │
│  │   UC    │                                                     │
│  └────┬────┘                                                     │
│       │ Refines                                                  │
│       ▼                                                          │
│  ┌─────────┐                                                     │
│  │  SCEN   │                                                     │
│  └────┬────┘                                                     │
│       │ DerivesFrom                                              │
│       ▼                                                          │
│  ┌─────────┐                                                     │
│  │ SYSREQ  │                                                     │
│  └────┬────┘                                                     │
│       │ Satisfies                                                │
│       ▼                                                          │
│  ┌─────────────┐◀───────Justifies───────┐                       │
│  │  SYSARCH    │                         │                       │
│  └──────┬──────┘                         │                       │
│    ┌────┴────┐                      ┌────┴────┐                  │
│    │         │                      │   ADR   │◀──Supersedes───┐ │
│    ▼         ▼                      └─────────┘                │ │
│ ┌──────┐ ┌──────┐                                              │ │
│ │HWREQ │ │SWREQ │                                              │ │
│ └──┬───┘ └──┬───┘                                              │ │
│    │        │                       ┌─────────┐                │ │
│    ▼        ▼                       │   ADR   │────────────────┘ │
│ ┌──────┐ ┌──────┐◀────Justifies─────┤ (older) │                  │
│ │ HWDD │ │ SWDD │                   └─────────┘                  │
│ └──────┘ └──────┘                                                │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## YAML Frontmatter Schema

### ADR Document

```yaml
---
id: "ADR-001"
type: architecture_decision_record
name: "Use Microservices Architecture"
description: "Optional extended description"
status: accepted
deciders:
  - "Alice Smith"
  - "Bob Jones"
justifies:
  - "SYSARCH-001"
  - "SWDD-001"
supersedes: []
superseded_by: null
---
```

### Design Artifact with ADR Reference

```yaml
---
id: "SYSARCH-001"
type: system_architecture
name: "Microservices Platform"
# ... existing fields ...
justified_by:
  - "ADR-001"
  - "ADR-003"
---
```

---

## Indexes and Queries

### Required Queries

| Query | Description | Implementation |
|-------|-------------|----------------|
| `adrs_by_status(status)` | All ADRs with given status | Filter `items_by_type(ADR)` by status |
| `adrs_for_artifact(id)` | All ADRs justifying an artifact | Graph traversal: `parents(id, IsJustifiedBy)` |
| `artifacts_for_adr(id)` | All artifacts justified by ADR | Graph traversal: `children(id, Justifies)` |
| `superseding_chain(id)` | Full chain of superseding ADRs | Recursive graph traversal |
| `active_adrs()` | All ADRs with status Accepted | Filter by status |

### Graph Traversal Examples

```rust
// Get all ADRs that justify SYSARCH-001
let adrs = graph.parents(&ItemId::new("SYSARCH-001")?, RelationshipType::IsJustifiedBy);

// Get all artifacts justified by ADR-001
let artifacts = graph.children(&ItemId::new("ADR-001")?, RelationshipType::Justifies);

// Get supersession chain for ADR-001
let chain = graph.traverse_chain(&ItemId::new("ADR-001")?, RelationshipType::IsSupersededBy);
```

---

## Migration Notes

### Existing Items

No migration required for existing items. The `justified_by` field in `ItemAttributes` is already reserved and optional.

### New ADR Items

1. Create markdown file with YAML frontmatter in appropriate directory
2. Run `sara validate` to verify frontmatter and relationships
3. Run `sara build` to add to knowledge graph

### Backward Compatibility

- Existing items without `justified_by` field: Treated as empty list
- Existing parsers: ADR fields are optional, won't break non-ADR items
- Existing graph queries: New relationship types don't affect existing traversals

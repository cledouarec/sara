# CLI Interface Contracts: ADR Management

**Feature**: 002-adr-management
**Date**: 2026-01-26

## Overview

This document defines the command-line interface contracts for ADR management operations in the SARA CLI tool.

---

## CLI Refactoring: Init Subcommands

This feature introduces a refactored `sara init` command structure using type-specific subcommands to reduce complexity and improve usability.

### Command Structure

```
sara init              # Interactive mode (prompts for everything)
sara init <item_type>  # Type-specific subcommand with relevant options only
```

### Available Subcommands

| Subcommand | Alias | Description |
|------------|-------|-------------|
| `sara init adr` | - | Create Architecture Decision Record |
| `sara init system-requirement` | `sysreq` | Create System Requirement |
| `sara init system-architecture` | `sysarch` | Create System Architecture |
| `sara init software-requirement` | `swreq` | Create Software Requirement |
| `sara init hardware-requirement` | `hwreq` | Create Hardware Requirement |
| `sara init software-detailed-design` | `swdd` | Create Software Detailed Design |
| `sara init hardware-detailed-design` | `hwdd` | Create Hardware Detailed Design |
| `sara init solution` | `sol` | Create Solution |
| `sara init use-case` | `uc` | Create Use Case |
| `sara init scenario` | `scen` | Create Scenario |

---

## Commands

### 1. Create ADR

**Command**: `sara init adr`

**Description**: Create a new Architecture Decision Record.

**Usage**:
```bash
sara init adr <FILE> [OPTIONS]
sara init adr docs/adr/use-microservices.md --status proposed --deciders "Alice, Bob"
```

**Arguments**:
| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `FILE` | Path | Yes | Target markdown file |

**Options**:
| Option | Short | Type | Required | Default | Description |
|--------|-------|------|----------|---------|-------------|
| `--id` | | String | No | Auto | Item identifier |
| `--name` | `-n` | String | No | From title | Decision title |
| `--description` | `-d` | String | No | - | Item description |
| `--status` | `-s` | Enum | No | proposed | ADR status (proposed, accepted, deprecated, superseded) |
| `--deciders` | | String[] | No | - | List of deciders |
| `--justifies` | `-j` | ItemId[] | No | [] | Design artifacts this ADR justifies |
| `--supersedes` | | ItemId[] | No | [] | Older ADRs this decision supersedes |
| `--force` | | Flag | No | false | Overwrite existing frontmatter |

**Output**:
```
Created docs/adr/use-microservices.md with Architecture Decision Record template

  ID:   ADR-003
  Name: Use Microservices
  Type: Architecture Decision Record
```

**Exit Codes**:
| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Validation error (invalid input) |
| 2 | File already has frontmatter (use --force) |
| 3 | Invalid option for item type |

---

### 2. List ADRs

**Command**: `sara list adr`

**Description**: List all Architecture Decision Records with optional filtering.

**Usage**:
```bash
sara list adr
sara list adr --status accepted
sara list adr --justifies SYSARCH-001
sara list adr --format json
```

**Options**:
| Option | Short | Type | Required | Default | Description |
|--------|-------|------|----------|---------|-------------|
| `--status` | `-s` | Enum | No | - | Filter by status |
| `--justifies` | `-j` | ItemId | No | - | Filter by justified artifact |
| `--format` | `-f` | Enum | No | table | Output format (table, json, csv) |
| `--sort` | | Enum | No | id | Sort by (id, status, name) |

**Output (table)**:
```
ID        Status     Name
────────  ─────────  ─────────────────────────
ADR-001   accepted   Use Microservices Architecture
ADR-002   accepted   PostgreSQL for Persistence
ADR-003   proposed   Event-Driven Communication

3 ADRs found
```

**Output (json)**:
```json
{
  "adrs": [
    {
      "id": "ADR-001",
      "name": "Use Microservices Architecture",
      "status": "accepted",
      "justifies": ["SYSARCH-001", "SWDD-001"]
    }
  ],
  "count": 1
}
```

---

### 3. Show ADR

**Command**: `sara show <ADR-ID>`

**Description**: Display detailed information about a specific ADR.

**Usage**:
```bash
sara show ADR-001
sara show ADR-001 --format json
sara show ADR-001 --with-content
```

**Options**:
| Option | Short | Type | Required | Default | Description |
|--------|-------|------|----------|---------|-------------|
| `--format` | `-f` | Enum | No | detail | Output format (detail, json) |
| `--with-content` | `-c` | Flag | No | false | Include markdown content |

**Output (detail)**:
```
ADR-001: Use Microservices Architecture
══════════════════════════════════════

Status:     accepted
Deciders:   Alice Smith, Bob Jones
File:       docs/adr/ADR-001-use-microservices.md

Justifies:
  • SYSARCH-001 (Microservices Platform)
  • SWDD-001 (API Gateway Service)

Supersedes:
  (none)

Superseded By:
  (none)
```

---

### 4. Update ADR Status

**Command**: `sara update adr <ADR-ID>`

**Description**: Update the status or metadata of an existing ADR.

**Usage**:
```bash
sara update adr ADR-001 --status accepted
sara update adr ADR-001 --superseded-by ADR-005
sara update adr ADR-001 --add-justifies SYSARCH-002
```

**Options**:
| Option | Short | Type | Required | Default | Description |
|--------|-------|------|----------|---------|-------------|
| `--status` | `-s` | Enum | No | - | New status |
| `--superseded-by` | | ItemId | No | - | ADR that supersedes this one |
| `--add-justifies` | | ItemId | No | - | Add justified artifact |
| `--remove-justifies` | | ItemId | No | - | Remove justified artifact |

**Output**:
```
Updated ADR-001
  Status: proposed → accepted
```

---

### 5. Link ADR to Artifact

**Command**: `sara link adr <ADR-ID> <ARTIFACT-ID>`

**Description**: Create a justification link between an ADR and a design artifact.

**Usage**:
```bash
sara link adr ADR-001 SYSARCH-001
sara link adr ADR-001 SWDD-001 HWDD-001
```

**Arguments**:
| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `ADR-ID` | ItemId | Yes | Source ADR |
| `ARTIFACT-ID` | ItemId[] | Yes | Target design artifacts (1+) |

**Output**:
```
Linked ADR-001 to:
  • SYSARCH-001 (Microservices Platform)
  • SWDD-001 (API Gateway Service)
```

**Validation**:
- ADR-ID must exist and be of type ADR
- ARTIFACT-IDs must exist and be SYSARCH, SWDD, or HWDD
- Duplicate links are idempotent (no error)

---

### 6. Query ADR Relationships

**Command**: `sara query adr`

**Description**: Query ADR relationships in the knowledge graph.

**Usage**:
```bash
# Find all ADRs justifying an artifact
sara query adr --justifying SYSARCH-001

# Find supersession chain
sara query adr --supersession-chain ADR-001

# Find all active ADRs for an artifact
sara query adr --active-for SWDD-001
```

**Options**:
| Option | Type | Description |
|--------|------|-------------|
| `--justifying` | ItemId | ADRs that justify given artifact |
| `--justified-by` | ItemId | Artifacts justified by given ADR |
| `--supersession-chain` | ItemId | Full supersession history |
| `--active-for` | ItemId | Active (accepted) ADRs for artifact |

**Output**:
```
ADRs justifying SYSARCH-001:
  • ADR-001 (accepted) - Use Microservices Architecture
  • ADR-003 (accepted) - Kubernetes Orchestration
```

---

### 7. Validate ADR

**Command**: `sara validate` (extended)

**Description**: Validate ADR documents and relationships (extends existing validate command).

**Validation Rules**:
| Rule | Severity | Message |
|------|----------|---------|
| Missing required fields | Error | `ADR-001: Missing required field 'deciders'` |
| Invalid status value | Error | `ADR-001: Invalid status 'draft', expected one of: proposed, accepted, deprecated, superseded` |
| Invalid justifies target | Error | `ADR-001: Cannot justify SYSREQ-001 (must be SYSARCH, SWDD, or HWDD)` |
| Superseded without superseded_by | Warning | `ADR-001: Status is 'superseded' but 'superseded_by' not set` |
| Circular supersession | Error | `ADR-001: Circular supersession detected: ADR-001 → ADR-002 → ADR-001` |
| Orphan ADR | Warning | `ADR-001: ADR does not justify any design artifacts` |

**Output**:
```
Validating ADRs...
  ✓ ADR-001: Valid
  ✓ ADR-002: Valid
  ⚠ ADR-003: Warning - ADR does not justify any design artifacts
  ✗ ADR-004: Error - Missing required field 'deciders'

3 valid, 1 warning, 1 error
```

---

## Error Messages

### Standard Error Format

```
Error: <short description>

  <detailed message>

  Hint: <actionable suggestion>
```

### Examples

```
Error: Invalid ADR ID format

  'ADR-1' does not match required format 'ADR-nnn'

  Hint: Use three or more digits, e.g., 'ADR-001'
```

```
Error: Cannot justify item

  ADR-001 cannot justify SYSREQ-001
  ADRs can only justify: SYSARCH, SWDD, HWDD

  Hint: Check if you meant to link to a different artifact type
```

---

## Integration Points

### With Existing Commands

| Command | Integration |
|---------|-------------|
| `sara build` | Include ADRs in graph construction |
| `sara validate` | Extended with ADR validation rules |
| `sara list` | Extended with `adr` subcommand |
| `sara show` | Works with ADR-nnn IDs |
| `sara graph` | Include ADR nodes and justification edges |

### Template Generation

The `sara new adr` command generates a file using the ADR Tera template:

```
docs/adr/ADR-{number}-{slug}.md
```

Where:
- `{number}` = Next available ADR number (zero-padded to 3 digits)
- `{slug}` = Kebab-case of name (e.g., "Use Microservices" → "use-microservices")

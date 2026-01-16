# CLI Contract: Requirements Knowledge Graph

**Phase**: 1 - Design
**Date**: 2026-01-11
**Branch**: `001-requirements-knowledge-graph`

## Command Overview

```
sara <COMMAND> [OPTIONS]

Commands:
  parse      Parse documents and build the knowledge graph
  validate   Validate graph integrity
  query      Query items and traceability chains
  report     Generate coverage and traceability reports
  init       Initialize metadata in a Markdown file
  diff       Compare graphs between Git references

Global Options:
  -c, --config <FILE>    Path to configuration file [default: sara.toml]
  -v, --verbose...       Increase verbosity (-v, -vv, -vvv)
  -q, --quiet            Suppress all output except errors
      --no-color         Disable colored output
      --no-emoji         Disable emoji output
  -h, --help             Print help
  -V, --version          Print version
```

---

## Commands

### parse

Parse Markdown documents from configured repositories and build the knowledge graph.

```
sara parse [OPTIONS]

Options:
  -r, --repository <PATH>...    Additional repository paths (adds to config)
      --at <GIT_REF>            Read from specific Git commit/branch
  -o, --output <FILE>           Output parsed graph to file (JSON format)
```

**Exit Codes**:
| Code | Meaning |
|------|---------|
| 0 | Success - all documents parsed |
| 1 | Parse errors - some documents failed to parse |
| 2 | Configuration error - invalid config or missing repos |

**Output Examples**:

```
$ sara parse
📂 Scanning 3 repositories...
  ✅ ./docs (42 documents)
  ✅ ../hw-repo/specs (18 documents)
  ✅ ../sw-repo/requirements (35 documents)

📊 Parse Summary:
  Documents: 95
  Items: 95
  Relationships: 142
  Duration: 0.23s
```

```
$ sara parse -r ./invalid_docs
📊 Parse Summary:
  Documents: 50
  Items: 50
  Relationships: 75
  Duration: 0.12s

✅ Parse completed successfully

⚠️  Found 5 validation error(s). Run 'sara validate' for details.
```

---

### validate

Validate the knowledge graph for integrity issues.

```
sara validate [OPTIONS]

Options:
      --strict            Treat orphan items as errors (default: warnings)
      --at <GIT_REF>      Validate at specific Git commit/branch
      --format <FORMAT>   Output format [default: text] [possible: text, json]
  -o, --output <FILE>     Write validation report to file
```

**Exit Codes**:
| Code | Meaning |
|------|---------|
| 0 | Valid - no errors found |
| 1 | Invalid - validation errors found |
| 2 | Configuration error |

**Output Examples**:

```
$ sara validate
🔍 Validating knowledge graph...

❌ Validation Failed (3 errors, 2 warnings)

Errors:
  ❌ Broken reference: REQ-005 references non-existent item SCEN-999
     └─ docs/requirements/REQ-005.md:12

  ❌ Duplicate identifier: UC-003 defined in multiple files
     ├─ ./docs/use-cases/UC-003.md:1
     └─ ../other-repo/UC-003.md:1

  ❌ Circular reference detected: SCEN-001 → SCEN-002 → SCEN-001
     └─ docs/scenarios/SCEN-001.md:8

Warnings:
  ⚠️  Orphan item: SWREQ-042 has no upstream parent
     └─ sw-repo/requirements/SWREQ-042.md:1
```

```
$ sara validate --format json
{
  "valid": false,
  "errors": [...],
  "warnings": [...],
  "summary": {
    "items_checked": 95,
    "relationships_checked": 142,
    "duration_ms": 45
  }
}
```

---

### query

Query items and traceability chains.

```
sara query <ITEM_ID> [OPTIONS]

Arguments:
  <ITEM_ID>    The item identifier to query

Options:
  -u, --upstream         Show upstream chain (toward Solution)
  -d, --downstream       Show downstream chain (toward Detailed Designs)
  -t, --type <TYPE>...   Filter by item type(s)
      --depth <N>        Limit traversal depth [default: unlimited]
      --format <FORMAT>  Output format [default: tree] [possible: tree, json]
```

**Exit Codes**:
| Code | Meaning |
|------|---------|
| 0 | Success - item found |
| 1 | Not found - item does not exist |
| 2 | Configuration error |

**Output Examples**:

```
$ sara query REQ-001
📋 REQ-001: Performance Requirement
   Type: System Requirement
   File: docs/requirements/REQ-001.md

   Requires:
     └─ SCEN-001: User Login Scenario

   Realized by:
     ├─ SARCH-001: Web Platform Architecture
     └─ SARCH-002: Mobile Platform Architecture
```

```
$ sara query REQ-001 --upstream
📋 Upstream Traceability for REQ-001

SOL-001: Customer Portal Solution
└── UC-001: User Authentication
    └── SCEN-001: User Login Scenario
        └── REQ-001: Performance Requirement ← (you are here)
```

```
$ sara query SOL-001 --downstream --depth 2
📋 Downstream from SOL-001 (depth: 2)

SOL-001: Customer Portal Solution
├── UC-001: User Authentication
│   ├── SCEN-001: User Login Scenario
│   └── SCEN-002: Password Reset Scenario
└── UC-002: Product Browsing
    └── SCEN-003: Search Products Scenario
```

```
$ sara query NONEXISTENT-001
❌ Item not found: NONEXISTENT-001

Did you mean?
  • REQ-001
  • SCEN-001
```

---

### report

Generate coverage and traceability reports.

```
sara report <TYPE> [OPTIONS]

Arguments:
  <TYPE>    Report type [possible: coverage, matrix]

Options:
      --format <FORMAT>   Output format [default: text] [possible: text, json, csv, html]
  -o, --output <FILE>     Write report to file
      --by-type           Group results by item type
      --include-warnings  Include warnings in coverage calculation
```

**Exit Codes**:
| Code | Meaning |
|------|---------|
| 0 | Success |
| 2 | Configuration error |

**Output Examples**:

```
$ sara report coverage
📊 Traceability Coverage Report

Overall Coverage: 87.4%

By Item Type:
  Type                      Items   Complete   Coverage
  ─────────────────────────────────────────────────────
  Solutions                     2          2     100.0%
  Use Cases                     8          8     100.0%
  Scenarios                    15         15     100.0%
  System Requirements          25         22      88.0%
  System Architectures          5          5     100.0%
  Hardware Requirements        12         10      83.3%
  Software Requirements        18         15      83.3%
  HW Detailed Designs           5          5     100.0%
  SW Detailed Designs           5          4      80.0%

Incomplete Items:
  • REQ-023: Missing parent Scenario
  • REQ-024: Missing parent Scenario
  • SWDD-005: Missing parent Software Requirement
```

```
$ sara report matrix --format csv -o traceability.csv
✅ Traceability matrix exported to traceability.csv
```

---

### init

Initialize or update metadata in a Markdown file.

```
sara init <FILE> [OPTIONS]

Arguments:
  <FILE>    Markdown file to initialize

Options:
  -t, --type <TYPE>              Item type [required]
      --id <ID>                  Item identifier [auto-generated if not provided]
      --name <NAME>              Item name [extracted from title if not provided]
  -d, --description <DESC>       Item description
      --refines <ID>...          Upstream references (for use_case, scenario)
      --derives-from <ID>...     Upstream references (for requirements)
      --satisfies <ID>...        Upstream references (for architectures, designs)
      --specification <TEXT>     Specification statement (for requirements)
      --platform <PLATFORM>      Target platform (for system_architecture)
      --force                    Overwrite existing frontmatter
```

**Type-specific options**:

| Option | Valid for types |
|--------|-----------------|
| `--refines` | use_case, scenario |
| `--derives-from` | system_requirement, hardware_requirement, software_requirement |
| `--satisfies` | system_architecture, hardware_detailed_design, software_detailed_design |
| `--specification` | system_requirement, hardware_requirement, software_requirement |
| `--platform` | system_architecture |

**Exit Codes**:
| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | File error - cannot read/write file |
| 2 | Already has frontmatter (use --force to overwrite) |
| 3 | Invalid option for item type |

**Output Examples**:

```
$ sara init docs/REQ-050.md --type system_requirement \
    --name "Performance Requirement" \
    --description "System response time constraint" \
    --derives-from SCEN-001 SCEN-002 \
    --specification "The system SHALL respond within 100ms"
✅ Initialized docs/REQ-050.md as System Requirement

Added frontmatter:
---
id: "SYSREQ-050"
type: system_requirement
name: "Performance Requirement"
description: "System response time constraint"
specification: "The system SHALL respond within 100ms"
derives_from:
  - "SCEN-001"
  - "SCEN-002"
---
```

```
$ sara init docs/UC-010.md --type use_case \
    --name "User Login" \
    --refines SOL-001
✅ Initialized docs/UC-010.md as Use Case

Added frontmatter:
---
id: "UC-010"
type: use_case
name: "User Login"
refines:
  - "SOL-001"
---
```

---

### diff

Compare knowledge graphs between Git references.

```
sara diff <REF1> <REF2> [OPTIONS]

Arguments:
  <REF1>    First Git reference (commit, branch, tag)
  <REF2>    Second Git reference (commit, branch, tag)

Options:
      --format <FORMAT>   Output format [default: text] [possible: text, json]
      --stat              Show summary statistics only
```

**Exit Codes**:
| Code | Meaning |
|------|---------|
| 0 | No differences or success |
| 1 | Differences found |
| 2 | Git error or invalid reference |

**Output Examples**:

```
$ sara diff main HEAD
📊 Graph Comparison: main → HEAD

Added Items (3):
  + REQ-051: New Performance Requirement
  + REQ-052: New Security Requirement
  + SWDD-010: Authentication Implementation

Removed Items (1):
  - REQ-023: Deprecated Requirement

Modified Relationships (2):
  ~ SCEN-005: Added requires → REQ-051
  ~ SARCH-002: Removed realizes → SWREQ-010

Broken Links Introduced:
  ❌ REQ-024 now references non-existent SCEN-099
```

```
$ sara diff v1.0.0 v2.0.0 --stat
📊 Summary: v1.0.0 → v2.0.0

  Added:      15 items, 23 relationships
  Removed:     3 items,  5 relationships
  Modified:    8 relationships
  New Errors:  1 broken reference
```

---

## Configuration File Format

**Location**: `sara.toml` (current directory or specified via `--config`)

```toml
# Repository configuration
[repositories]
paths = [
    "./docs",
    "../hardware-repo/specs",
    "../software-repo/requirements"
]

# Validation settings
[validation]
# Treat orphan items as errors (true) or warnings (false)
strict_orphans = false

# Output settings
[output]
# Enable colored output
colors = true
# Enable emoji output
emojis = true

# Custom templates (optional)
# Override built-in Tera templates with custom template files
# Built-in templates are embedded in the binary and use the Tera engine
# Supports glob patterns (e.g., "*.tera", "*.md", "./templates/**/*.tera")
# Each template must contain exactly one 'type' field in its YAML frontmatter
[templates]
paths = [
    "./templates/*.tera"
]
```

---

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `SARA_CONFIG` | Path to configuration file | `sara.toml` |
| `RUST_LOG` | Logging level (error, warn, info, debug, trace) | `warn` |
| `NO_COLOR` | Disable colored output when set | unset |

---

## Error Message Format

All error messages follow a consistent format for parseability:

```
{emoji} {severity}: {message}
   └─ {file}:{line}
```

Example:
```
❌ Error: Broken reference: REQ-005 references non-existent item SCEN-999
   └─ docs/requirements/REQ-005.md:12
```

For JSON output, errors are structured:
```json
{
  "severity": "error",
  "code": "broken_reference",
  "message": "REQ-005 references non-existent item SCEN-999",
  "location": {
    "file": "docs/requirements/REQ-005.md",
    "line": 12
  },
  "related_items": ["REQ-005", "SCEN-999"]
}
```

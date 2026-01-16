# Quickstart: Sara - Requirements Knowledge Graph CLI

**Phase**: 1 - Design
**Date**: 2026-01-11
**Branch**: `001-requirements-knowledge-graph`

## Overview

Sara is a command-line tool that manages Architecture documents and Requirements as an interconnected knowledge graph. It parses Markdown files with YAML frontmatter from multiple Git repositories and provides validation, querying, and reporting capabilities.

## Installation

### From Source (Rust)

```bash
# Clone the repository
git clone https://github.com/your-org/sara.git
cd sara

# Build and install
cargo install --path sara-cli

# Verify installation
sara --version
```

### Pre-built Binaries

Download from the releases page for your platform:
- `sara-linux-x86_64` - Linux (64-bit)
- `sara-darwin-x86_64` - macOS (Intel)
- `sara-darwin-arm64` - macOS (Apple Silicon)
- `sara-windows-x86_64.exe` - Windows (64-bit)

## Getting Started

### 1. Create Your First Document

Create a Markdown file with YAML frontmatter:

```markdown
<!-- docs/solutions/SOL-001.md -->
---
id: "SOL-001"
type: solution
name: "Customer Portal"
description: "Web-based customer self-service portal"
requires:
  - "UC-001"
  - "UC-002"
---

# Customer Portal Solution

This solution provides a self-service portal for customers to manage their accounts,
view orders, and submit support requests.

## Goals

- Reduce support call volume by 30%
- Improve customer satisfaction scores
- Enable 24/7 self-service capabilities
```

### 2. Create Configuration

Create a `sara.toml` file in your project root:

```toml
[repositories]
paths = [
    "./docs"
]

[validation]
strict_orphans = false

[output]
colors = true
emojis = true
```

### 3. Parse and Validate

```bash
# Parse all documents
sara parse

# Validate the knowledge graph
sara validate

# Query a specific item
sara query SOL-001

# Generate a coverage report
sara report coverage
```

## Document Types

Sara recognizes 9 document types that form the requirements hierarchy:

| Type | YAML Value | Description |
|------|------------|-------------|
| Solution | `solution` | Customer-facing solution |
| Use Case | `use_case` | Customer/market need |
| Scenario | `scenario` | Abstract system behavior |
| System Requirement | `system_requirement` | Quantifiable system-level need |
| System Architecture | `system_architecture` | Platform implementation |
| Hardware Requirement | `hardware_requirement` | Hardware-specific need |
| Software Requirement | `software_requirement` | Software-specific need |
| HW Detailed Design | `hardware_detailed_design` | Hardware implementation |
| SW Detailed Design | `software_detailed_design` | Software implementation |

## YAML Frontmatter Schema

### Required Fields

```yaml
---
id: "UNIQUE-ID"          # Unique identifier (required)
type: solution           # Document type (required)
name: "Human Name"       # Display name (required)
---
```

### Optional Fields

```yaml
---
id: "REQ-001"
type: system_requirement
name: "Performance Requirement"
description: "Extended description of this item"
derives_from:                     # IDs of parent items (upstream)
  - "SCEN-001"
is_satisfied_by: []               # IDs of child items (downstream)
verification_method: test         # For requirements: test, analysis, inspection, demonstration
requirement_text: "The system SHALL respond within 100ms"
platform: "AWS Cloud"             # For architectures
---
```

### Relationship Fields by Item Type

| Item Type | Upstream Field | Downstream Field |
|-----------|----------------|------------------|
| Use Case | `refines` (Solution) | `is_refined_by` (Scenarios) |
| Scenario | `refines` (Use Case) | `derives` (System Requirements) |
| System Requirement | `derives_from` (Scenario) | `is_satisfied_by` (System Architectures) |
| System Architecture | `satisfies` (System Requirements) | `derives` (HW/SW Requirements) |
| HW/SW Requirement | `derives_from` (System Architecture) | `is_satisfied_by` (Detailed Designs) |
| HW/SW Detailed Design | `satisfies` (HW/SW Requirement) | - |

## Relationship Rules

Items follow a bidirectional hierarchy with semantic relationship names:

**Downstream (from Solution toward Detailed Designs):**
```
Solution
  └─[is_refined_by]─► Use Case
                        └─[is_refined_by]─► Scenario
                                              └─[derives]─► System Requirement
                                                              └─[is_satisfied_by]─► System Architecture
                                                                                      └─[derives]─► HW/SW Requirement
                                                                                                      └─[is_satisfied_by]─► HW/SW Detailed Design
```

**Upstream (from Detailed Designs toward Solution):**
```
Solution
  └─[refines]─◄ Use Case
                  └─[refines]─◄ Scenario
                                  └─[derives_from]─◄ System Requirement
                                                       └─[satisfies]─◄ System Architecture
                                                                         └─[derives_from]─◄ HW/SW Requirement
                                                                                              └─[satisfies]─◄ HW/SW Detailed Design
```

**Relationship Types:**
| Downstream | Upstream | Usage |
|------------|----------|-------|
| `is_refined_by` | `refines` | Solution ↔ Use Case ↔ Scenario |
| `derives` | `derives_from` | Scenario ↔ Sys Req, Sys Arch ↔ HW/SW Req |
| `is_satisfied_by` | `satisfies` | Sys Req ↔ Sys Arch, HW/SW Req ↔ Detailed Design |

## Common Workflows

### Validate Before Commit

Add to your git pre-commit hook:

```bash
#!/bin/bash
sara validate --strict
if [ $? -ne 0 ]; then
    echo "❌ Validation failed. Please fix errors before committing."
    exit 1
fi
```

### Multi-Repository Setup

```toml
# sara.toml
[repositories]
paths = [
    "./system-docs",
    "../hardware-repo/specs",
    "../software-repo/requirements"
]
```

### Check Traceability

```bash
# Find all items without complete traceability
sara validate --strict

# View upstream chain for a requirement
sara query SWREQ-042 --upstream

# View what implements a scenario
sara query SCEN-001 --downstream
```

### Compare Branches

```bash
# See what changed in your feature branch
sara diff main HEAD

# Compare releases
sara diff v1.0.0 v2.0.0
```

### Generate Reports

```bash
# Coverage report to stdout
sara report coverage

# Export traceability matrix to CSV
sara report matrix --format csv -o traceability.csv

# HTML report for stakeholders
sara report coverage --format html -o report.html
```

## Troubleshooting

### "Broken reference" Error

```
❌ Broken reference: REQ-005 references non-existent item SCEN-999
```

**Cause**: The `requires` or `realizes` field references an ID that doesn't exist.

**Fix**: Check the ID spelling or create the missing document.

### "Duplicate identifier" Error

```
❌ Duplicate identifier: UC-003 defined in multiple files
```

**Cause**: Two documents have the same `id` field.

**Fix**: Each ID must be unique across all repositories. Rename one of them.

### "Circular reference" Error

```
❌ Circular reference detected: SCEN-001 → SCEN-002 → SCEN-001
```

**Cause**: Documents form a cycle in their `requires`/`realizes` relationships.

**Fix**: Review the relationships and break the cycle. Requirements should flow one direction (Solution → Detailed Design).

### "Orphan item" Warning

```
⚠️ Orphan item: SWREQ-042 has no upstream parent
```

**Cause**: This item doesn't have any parent item requiring it.

**Fix**: Either add this item to a parent's `requires`/`realizes` list, or this might be intentional for top-level Solutions.

### "Invalid metadata" Error

```
❌ Invalid metadata in docs/REQ-001.md: missing required field 'type'
```

**Cause**: The YAML frontmatter is missing a required field.

**Fix**: Ensure the frontmatter has `id`, `type`, and `name` fields.

## Interactive Mode (Added 2026-01-14)

When you run `sara init` without the `--type` argument, Sara enters interactive mode to guide you through creating a new document.

### Starting Interactive Mode

```bash
# Interactive mode - prompts for all fields
sara init docs/new-requirement.md

# Interactive mode with some fields pre-filled
sara init docs/feature.md --name "Authentication Feature"

# Non-interactive mode (traditional)
sara init docs/SOL-001.md --type solution --name "My Solution"
```

### Interactive Prompt Flow

When running interactively, Sara will prompt you for:

1. **Item Type** - Select from available types using arrow keys:
   ```
   ? Select item type:
   > Solution
     Use Case
     Scenario
     System Requirement
     System Architecture
     Hardware Requirement
     Software Requirement
     Hardware Detailed Design
     Software Detailed Design
   ```

2. **Item Name** - Enter the human-readable name:
   ```
   ? Item name: Customer Authentication Flow
   ```

3. **Identifier** - Auto-suggested based on type and existing items:
   ```
   ? Identifier [SCEN-003]: SCEN-003
   ```

4. **Description** (optional) - Brief summary:
   ```
   ? Description (optional): Describes the user login process
   ```

5. **Traceability Links** - Multi-select from existing items:
   ```
   ? Select Use Cases this Scenario refines:
   > [x] UC-001 - User Login
     [ ] UC-002 - Password Reset
     [ ] UC-003 - Account Registration
   ```

6. **Type-specific fields** (if applicable):
   ```
   ? Specification: The system SHALL authenticate users within 2 seconds
   ```

7. **Confirmation** - Review before creating:
   ```
   📋 Summary:
     Type: Scenario
     ID:   SCEN-003
     Name: Customer Authentication Flow
     Refines: UC-001

   ? Create document? (Y/n)
   ```

### Canceling Interactive Mode

Press **Ctrl+C** at any time to cancel. No files will be created.

```
✖ Cancelled. No file was created.
```

### Non-TTY Environments

If running in a non-interactive environment (piped input, CI/CD, scripts), Sara will exit with an error:

```bash
echo "" | sara init docs/test.md
# ❌ Interactive mode requires a terminal. Use --type <TYPE> to specify the item type.
```

Use the `--type` flag for scripted/automated scenarios:

```bash
sara init docs/test.md --type solution --name "Scripted Solution" --id SOL-999
```

### Parent Item Enforcement

Interactive mode ensures proper traceability by checking that required parent items exist:

```
❌ Cannot create Scenario: no Use Cases exist. Create a Use Case first.
```

Create items in hierarchical order:
1. Solution (root - no parents required)
2. Use Case (requires Solution)
3. Scenario (requires Use Case)
4. And so on...

## Edit Command (Added 2026-01-14)

The `edit` command allows you to modify existing document metadata by item ID.

### Starting Edit Mode

```bash
# Interactive edit - prompts for all editable fields with current values as defaults
sara edit SREQ-001

# Non-interactive edit - update specific fields directly
sara edit SREQ-001 --name "New Requirement Name"

# Update multiple fields
sara edit SREQ-001 --name "Updated Name" --description "New description"

# Update traceability links
sara edit SREQ-001 --derives-from SCEN-002 SCEN-003
```

### Interactive Edit Flow

When running `sara edit <ID>` without modification flags, Sara enters interactive mode:

1. **Read-only header** - Shows immutable fields:
   ```
   📋 Editing SREQ-001 (System Requirement)
   ```

2. **Editable fields** - Each prompt shows current value as default:
   ```
   ? Item name [Performance Requirement]: High Performance Requirement
   ? Description [System response time] (Enter to keep):
   ```

3. **Traceability links** - Current selections pre-checked:
   ```
   ? Select Scenarios this requirement derives from:
   > [x] SCEN-001 - User Login Flow
     [x] SCEN-002 - Authentication (currently selected)
     [ ] SCEN-003 - Password Reset
   ```

4. **Change summary** - Review before applying:
   ```
   📋 Changes to apply:

     Name: Performance Requirement → High Performance Requirement
     Description: (unchanged)
     Derives from: SCEN-001 → SCEN-001, SCEN-002

   ? Apply changes? (Y/n)
   ```

### Non-Interactive Edit

Use flags to update specific fields without prompts:

```bash
# Update name only
sara edit UC-001 --name "Updated Use Case Name"

# Update specification for a requirement
sara edit SWREQ-005 --specification "The software SHALL process requests within 50ms"

# Update platform for an architecture
sara edit SARCH-001 --platform "AWS Lambda"

# Replace traceability links
sara edit SCEN-002 --refines UC-001 UC-002
```

### Immutable Fields

The following fields cannot be changed via edit:
- **ID** - Would break existing references to the item
- **Type** - Would violate relationship rules

To change these, create a new document and update references manually.

### Item Not Found

If the ID doesn't exist, Sara suggests similar IDs:

```
❌ Item not found: SREQ-099
Did you mean: SREQ-001, SREQ-009, SREQ-010?
```

### Canceling Edit

Press **Ctrl+C** at any time to cancel. No changes will be saved.

```
✖ Cancelled. No changes were made.
```

### Non-TTY Environments

In non-interactive environments, use modification flags:

```bash
# This will fail in CI/CD or piped input
sara edit SREQ-001
# ❌ Interactive mode requires a terminal. Use modification flags (--name, --description, etc.) to edit non-interactively.

# This works in any environment
sara edit SREQ-001 --name "Automated Update" --description "Updated via CI"
```

## Next Steps

- Read the [CLI Reference](contracts/cli.md) for all commands and options
- Review the [Data Model](data-model.md) for entity details
- Check the [Spec](spec.md) for full requirements

# Data Model: MCP Connector for SARA

**Date**: 2026-01-16
**Feature**: 002-mcp-connector

## Overview

The MCP connector acts as a bridge between AI assistants and the SARA knowledge graph. It does not introduce new persistent data models but instead exposes existing sara-core entities through the MCP protocol.

## Existing Entities (from sara-core)

These entities are already defined in sara-core and will be exposed via MCP:

### Item

Represents a requirement or architecture document in the knowledge graph.

| Field | Type | Description |
|-------|------|-------------|
| `id` | `ItemId` (String) | Unique identifier (e.g., "SOL-001", "SYSREQ-042") |
| `item_type` | `ItemType` | Type enum (Solution, UseCase, Scenario, etc.) |
| `name` | `String` | Human-readable name |
| `description` | `Option<String>` | Optional description |
| `source` | `SourceLocation` | File path and line number |
| `traceability` | `TraceabilityLinks` | Upstream and downstream relationships |

### ItemType

Enum representing the 9 document types in the hierarchy.

| Variant | YAML Value | Level |
|---------|------------|-------|
| `Solution` | `solution` | Top |
| `UseCase` | `use_case` | 2 |
| `Scenario` | `scenario` | 3 |
| `SystemRequirement` | `system_requirement` | 4 |
| `SystemArchitecture` | `system_architecture` | 5 |
| `HardwareRequirement` | `hardware_requirement` | 6 |
| `SoftwareRequirement` | `software_requirement` | 6 |
| `HardwareDetailedDesign` | `hardware_detailed_design` | 7 |
| `SoftwareDetailedDesign` | `software_detailed_design` | 7 |

### TraceabilityLinks

Upstream and downstream references for an item.

| Field | Type | Description |
|-------|------|-------------|
| `refines` | `Vec<ItemId>` | Items this refines (upstream) |
| `is_refined_by` | `Vec<ItemId>` | Items that refine this (downstream) |
| `derives_from` | `Vec<ItemId>` | Items this derives from (upstream) |
| `derives` | `Vec<ItemId>` | Items derived from this (downstream) |
| `satisfies` | `Vec<ItemId>` | Items this satisfies (upstream) |
| `is_satisfied_by` | `Vec<ItemId>` | Items satisfied by this (downstream) |

### ValidationReport

Results from validating the knowledge graph.

| Field | Type | Description |
|-------|------|-------------|
| `errors` | `Vec<ValidationError>` | Critical issues (broken refs, duplicates) |
| `warnings` | `Vec<ValidationWarning>` | Non-critical issues (orphans, hints) |
| `stats` | `ValidationStats` | Summary counts |

### CoverageReport

Coverage metrics for the knowledge graph.

| Field | Type | Description |
|-------|------|-------------|
| `total_items` | `usize` | Total items in graph |
| `covered_items` | `usize` | Items with complete traceability |
| `uncovered_items` | `Vec<ItemId>` | Items missing traceability |
| `coverage_by_type` | `HashMap<ItemType, TypeCoverage>` | Per-type breakdown |

## MCP-Specific Types

New types introduced by the MCP connector layer:

### McpServerState

Runtime state for the MCP server (not persisted).

| Field | Type | Description |
|-------|------|-------------|
| `config` | `Config` | Loaded sara.toml configuration |
| `graph` | `Option<KnowledgeGraph>` | Cached knowledge graph (lazy loaded) |
| `config_path` | `PathBuf` | Path to sara.toml |

**State Transitions**:
- `Uninitialized` → `ConfigLoaded` (on server start)
- `ConfigLoaded` → `GraphLoaded` (on first tool call requiring graph)
- `GraphLoaded` → `GraphLoaded` (on parse tool refresh)

### ToolResult

Wrapper for MCP tool responses.

| Field | Type | Description |
|-------|------|-------------|
| `success` | `bool` | Whether operation succeeded |
| `content` | `Vec<Content>` | MCP content blocks (text, json) |
| `is_error` | `bool` | Whether this is an error response |

### ResourceContent

Content returned for MCP resources.

| Field | Type | Description |
|-------|------|-------------|
| `uri` | `String` | Resource URI (e.g., "sara://items/SOL-001") |
| `mime_type` | `String` | Content type (application/json) |
| `content` | `String` | JSON-serialized data |

## Serialization Formats

All MCP responses use JSON serialization. Key formats:

### Query Response

```json
{
  "item": {
    "id": "SOL-001",
    "type": "solution",
    "name": "Customer Portal",
    "description": "Web-based customer self-service portal",
    "source": {
      "path": "docs/solutions/portal.md",
      "line": 1
    }
  },
  "traceability": {
    "upstream": [],
    "downstream": ["UC-001", "UC-002"]
  }
}
```

### Validation Response

```json
{
  "valid": false,
  "errors": [
    {
      "type": "broken_reference",
      "item_id": "SYSREQ-001",
      "message": "References non-existent item 'SCEN-999'",
      "suggestion": "Did you mean 'SCEN-001'?"
    }
  ],
  "warnings": [
    {
      "type": "orphan",
      "item_id": "SWREQ-042",
      "message": "Item has no upstream parent"
    }
  ],
  "summary": {
    "total_errors": 1,
    "total_warnings": 1
  }
}
```

### Coverage Response

```json
{
  "coverage_percentage": 85.5,
  "total_items": 100,
  "covered_items": 85,
  "by_type": {
    "solution": { "total": 5, "covered": 5, "percentage": 100.0 },
    "use_case": { "total": 20, "covered": 18, "percentage": 90.0 }
  },
  "uncovered": ["SWREQ-042", "HWREQ-015"]
}
```

## Validation Rules

The MCP connector enforces these validation rules:

### Tool Input Validation

| Rule | Tool | Validation |
|------|------|------------|
| Valid item ID | query, edit | Must match existing item or suggest alternatives |
| Valid item type | init | Must be one of 9 ItemType variants |
| Valid file path | init, edit | Must be within configured repository paths |
| Valid Git ref | diff | Must be valid commit, branch, or tag |

### Configuration Validation

| Rule | Validation |
|------|------------|
| Config exists | sara.toml must exist (search current + parent dirs) |
| Valid TOML | Must parse as valid TOML |
| Paths accessible | Repository paths must be readable |

## Entity Relationships

```
┌─────────────────┐
│  MCP Client     │
│  (AI Assistant) │
└────────┬────────┘
         │ MCP Protocol (JSON-RPC 2.0)
         ▼
┌─────────────────┐
│  sara-mcp       │
│  (MCP Server)   │
├─────────────────┤
│ McpServerState  │
│ - config        │
│ - graph (lazy)  │
└────────┬────────┘
         │ Rust API calls
         ▼
┌─────────────────┐
│  sara-core      │
│  (Library)      │
├─────────────────┤
│ KnowledgeGraph  │
│ - items         │
│ - relationships │
└────────┬────────┘
         │ File I/O
         ▼
┌─────────────────┐
│  Filesystem     │
│  (Markdown)     │
└─────────────────┘
```

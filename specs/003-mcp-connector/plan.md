# Implementation Plan: MCP Connector for SARA

**Branch**: `002-mcp-connector` | **Date**: 2026-01-16 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/002-mcp-connector/spec.md`

## Summary

Build an MCP (Model Context Protocol) server that exposes SARA's requirements knowledge graph capabilities to AI assistants. The server will use the `rmcp` crate to implement both stdio transport (for local IDE integrations) and HTTP/SSE transport with OAuth 2.0 authentication (for network access). This is a thin integration layer over the existing `sara-core` library.

## Technical Context

**Language/Version**: Rust 1.92 (2024 edition) - matching existing workspace configuration
**Primary Dependencies**: rmcp (MCP SDK), sara-core (existing), tokio (async runtime), serde (serialization)
**Storage**: N/A - leverages sara-core which reads from filesystem
**Testing**: cargo test (unit + integration tests)
**Target Platform**: Linux, macOS, Windows (cross-platform CLI tool)
**Project Type**: Single project - new crate `sara-mcp` in existing workspace
**Performance Goals**: Query response <5s, validation <30s, reports <10s for 10k items (per spec SC-001 to SC-003)
**Constraints**: Must integrate seamlessly with existing sara-core API, OAuth 2.0 for HTTP transport
**Scale/Scope**: Graphs up to 10,000 requirement items

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| **Simplicity First** | ✅ PASS | Thin wrapper over sara-core; no new abstractions; direct mapping of CLI commands to MCP tools |
| **Modern Standards** | ✅ PASS | Using rmcp (official MCP SDK), Rust 2024 edition, tokio async runtime, OAuth 2.0 per MCP spec |
| **Code Quality** | ✅ PASS | Will follow existing sara codebase patterns; explicit error handling via thiserror |
| **Testing Standards** | ✅ PASS | Plan includes unit tests for tool handlers, integration tests with mock MCP clients |
| **UX Consistency** | ✅ PASS | Error messages follow FR-010 (clear, actionable); tool responses match CLI output patterns |

**Gate Result**: PASS - No violations requiring justification.

## Project Structure

### Documentation (this feature)

```text
specs/002-mcp-connector/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (MCP tool schemas)
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
sara-mcp/                # New crate for MCP server
├── Cargo.toml
├── src/
│   ├── lib.rs           # Library exports
│   ├── main.rs          # CLI entry point (stdio or HTTP mode)
│   ├── server.rs        # MCP server implementation with tool_router
│   ├── tools/           # MCP tool implementations
│   │   ├── mod.rs
│   │   ├── query.rs     # Query tools (FR-001)
│   │   ├── validate.rs  # Validation tool (FR-002)
│   │   ├── report.rs    # Reporting tools (FR-003)
│   │   ├── init.rs      # Document initialization (FR-004)
│   │   ├── edit.rs      # Document editing (FR-005)
│   │   ├── diff.rs      # Git diff tool (FR-007)
│   │   └── parse.rs     # Parse/refresh tool (FR-012)
│   ├── resources/       # MCP resource implementations
│   │   ├── mod.rs
│   │   └── graph.rs     # Knowledge graph resources (FR-006)
│   ├── transport/       # Transport configuration
│   │   ├── mod.rs
│   │   ├── stdio.rs     # Stdio transport (FR-013)
│   │   └── http.rs      # HTTP/SSE with OAuth (FR-014, FR-015, FR-016)
│   └── error.rs         # MCP-specific error types

tests/
├── mcp_integration/     # Integration tests for MCP server
│   ├── mod.rs
│   ├── tools_test.rs
│   └── transport_test.rs
```

**Structure Decision**: New `sara-mcp` crate as a workspace member, following the existing pattern of `sara-core` (library) and `sara-cli` (binary). The MCP server is a separate binary that depends on `sara-core` for all business logic.

## Complexity Tracking

No violations requiring justification. The design maintains simplicity by:
- Reusing all business logic from sara-core (no duplication)
- Direct 1:1 mapping between CLI commands and MCP tools
- Using rmcp's macro-based tool definition for minimal boilerplate

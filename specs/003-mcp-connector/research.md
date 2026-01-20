# Research: MCP Connector for SARA

**Date**: 2026-01-16
**Feature**: 002-mcp-connector

## Research Tasks Completed

### 1. MCP Rust SDK Selection

**Decision**: Use `rmcp` crate (official Rust SDK for Model Context Protocol)

**Rationale**:
- Official SDK maintained by the MCP project (github.com/modelcontextprotocol/rust-sdk)
- 2.8k stars, 123 contributors, active development
- Provides `#[tool]` and `#[tool_router]` macros for minimal boilerplate
- Built on tokio async runtime (matches modern Rust patterns)
- Supports all required transports: stdio, HTTP/SSE
- OAuth 2.0 authentication support via `auth` feature

**Alternatives Considered**:
- Custom implementation: Rejected - would duplicate effort and miss spec updates
- Other MCP libraries: None as mature in Rust ecosystem

### 2. Transport Implementation Strategy

**Decision**: Support both stdio and HTTP/SSE transports with feature flags

**Configuration**:
```toml
[dependencies]
rmcp = { version = "0.9", features = [
    "server",
    "transport-io",                    # stdio support
    "transport-streamable-http-server", # HTTP/SSE support
    "auth"                              # OAuth 2.0
] }
```

**Rationale**:
- stdio is required for Claude Desktop, Cursor, VS Code integrations
- HTTP/SSE enables remote access and team sharing scenarios
- Feature flags keep binary size minimal for stdio-only users

**Alternatives Considered**:
- stdio only: Rejected - limits use cases per spec requirements FR-014
- HTTP only: Rejected - would break local IDE integrations

### 3. OAuth 2.0 Implementation

**Decision**: Implement as OAuth 2.0 Resource Server per MCP spec v2025-06-18

**Implementation Approach**:
- Use rmcp's `auth` feature which provides full OAuth 2.1 flow
- Implement PKCE for enhanced security
- Require resource parameter (RFC 8707) to bind tokens to server instance
- HTTPS-only for production; allow localhost for development

**Rationale**:
- Aligns with MCP specification requirements
- rmcp provides built-in support, minimizing custom code
- PKCE mitigates authorization code interception attacks

**Alternatives Considered**:
- Simple API key: Rejected - doesn't meet MCP spec requirements
- No authentication: Rejected - security risk for network access

### 4. Tool-to-Sara-Core Mapping

**Decision**: Direct 1:1 mapping between MCP tools and sara-core functions

| MCP Tool | Sara-Core Function | FR |
|----------|-------------------|-----|
| `sara_query` | `QueryEngine::lookup_item_or_suggest`, `traverse_upstream/downstream` | FR-001 |
| `sara_validate` | `Validator::validate` | FR-002 |
| `sara_coverage_report` | `CoverageReport::generate` | FR-003 |
| `sara_matrix_report` | `TraceabilityMatrix::generate` | FR-003 |
| `sara_init` | `InitService::init` | FR-004 |
| `sara_edit` | `EditService::edit` | FR-005 |
| `sara_diff` | `DiffService::diff` | FR-007 |
| `sara_parse` | `parse_repositories` | FR-012 |

**Rationale**:
- All business logic already exists in sara-core
- No new abstractions needed - simplicity principle
- Consistent behavior between CLI and MCP interfaces

### 5. Resource Implementation

**Decision**: Expose knowledge graph items as MCP resources

**Resource URIs**:
- `sara://items` - List all items in graph
- `sara://items/{id}` - Get specific item by ID
- `sara://stats` - Graph statistics (counts by type, relationship stats)

**Rationale**:
- Resources provide read-only browsing capability (FR-006)
- URI scheme follows MCP patterns
- Enables AI assistants to explore graph structure

### 6. Error Handling Strategy

**Decision**: Map sara-core errors to MCP error responses with actionable messages

**Error Mapping**:
```rust
impl From<SaraError> for McpError {
    fn from(err: SaraError) -> Self {
        match err {
            SaraError::ConfigNotFound(path) => McpError::invalid_params(
                format!("Configuration not found at {}. Create sara.toml with: sara init --config", path)
            ),
            SaraError::ItemNotFound(id) => McpError::invalid_params(
                format!("Item '{}' not found. Use sara_query to list available items.", id)
            ),
            // ... other mappings
        }
    }
}
```

**Rationale**:
- FR-010 requires clear, actionable error messages
- SC-007 targets 80% self-resolution rate
- Consistent with CLI error messaging patterns

### 7. Progress Reporting

**Decision**: Use MCP progress notifications for long-running operations

**Implementation**:
- Wrap sara-core operations with progress tracking
- Send `$/progress` notifications during parse, validate, report operations
- Include percentage complete and current operation description

**Rationale**:
- FR-011 requires progress reporting
- MCP protocol supports progress notifications natively
- Improves UX for large knowledge graphs

### 8. Configuration Discovery

**Decision**: Search for sara.toml in current directory and parent directories

**Algorithm**:
1. Check current working directory for `sara.toml`
2. If not found, traverse parent directories up to filesystem root
3. If still not found, return error with setup instructions (per Edge Case: Missing configuration)

**Rationale**:
- Matches common tool behavior (cargo, git)
- FR-008 requires automatic discovery
- Edge case specifies error with guidance when not found

### 9. Concurrency Model

**Decision**: Use tokio async runtime with file-level locking for writes

**Implementation**:
- Read operations: Concurrent, no locking
- Write operations: File-level advisory locks via `fs2` or similar
- Knowledge graph: Rebuilt on-demand (stateless server pattern)

**Rationale**:
- Edge Case specifies concurrent reads, locked writes
- Stateless pattern simplifies reasoning about consistency
- Tokio provides efficient async I/O

## Dependencies Summary

```toml
[dependencies]
rmcp = { version = "0.9", features = ["server", "transport-io", "transport-streamable-http-server", "auth"] }
sara-core = { path = "../sara-core" }
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
tracing = "0.1"
clap = { version = "4.5", features = ["derive"] }

[dev-dependencies]
tempfile = "3.24"
assert_cmd = "2.1"
```

## Open Questions Resolved

All technical questions have been resolved through research:

1. ✅ MCP SDK choice: rmcp
2. ✅ Transport strategy: Both stdio and HTTP/SSE
3. ✅ Authentication: OAuth 2.0 via rmcp `auth` feature
4. ✅ Tool mapping: Direct 1:1 to sara-core
5. ✅ Error handling: MCP errors with actionable messages
6. ✅ Progress reporting: MCP progress notifications
7. ✅ Configuration: Directory traversal discovery
8. ✅ Concurrency: Async runtime with file locking

## References

- [MCP Specification](https://modelcontextprotocol.io/specification/2025-11-25)
- [rmcp GitHub Repository](https://github.com/modelcontextprotocol/rust-sdk)
- [rmcp Documentation](https://docs.rs/rmcp)
- [OAuth Support in rmcp](https://github.com/modelcontextprotocol/rust-sdk/blob/main/docs/OAUTH_SUPPORT.md)

# Feature Specification: MCP Connector for SARA

**Feature Branch**: `002-mcp-connector`
**Created**: 2026-01-16
**Status**: Draft
**Input**: User description: "Build a MCP connector for sara application."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Query Requirements via AI Assistant (Priority: P1)

As an AI assistant user (such as Claude, ChatGPT, or Cursor), I want to query the SARA knowledge graph through natural language requests so that I can understand requirements, traceability, and architecture without leaving my AI conversation.

**Why this priority**: This is the core value proposition of the MCP connector - enabling AI assistants to access and query the requirements knowledge graph. Without this capability, the connector provides no value.

**Independent Test**: Can be fully tested by asking an AI assistant connected to the SARA MCP server "What requirements derive from scenario SCEN-001?" and receiving accurate traceability information. Delivers immediate value for requirements understanding.

**Acceptance Scenarios**:

1. **Given** a valid SARA knowledge graph exists with requirements documents, **When** an AI assistant requests to query an item by ID, **Then** the assistant receives the item's metadata, relationships, and content.

2. **Given** a SARA knowledge graph with established traceability links, **When** an AI assistant requests upstream or downstream traceability for an item, **Then** the assistant receives the complete traceability chain in a structured format.

3. **Given** a SARA configuration file exists in the project, **When** an AI assistant connects to the MCP server, **Then** the server automatically loads the knowledge graph from configured repository paths.

---

### User Story 2 - Validate Knowledge Graph Integrity (Priority: P2)

As an AI assistant user, I want to validate the SARA knowledge graph for integrity issues so that I can identify broken references, orphaned items, and circular dependencies through my AI conversation.

**Why this priority**: Validation is essential for maintaining knowledge graph quality, but it's secondary to basic querying since users need to understand the graph before validating it.

**Independent Test**: Can be fully tested by asking an AI assistant "Validate my SARA knowledge graph" and receiving a report of any validation issues found. Delivers value for quality assurance.

**Acceptance Scenarios**:

1. **Given** a SARA knowledge graph with broken references, **When** an AI assistant requests validation, **Then** the assistant receives a report listing all broken references with affected item IDs.

2. **Given** a SARA knowledge graph with circular dependencies, **When** an AI assistant requests validation, **Then** the assistant receives details about the cycle including the items involved.

3. **Given** a SARA knowledge graph with orphaned items, **When** an AI assistant requests validation with strict mode, **Then** orphaned items are reported as errors.

---

### User Story 4 - Initialize and Edit Documents (Priority: P3)

As an AI assistant user, I want to initialize new requirement documents and edit existing ones so that I can maintain my requirements directly through my AI conversation.

**Why this priority**: Document creation and editing are valuable but require more careful consideration for data integrity and are less frequently needed than read operations.

**Independent Test**: Can be fully tested by asking an AI assistant "Create a new system requirement that derives from SCEN-001" and having a properly formatted Markdown file with YAML frontmatter created. Delivers value for documentation maintenance.

**Acceptance Scenarios**:

1. **Given** a project with SARA configured, **When** an AI assistant requests to initialize a new requirement document of a specific type, **Then** a new Markdown file is created with proper YAML frontmatter template.

2. **Given** an existing requirement document, **When** an AI assistant requests to update its metadata, **Then** the document's YAML frontmatter is updated while preserving the content body.

---

### User Story 3 - Generate Coverage Reports (Priority: P4)

As an AI assistant user, I want to generate coverage and traceability reports so that I can understand the completeness of my requirements documentation through my AI conversation.

**Why this priority**: Reporting provides valuable insights for stakeholders but is less frequently needed than querying or validation.

**Independent Test**: Can be fully tested by asking an AI assistant "Generate a coverage report for my requirements" and receiving a summary of coverage metrics. Delivers value for requirements completeness assessment.

**Acceptance Scenarios**:

1. **Given** a SARA knowledge graph, **When** an AI assistant requests a coverage report, **Then** the assistant receives metrics showing which items have complete traceability.

2. **Given** a SARA knowledge graph, **When** an AI assistant requests a traceability matrix, **Then** the assistant receives a matrix showing relationships between items at different levels.

---

### Edge Cases

- **Missing configuration**: When the MCP server is started without a valid sara.toml, it MUST refuse to start and return an error message explaining how to create the configuration file, including a sample template.
- **Non-existent item ID**: When querying an item ID that doesn't exist, the system returns a clear "item not found" error with the requested ID.
- **File permission issues**: When the knowledge graph cannot be loaded due to file permissions, the system returns an error listing the affected paths and required permissions.
- **Concurrent operations**: Read operations are always allowed concurrently; write operations use file-level locking to prevent data corruption.
- **Dirty Git state**: Diff operations work with uncommitted changes; the system warns users when comparing against a dirty working tree.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST expose SARA query capabilities as MCP tools, allowing AI assistants to query items by ID with upstream/downstream traceability options.

- **FR-002**: System MUST expose SARA validation as an MCP tool, returning structured validation results including broken references, orphaned items, circular dependencies, and duplicate identifiers.

- **FR-003**: System MUST expose SARA reporting capabilities as MCP tools for coverage reports and traceability matrices.

- **FR-004**: System MUST expose SARA document initialization as an MCP tool, allowing creation of new requirement documents with proper YAML frontmatter.

- **FR-005**: System MUST expose SARA document editing as an MCP tool, allowing metadata updates to existing documents.

- **FR-006**: System MUST expose SARA knowledge graph resources, allowing AI assistants to browse available items and their metadata.

- **FR-007**: System MUST expose SARA diff capabilities as an MCP tool, allowing comparison between Git commits or branches.

- **FR-008**: System MUST automatically discover and load the SARA configuration from the project's sara.toml file on startup.

- **FR-009**: System MUST support the MCP roots feature to allow servers to understand the filesystem boundaries where SARA documents are located.

- **FR-010**: System MUST return clear, actionable error messages when operations fail, including suggestions for resolution.

- **FR-011**: System MUST support progress reporting for long-running operations such as parsing large knowledge graphs.

- **FR-012**: System MUST expose SARA parse capabilities as an MCP tool, allowing on-demand refresh of the knowledge graph from source documents.

- **FR-013**: System MUST support stdio transport for local process communication with IDE integrations (Claude Desktop, Cursor, VS Code).

- **FR-014**: System MUST support HTTP/SSE transport for network-based access, enabling remote AI assistant connections.

- **FR-015**: System MUST implement OAuth 2.0 authentication for HTTP/SSE transport, treating the MCP server as an OAuth 2.0 Resource Server per the MCP specification.

- **FR-016**: System MUST require clients to include a resource parameter (RFC 8707) when requesting tokens for HTTP/SSE transport, binding access tokens to the specific MCP server.

### Key Entities

- **MCP Server**: The SARA component that implements the MCP server protocol, exposing SARA capabilities to AI assistants.
- **Tool**: An MCP primitive representing an executable SARA operation (query, validate, report, init, edit, diff, parse).
- **Resource**: An MCP primitive representing read-only access to SARA knowledge graph data (items, relationships, statistics).
- **Configuration**: The sara.toml file that defines repository paths and settings, loaded automatically by the MCP server.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can query any item in the knowledge graph and receive complete traceability information within 5 seconds for graphs containing up to 10,000 items.

- **SC-002**: Users can validate the entire knowledge graph and receive a complete validation report within 30 seconds for graphs containing up to 10,000 items.

- **SC-003**: Users can generate coverage reports within 10 seconds for graphs containing up to 10,000 items.

- **SC-004**: Users can initialize new documents in under 2 seconds.

- **SC-005**: All SARA command-line capabilities are accessible through the MCP connector.

- **SC-006**: 90% of users can successfully connect their AI assistant to the SARA MCP server on their first attempt with provided documentation.

- **SC-007**: Error messages enable users to resolve issues without external support in 80% of cases.

## Clarifications

### Session 2026-01-16

- Q: Which MCP transport protocols should be supported? → A: Both stdio and HTTP/SSE for maximum flexibility
- Q: What authentication mechanism for HTTP/SSE transport? → A: OAuth 2.0 (per MCP spec)
- Q: Behavior when sara.toml is missing at startup? → A: Return error with guidance on how to create config

## Assumptions

- Users have an MCP-compatible AI assistant or host application (Claude Desktop, Cursor, VS Code with MCP extension, etc.)
- Users have SARA installed and configured with a valid sara.toml file
- The MCP server supports both stdio transport (for local IDE integrations) and HTTP/SSE transport (for network-based access)
- Users understand basic SARA concepts (items, traceability, validation) before using the MCP connector
- The existing sara-core library provides all necessary functionality; the MCP connector is a thin integration layer

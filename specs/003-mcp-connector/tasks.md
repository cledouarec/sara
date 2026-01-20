# Tasks: MCP Connector for SARA

**Input**: Design documents from `/specs/002-mcp-connector/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: No test tasks included (not explicitly requested in spec).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

- **Crate**: `sara-mcp/` as workspace member
- **Source**: `sara-mcp/src/`
- **Tests**: `sara-mcp/tests/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and crate structure

- [ ] T001 Create sara-mcp crate directory and add to workspace in Cargo.toml
- [ ] T002 Create sara-mcp/Cargo.toml with dependencies (rmcp, sara-core, tokio, serde, clap, thiserror, tracing)
- [ ] T003 [P] Create sara-mcp/src/lib.rs with module declarations
- [ ] T004 [P] Create sara-mcp/src/main.rs with CLI entry point (clap: --mode stdio|http, --port)
- [ ] T005 [P] Create sara-mcp/src/error.rs with MCP-specific error types and sara-core error mapping

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core MCP server infrastructure that MUST be complete before ANY user story

**CRITICAL**: No user story work can begin until this phase is complete

- [ ] T006 Create sara-mcp/src/state.rs with McpServerState (config, lazy graph loading)
- [ ] T007 Create sara-mcp/src/config.rs with configuration discovery (traverse parent dirs for sara.toml)
- [ ] T008 Create sara-mcp/src/server.rs with MCP server setup using rmcp tool_router macro
- [ ] T009 [P] Create sara-mcp/src/transport/mod.rs with transport module exports
- [ ] T010 [P] Create sara-mcp/src/transport/stdio.rs with stdio transport setup (FR-013)
- [ ] T011 [P] Create sara-mcp/src/tools/mod.rs with tools module exports
- [ ] T012 [P] Create sara-mcp/src/resources/mod.rs with resources module exports

**Checkpoint**: Foundation ready - MCP server can start (but has no tools yet)

---

## Phase 3: User Story 1 - Query Requirements via AI Assistant (Priority: P1)

**Goal**: Enable AI assistants to query the SARA knowledge graph by item ID with traceability support

**Independent Test**: Ask AI assistant "What requirements derive from scenario SCEN-001?" and receive accurate traceability information

### Implementation for User Story 1

- [ ] T013 [US1] Create sara-mcp/src/tools/query.rs with sara_query tool (item lookup, upstream/downstream traversal)
- [ ] T014 [US1] Create sara-mcp/src/tools/list.rs with sara_list_items tool (list items with optional type filter)
- [ ] T015 [US1] Create sara-mcp/src/tools/stats.rs with sara_stats tool (graph statistics)
- [ ] T016 [P] [US1] Create sara-mcp/src/resources/items.rs with sara://items and sara://items/{id} resources
- [ ] T017 [P] [US1] Create sara-mcp/src/resources/stats.rs with sara://stats resource
- [ ] T018 [US1] Register US1 tools and resources in sara-mcp/src/server.rs
- [ ] T019 [US1] Wire main.rs to start server with stdio transport and US1 capabilities

**Checkpoint**: User Story 1 complete - AI can query items, list items, and get stats

---

## Phase 4: User Story 2 - Validate Knowledge Graph Integrity (Priority: P2)

**Goal**: Enable AI assistants to validate the knowledge graph for integrity issues

**Independent Test**: Ask AI assistant "Validate my SARA knowledge graph" and receive validation report

### Implementation for User Story 2

- [ ] T020 [US2] Create sara-mcp/src/tools/validate.rs with sara_validate tool (broken refs, orphans, cycles, duplicates)
- [ ] T021 [P] [US2] Create sara-mcp/src/resources/validation.rs with sara://validation resource
- [ ] T022 [US2] Register US2 tools and resources in sara-mcp/src/server.rs

**Checkpoint**: User Story 2 complete - AI can validate graph integrity

---

## Phase 5: User Story 3 - Initialize and Edit Documents (Priority: P3)

**Goal**: Enable AI assistants to create new requirement documents and edit existing ones

**Independent Test**: Ask AI assistant "Create a new system requirement that derives from SCEN-001" and verify file is created

### Implementation for User Story 3

- [ ] T023 [US3] Create sara-mcp/src/tools/init.rs with sara_init tool (create new document with frontmatter)
- [ ] T024 [US3] Create sara-mcp/src/tools/edit.rs with sara_edit tool (update document metadata)
- [ ] T025 [US3] Register US3 tools in sara-mcp/src/server.rs

**Checkpoint**: User Story 3 complete - AI can create and edit documents

---

## Phase 6: User Story 4 - Generate Coverage Reports (Priority: P4)

**Goal**: Enable AI assistants to generate coverage and traceability reports

**Independent Test**: Ask AI assistant "Generate a coverage report for my requirements" and receive metrics

### Implementation for User Story 4

- [ ] T026 [US4] Create sara-mcp/src/tools/report.rs with sara_coverage_report and sara_matrix_report tools
- [ ] T027 [US4] Register US4 tools in sara-mcp/src/server.rs

**Checkpoint**: User Story 4 complete - AI can generate reports

---

## Phase 7: Additional Tools (FR-007, FR-012)

**Goal**: Complete remaining functional requirements for diff and parse

- [ ] T028 Create sara-mcp/src/tools/diff.rs with sara_diff tool (compare Git references)
- [ ] T029 Create sara-mcp/src/tools/parse.rs with sara_parse tool (refresh knowledge graph)
- [ ] T030 Register diff and parse tools in sara-mcp/src/server.rs

**Checkpoint**: All core tools implemented

---

## Phase 8: HTTP Transport & OAuth (FR-014, FR-015, FR-016)

**Goal**: Enable network-based access with OAuth 2.0 authentication

- [ ] T031 Create sara-mcp/src/transport/http.rs with HTTP/SSE transport setup
- [ ] T032 Implement OAuth 2.0 Resource Server in sara-mcp/src/transport/http.rs (PKCE, RFC 8707 resource parameter)
- [ ] T033 Update sara-mcp/src/main.rs to support --mode http with --port and --oauth flags
- [ ] T034 Add HTTPS enforcement for production (allow localhost for development)

**Checkpoint**: HTTP transport with OAuth complete

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T035 [P] Add progress reporting for long-running operations (FR-011) in sara-mcp/src/progress.rs
- [ ] T036 [P] Create sara-mcp/src/resources/config.rs with sara://config resource
- [ ] T037 [P] Create sara-mcp/src/resources/types.rs with sara://types/{item_type} resource
- [ ] T038 Add MCP roots support (FR-009) in sara-mcp/src/server.rs
- [ ] T039 Validate all error messages are actionable (FR-010, SC-007)
- [ ] T040 Run quickstart.md validation with Claude Desktop integration
- [ ] T041 Update workspace README.md with sara-mcp installation instructions

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-6)**: All depend on Foundational phase completion
  - User stories can proceed sequentially in priority order (P1 -> P2 -> P3 -> P4)
  - Or in parallel if staffed
- **Additional Tools (Phase 7)**: Can run after Foundational, parallel to user stories
- **HTTP Transport (Phase 8)**: Depends on at least US1 complete (needs tools to expose)
- **Polish (Phase 9)**: Depends on core user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational - Independent of US1
- **User Story 3 (P3)**: Can start after Foundational - Independent of US1/US2
- **User Story 4 (P4)**: Can start after Foundational - Independent of other stories

### Within Each User Story

- Tools before resources (resources may use tool logic)
- Register tools/resources after implementing them
- Story complete before moving to next priority

### Parallel Opportunities

- T003, T004, T005 can run in parallel (different files)
- T009, T010, T011, T012 can run in parallel (different modules)
- T016, T017 can run in parallel (different resource files)
- T035, T036, T037 can run in parallel (different files)
- Different user stories can be worked on in parallel by different team members

---

## Parallel Example: Phase 2 (Foundational)

```bash
# After T006-T008 complete (sequential server setup):

# Launch transport and module setup in parallel:
Task: "Create sara-mcp/src/transport/mod.rs"
Task: "Create sara-mcp/src/transport/stdio.rs"
Task: "Create sara-mcp/src/tools/mod.rs"
Task: "Create sara-mcp/src/resources/mod.rs"
```

## Parallel Example: User Story 1

```bash
# After T013-T015 complete (tools):

# Launch resources in parallel:
Task: "Create sara-mcp/src/resources/items.rs"
Task: "Create sara-mcp/src/resources/stats.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Test with Claude Desktop - can query items?
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational -> Foundation ready
2. Add User Story 1 -> Test with Claude Desktop -> MVP!
3. Add User Story 2 -> Test validation -> Deploy/Demo
4. Add User Story 3 -> Test init/edit -> Deploy/Demo
5. Add User Story 4 -> Test reports -> Deploy/Demo
6. Add HTTP transport -> Enable remote access
7. Each story adds value without breaking previous stories

### Tool-to-Sara-Core Mapping Reference

| MCP Tool | Sara-Core Function | User Story |
|----------|-------------------|------------|
| `sara_query` | `QueryEngine::lookup_item_or_suggest`, `traverse_upstream/downstream` | US1 |
| `sara_list_items` | `KnowledgeGraph::items()`, `items_by_type()` | US1 |
| `sara_stats` | `GraphStats::from_graph()` | US1 |
| `sara_validate` | `Validator::validate()` | US2 |
| `sara_init` | `InitService::init()` | US3 |
| `sara_edit` | `EditService::edit()` | US3 |
| `sara_coverage_report` | `CoverageReport::generate()` | US4 |
| `sara_matrix_report` | `TraceabilityMatrix::generate()` | US4 |
| `sara_diff` | `DiffService::diff()` | Phase 7 |
| `sara_parse` | `parse_repositories()` | Phase 7 |

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- All MCP tools are thin wrappers over sara-core functions

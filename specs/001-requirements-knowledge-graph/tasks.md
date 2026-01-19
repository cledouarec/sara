# Tasks: Requirements Knowledge Graph CLI

**Input**: Design documents from `/specs/001-requirements-knowledge-graph/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/cli.md

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

```text
Cargo.toml               # Workspace manifest
sara-core/               # Library crate (all business logic)
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── model/
    ├── graph/
    ├── parser/
    ├── repository/
    ├── validation/
    ├── query/
    ├── report/
    ├── config/
    ├── template/
    └── error.rs

sara-cli/                # Binary crate (CLI interface)
├── Cargo.toml
└── src/
    ├── main.rs
    ├── commands/
    ├── output/
    └── logging.rs

tests/                   # Integration tests
├── fixtures/
└── cli_tests.rs
```

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [x] T001 Create Rust workspace with Cargo.toml at repository root
- [x] T002 [P] Create sara-core library crate in sara-core/Cargo.toml with dependencies (petgraph, serde, serde_yaml, thiserror, tracing, toml, glob)
- [x] T003 [P] Create sara-cli binary crate in sara-cli/Cargo.toml with dependencies (clap, colored, console, tracing-subscriber, sara-core)
- [x] T004 [P] Configure clippy.toml and rustfmt.toml for code quality
- [x] T005 Create directory structure for sara-core/src/ modules (model/, graph/, parser/, repository/, validation/, query/, report/, config/, template/)
- [x] T006 Create directory structure for sara-cli/src/ modules (commands/, output/)
- [x] T007 Create tests/fixtures/ directory with subdirectories (valid_graph/, broken_refs/, orphans/, duplicates/, cycles/)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**Blocking**: No user story work can begin until this phase is complete

### Core Domain Model

- [x] T008 [P] Implement ItemType enum in sara-core/src/model/item.rs
- [x] T009 [P] Implement ItemId value object in sara-core/src/model/item.rs
- [x] T010 [P] Implement RelationshipType enum in sara-core/src/model/relationship.rs
- [x] T011 [P] Implement SourceLocation struct in sara-core/src/model/metadata.rs
- [x] T012 Implement Item struct with UpstreamRefs/DownstreamRefs/ItemAttributes in sara-core/src/model/item.rs
- [x] T013 Implement Relationship struct in sara-core/src/model/relationship.rs
- [x] T014 Create model module exports in sara-core/src/model/mod.rs

### Error Types

- [x] T015 [P] Implement error types (ParseError, ValidationError, ConfigError, QueryError) in sara-core/src/error.rs

### Configuration

- [x] T016 Implement Config, RepositoryConfig, ValidationConfig, OutputConfig, TemplatesConfig structs in sara-core/src/config/settings.rs
- [x] T017 Implement TOML config file loading in sara-core/src/config/mod.rs
- [x] T018 [P] Implement glob pattern expansion for template paths in sara-core/src/config/settings.rs

### Knowledge Graph Core

- [x] T019 Implement KnowledgeGraph struct with petgraph DiGraph in sara-core/src/graph/knowledge_graph.rs
- [x] T020 Implement GraphBuilder with add_item() and build() in sara-core/src/graph/builder.rs
- [x] T021 Create graph module exports in sara-core/src/graph/mod.rs

### CLI Framework

- [x] T022 [P] Implement CLI argument parsing with clap derive in sara-cli/src/main.rs
- [x] T023 [P] Implement output formatter with colors/emojis in sara-cli/src/output/formatter.rs
- [x] T024 [P] Implement logging configuration in sara-cli/src/logging.rs
- [x] T025 Create lib.rs exports in sara-core/src/lib.rs

### Test Fixtures

- [x] T026 [P] Create valid graph test fixtures in tests/fixtures/valid_graph/ (SOL-001.md, UC-001.md, SCEN-001.md, SYSREQ-001.md, SYSARCH-001.md, SWREQ-001.md, SWDD-001.md)

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Validate Documentation Integrity (Priority: P1)

**Goal**: Validate that all architecture and requirements documents are properly linked and complete, detecting broken references, orphaned items, circular references, and duplicate identifiers.

**Independent Test**: Run `sara validate` against test fixtures and verify it reports all validation errors with file locations and line numbers.

### Parsing Infrastructure for US1

- [x] T027 [P] [US1] Implement YAML frontmatter extraction in sara-core/src/parser/frontmatter.rs
- [x] T028 [P] [US1] Implement Markdown file parsing in sara-core/src/parser/markdown.rs
- [x] T029 [US1] Implement ItemBuilder for constructing items from parsed frontmatter in sara-core/src/model/item.rs
- [x] T030 [US1] Create parser module exports in sara-core/src/parser/mod.rs

### Validation Rules for US1

- [x] T031 [P] [US1] Implement broken reference detection in sara-core/src/validation/rules/broken_refs.rs
- [x] T032 [P] [US1] Implement orphan item detection (configurable strict/permissive) in sara-core/src/validation/rules/orphans.rs
- [x] T033 [P] [US1] Implement duplicate identifier detection in sara-core/src/validation/rules/duplicates.rs
- [x] T034 [P] [US1] Implement circular reference detection using petgraph in sara-core/src/validation/rules/cycles.rs
- [x] T035 [P] [US1] Implement metadata validation (required fields, specification field) in sara-core/src/validation/rules/metadata.rs
- [x] T036 [P] [US1] Implement relationship type validation in sara-core/src/validation/rules/relationships.rs
- [x] T037 [US1] Create validation rules module exports in sara-core/src/validation/rules/mod.rs

### Validation Engine for US1

- [x] T038 [US1] Implement ValidationReport struct (errors, warnings, summary) in sara-core/src/validation/report.rs
- [x] T039 [US1] Implement Validator orchestrating all rules in sara-core/src/validation/validator.rs
- [x] T040 [US1] Create validation module exports in sara-core/src/validation/mod.rs

### CLI Command for US1

- [x] T041 [US1] Implement `validate` command in sara-cli/src/commands/validate.rs
- [x] T042 [US1] Implement validation output formatting (text/JSON) in sara-cli/src/output/formatter.rs
- [x] T043 [US1] Add validate to commands module in sara-cli/src/commands/mod.rs

### Test Fixtures for US1

- [x] T044 [P] [US1] Create broken reference test fixtures in tests/fixtures/broken_refs/
- [x] T045 [P] [US1] Create orphan item test fixtures in tests/fixtures/orphans/
- [x] T046 [P] [US1] Create duplicate identifier test fixtures in tests/fixtures/duplicates/
- [x] T047 [P] [US1] Create circular reference test fixtures in tests/fixtures/cycles/

**Checkpoint**: User Story 1 complete - `sara validate` works with all validation rules

---

## Phase 4: User Story 2 - Parse Multi-Repository Document Graph (Priority: P1)

**Goal**: Parse and aggregate Markdown files from multiple Git repositories into a unified knowledge graph.

**Independent Test**: Configure multiple repository paths in sara.toml and verify `sara parse` scans all repositories and builds a unified graph.

### Repository Scanning for US2

- [x] T048 [P] [US2] Implement file scanner for discovering Markdown files in sara-core/src/repository/scanner.rs
- [x] T049 [P] [US2] Implement git2 integration for reading from Git repos in sara-core/src/repository/git.rs
- [x] T050 [US2] Implement multi-repository aggregation in sara-core/src/repository/mod.rs

### Graph Building for US2

- [x] T051 [US2] Implement cross-repository reference resolution in sara-core/src/graph/builder.rs
- [x] T052 [US2] Implement parallel file parsing with rayon in sara-core/src/parser/mod.rs

### CLI Command for US2

- [x] T053 [US2] Implement `parse` command in sara-cli/src/commands/parse.rs
- [x] T054 [US2] Implement parse output formatting (summary) in sara-cli/src/output/formatter.rs
- [x] T055 [US2] Implement JSON graph export option in sara-cli/src/commands/parse.rs
- [x] T056 [US2] Add parse to commands module in sara-cli/src/commands/mod.rs

### Git Reference Support for US2

- [x] T057 [US2] Implement --at flag for reading from specific Git commit/branch in sara-core/src/repository/git.rs

**Checkpoint**: User Story 2 complete - `sara parse` works with multiple repositories

---

## Phase 5: User Story 3 - Query Traceability Chains (Priority: P2)

**Goal**: Query the complete traceability chain from any item upstream (toward Solution) or downstream (toward Detailed Designs).

**Independent Test**: Run `sara query <ITEM_ID> --upstream` and `sara query <ITEM_ID> --downstream` and verify complete traceability chains are displayed.

### Graph Traversal for US3

- [x] T058 [P] [US3] Implement upstream traversal (toward Solution) in sara-core/src/graph/traversal.rs
- [x] T059 [P] [US3] Implement downstream traversal (toward Detailed Designs) in sara-core/src/graph/traversal.rs
- [x] T060 [US3] Implement depth-limited traversal in sara-core/src/graph/traversal.rs
- [x] T061 [US3] Implement type filtering for traversal results in sara-core/src/graph/traversal.rs

### Query Engine for US3

- [x] T062 [US3] Implement item lookup by ID with fuzzy suggestions in sara-core/src/query/traceability.rs
- [x] T063 [US3] Create query module exports in sara-core/src/query/mod.rs

### CLI Command for US3

- [x] T064 [US3] Implement `query` command in sara-cli/src/commands/query.rs
- [x] T065 [US3] Implement tree output format for traceability chains in sara-cli/src/output/formatter.rs
- [x] T066 [US3] Implement list and JSON output formats in sara-cli/src/output/formatter.rs
- [x] T067 [US3] Add query to commands module in sara-cli/src/commands/mod.rs

**Checkpoint**: User Story 3 complete - `sara query` works with upstream/downstream traversal

---

## Phase 6: User Story 4 - Generate Traceability Reports (Priority: P2)

**Goal**: Generate coverage and traceability reports in exportable formats (text, JSON, CSV, HTML).

**Independent Test**: Run `sara report coverage` and `sara report matrix` and verify correctly formatted output containing expected traceability information.

### Report Generation for US4

- [x] T068 [P] [US4] Implement coverage report calculation in sara-core/src/report/coverage.rs
- [x] T069 [P] [US4] Implement traceability matrix generation in sara-core/src/report/matrix.rs
- [x] T070 [US4] Create report module exports in sara-core/src/report/mod.rs

### CLI Command for US4

- [x] T071 [US4] Implement `report` command in sara-cli/src/commands/report.rs
- [x] T072 [US4] Implement text output format for reports in sara-cli/src/output/formatter.rs
- [x] T073 [US4] Implement CSV output format for reports in sara-cli/src/output/formatter.rs
- [x] T074 [US4] Implement JSON output format for reports in sara-cli/src/output/formatter.rs
- [x] T075 [US4] Add report to commands module in sara-cli/src/commands/mod.rs

**Checkpoint**: User Story 4 complete - `sara report coverage` and `sara report matrix` work with multiple output formats

---

## Phase 7: User Story 5 - Manage Document Metadata (Priority: P3)

**Goal**: Help authors add and validate metadata in Markdown files using the `init` command with full support for all frontmatter fields.

**Independent Test**: Run `sara init <FILE> --type system_requirement --derives-from SCEN-001 --specification "The system SHALL..."` and verify correct YAML frontmatter is added with all specified fields.

### Template System for US5

- [x] T076 [P] [US5] Implement built-in Tera templates for all 9 item types in sara-core/templates/*.tera
- [x] T077 [P] [US5] Implement custom template loading from config paths in sara-core/src/template/loader.rs
- [x] T078 [US5] Implement Tera template rendering with GeneratorOptions context in sara-core/src/template/generator.rs
- [x] T079 [US5] Create template module exports in sara-core/src/template/mod.rs

### Generator Options for US5

- [x] T080 [US5] Implement GeneratorOptions struct with all frontmatter fields in sara-core/src/template/generator.rs
- [x] T081 [US5] Add with_description() builder method in sara-core/src/template/generator.rs
- [x] T082 [US5] Add with_refines() builder method in sara-core/src/template/generator.rs
- [x] T083 [US5] Add with_derives_from() builder method in sara-core/src/template/generator.rs
- [x] T084 [US5] Add with_satisfies() builder method in sara-core/src/template/generator.rs
- [x] T085 [US5] Add with_specification() builder method in sara-core/src/template/generator.rs
- [x] T086 [US5] Add with_platform() builder method for system_architecture in sara-core/src/template/generator.rs

### CLI Command for US5

- [x] T087 [US5] Implement `init` command with all options in sara-cli/src/commands/init.rs
- [x] T088 [US5] Add -d/--description flag to init command in sara-cli/src/commands/init.rs
- [x] T089 [US5] Add --refines flag (for use_case, scenario) in sara-cli/src/commands/init.rs
- [x] T090 [US5] Add --derives-from flag (for requirements) in sara-cli/src/commands/init.rs
- [x] T091 [US5] Add --satisfies flag (for architectures, designs) in sara-cli/src/commands/init.rs
- [x] T092 [US5] Add --specification flag (for requirements) in sara-cli/src/commands/init.rs
- [x] T093 [US5] Add --platform flag (for system_architecture) in sara-cli/src/commands/init.rs
- [x] T094 [US5] Implement type-specific option validation (exit code 3 for invalid options) in sara-cli/src/commands/init.rs
- [x] T095 [US5] Implement frontmatter insertion/replacement in files in sara-cli/src/commands/init.rs
- [x] T096 [US5] Add init to commands module in sara-cli/src/commands/mod.rs

**Checkpoint**: User Story 5 complete - `sara init` generates documents from templates with full frontmatter options

---

## Phase 8: User Story 6 - Track Changes Across Versions (Priority: P3)

**Goal**: Compare the knowledge graph between Git commits or branches to understand documentation evolution.

**Independent Test**: Run `sara diff main HEAD` and verify added/removed/modified items and relationships are correctly identified.

### Graph Comparison for US6

- [x] T097 [US6] Implement graph snapshot at Git reference in sara-core/src/repository/git.rs
- [x] T098 [US6] Implement graph diff algorithm (added, removed, modified) in sara-core/src/graph/diff.rs

### CLI Command for US6

- [x] T099 [US6] Implement `diff` command in sara-cli/src/commands/diff.rs
- [x] T100 [US6] Implement diff output formatting (text/JSON) in sara-cli/src/output/formatter.rs
- [x] T101 [US6] Add diff to commands module in sara-cli/src/commands/mod.rs

**Checkpoint**: User Story 6 complete - `sara diff` compares graphs between Git references

---

## Phase 9: User Story 5 Extension - Interactive Mode (Priority: P3)

**Goal**: Add interactive mode to the `sara init` command that activates when `--type` is not provided. Users will be guided through creating requirement documents via terminal prompts.

**Independent Test**: Run `sara init docs/test.md` without --type flag and verify interactive prompts guide through item creation with type selection, name input, ID suggestion, traceability multi-select, and confirmation.

**Requirements**: FR-040 through FR-052

### Setup for Interactive Mode

- [x] T110 Add inquire = "0.9" dependency in sara-cli/Cargo.toml

### Interactive Session Types

- [x] T111 [P] [US5-IM] Implement PrefilledFields struct in sara-cli/src/commands/interactive.rs
- [x] T112 [P] [US5-IM] Implement InteractiveSession struct in sara-cli/src/commands/interactive.rs
- [x] T113 [P] [US5-IM] Implement InteractiveInput and TraceabilityInput structs in sara-cli/src/commands/interactive.rs
- [x] T114 [P] [US5-IM] Implement PromptError enum with thiserror in sara-cli/src/commands/interactive.rs
- [x] T115 [P] [US5-IM] Implement SelectOption struct with Display trait in sara-cli/src/commands/interactive.rs

### TTY Detection (FR-051)

- [x] T116 [US5-IM] Implement require_tty() function using std::io::IsTerminal in sara-cli/src/commands/interactive.rs

### Parent Item Enforcement (FR-052)

- [x] T117 [US5-IM] Add required_parent_type() method to ItemType in sara-core/src/model/item.rs
- [x] T118 [US5-IM] Add traceability_field() method to ItemType in sara-core/src/model/item.rs
- [x] T119 [US5-IM] Implement check_parent_exists() function in sara-cli/src/commands/interactive.rs

### ID Generation (FR-044)

- [x] T120 [US5-IM] Implement suggest_next_id() function using graph analysis in sara-cli/src/commands/interactive.rs

### Interactive Prompts

- [x] T121 [US5-IM] Implement prompt_item_type() with Select (FR-041) in sara-cli/src/commands/interactive.rs
- [x] T122 [US5-IM] Implement prompt_name() with Text and validator (FR-042) in sara-cli/src/commands/interactive.rs
- [x] T123 [US5-IM] Implement prompt_description() with Text, optional (FR-043) in sara-cli/src/commands/interactive.rs
- [x] T124 [US5-IM] Implement prompt_identifier() with suggested default (FR-044) in sara-cli/src/commands/interactive.rs
- [x] T125 [US5-IM] Implement prompt_traceability() with MultiSelect (FR-045) in sara-cli/src/commands/interactive.rs
- [x] T126 [US5-IM] Implement prompt_specification() for requirement types (FR-046) in sara-cli/src/commands/interactive.rs
- [x] T127 [US5-IM] Implement prompt_platform() for system_architecture (FR-046) in sara-cli/src/commands/interactive.rs
- [x] T128 [US5-IM] Implement prompt_confirmation() with summary display (FR-048) in sara-cli/src/commands/interactive.rs

### Interactive Session Orchestration

- [x] T129 [US5-IM] Implement run_interactive_session() orchestrating all prompts in sara-cli/src/commands/interactive.rs
- [x] T130 [US5-IM] Handle Ctrl+C via InquireError::OperationInterrupted (FR-049) in sara-cli/src/commands/interactive.rs
- [x] T131 [US5-IM] Implement skip logic for pre-provided CLI args (FR-050) in sara-cli/src/commands/interactive.rs

### CLI Integration (FR-040)

- [x] T132 [US5-IM] Add mod interactive; to sara-cli/src/commands/mod.rs
- [x] T133 [US5-IM] Modify run_init() to call interactive mode when --type missing in sara-cli/src/commands/init.rs
- [x] T134 [US5-IM] Implement conversion from InteractiveInput to GeneratorOptions in sara-cli/src/commands/init.rs

### Validation Integration

- [x] T135 [US5-IM] Add ID format validator (alphanumeric, hyphens, underscores) in sara-cli/src/commands/interactive.rs
- [x] T136 [US5-IM] Add name length validator in sara-cli/src/commands/interactive.rs

**Checkpoint**: Interactive Mode complete - `sara init` without --type enters guided mode

---

## Phase 10: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] ~~T102 [P] Add progress indicators for long operations~~ (removed - parser is fast enough)
- [x] T103 [P] Implement --no-color and --no-emoji flags in sara-cli/src/main.rs
- [x] T104 [P] Implement SARA_CONFIG and NO_COLOR environment variable support in sara-cli/src/main.rs
- [x] T105 [P] Add custom field warning (FR-019) in sara-core/src/validation/rules/metadata.rs
- [x] T106 Integration tests for CLI commands in tests/cli_tests.rs
- [x] T107 Performance optimization for 500-document target (SC-001) in sara-core/src/parser/mod.rs
- [x] T108 Cross-platform path handling verification (Windows, macOS, Linux)
- [x] T109 Run quickstart.md validation scenarios

### Interactive Mode Polish

- [x] T137 [P] Add interactive mode section to CLI help output in sara-cli/src/commands/mod.rs
- [x] T138 Interactive mode integration tests in tests/cli_tests.rs
- [x] T139 Run interactive mode quickstart.md validation scenarios

---

## Phase 11: CLI Help Grouping (FR-053) ✅

**Goal**: Organize CLI help output into logical groups using help headings to improve readability and discoverability of related options.

**Independent Test**: Run `sara init --help` and verify options are grouped under headings like "Item Properties", "Traceability", "Type-Specific".

**Requirements**: FR-053

### Init Command Help Grouping

- [x] T140 [P] Add "Item Properties" help heading for --type, --id, --name, --description in sara-cli/src/commands/mod.rs
- [x] T141 [P] Add "Traceability" help heading for --refines, --derives-from, --satisfies in sara-cli/src/commands/mod.rs
- [x] T142 [P] Add "Type-Specific" help heading for --specification, --platform in sara-cli/src/commands/mod.rs

### Other Commands Help Grouping

- [x] T143 [P] Add help headings to Validate command options (--strict, --at, --format, --output) in sara-cli/src/commands/mod.rs
- [x] T144 [P] Add help headings to Query command options (--upstream, --downstream, --type, --depth, --format) in sara-cli/src/commands/mod.rs
- [x] T145 [P] Add help headings to Parse command options (--repository, --at, --output) in sara-cli/src/commands/mod.rs
- [x] T146 [P] Add help headings to Report subcommands options in sara-cli/src/commands/mod.rs
- [x] T147 [P] Add help headings to Diff command options in sara-cli/src/commands/mod.rs

### Global Options Grouping

- [x] T148 Add global help headings for -c/--config, -v/--verbose, -q/--quiet, --no-color, --no-emoji in sara-cli/src/main.rs

### Verification

- [x] T149 Update CLI integration tests to verify help headings appear in sara-cli/tests/cli_tests.rs
- [x] T150 Run `sara --help` and `sara <command> --help` for all commands to verify grouping

**Checkpoint**: FR-053 complete - CLI help output is organized with logical groupings

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational - Can start immediately after Phase 2
- **User Story 2 (Phase 4)**: Depends on Foundational - Can run in parallel with US1
- **User Story 3 (Phase 5)**: Depends on US1 and US2 (needs parsing and graph)
- **User Story 4 (Phase 6)**: Depends on US1 and US2 (needs parsing and graph)
- **User Story 5 (Phase 7)**: Depends on Foundational only - Can run in parallel with US1/US2
- **User Story 6 (Phase 8)**: Depends on US2 (needs Git integration and parsing)
- **Interactive Mode (Phase 9)**: Depends on US5 (extends init command) and US2 (needs graph for traceability)
- **Polish (Phase 10)**: Depends on all user stories being complete
- **CLI Help Grouping (Phase 11)**: Depends on all commands being implemented (Phase 10)

### User Story Dependencies

```
Phase 1: Setup
    ↓
Phase 2: Foundational (BLOCKING)
    ↓
    ├── US1: Validate (P1) ──────────┬──→ US3: Query (P2)
    │                                │
    ├── US2: Parse (P1) ─────────────┼──→ US4: Report (P2)
    │                                │
    │                                └──→ US6: Diff (P3)
    │
    └── US5: Init (P3) ──────────────────→ US5-IM: Interactive Mode (P3)
    ↓
Phase 10: Polish
    ↓
Phase 11: CLI Help Grouping (FR-053)
```

### Parallel Opportunities

**Within Phase 2 (Foundational)**:
- T002, T003, T004 can run in parallel
- T008, T009, T010, T011 can run in parallel
- T015, T022, T023, T024 can run in parallel

**After Phase 2 (User Stories)**:
- US1 and US2 can run in parallel (both P1)
- US5 can run in parallel with US1/US2
- US3 and US4 can run in parallel once US1+US2 complete

**Within Each User Story**:
- All tasks marked [P] within a story can run in parallel

---

## Parallel Example: Phase 2 Foundational

```bash
# Launch all model definitions together:
Task: "Implement ItemType enum in sara-core/src/model/item.rs"
Task: "Implement ItemId value object in sara-core/src/model/item.rs"
Task: "Implement RelationshipType enum in sara-core/src/model/relationship.rs"
Task: "Implement SourceLocation struct in sara-core/src/model/metadata.rs"
```

## Parallel Example: User Story 1

```bash
# Launch all validation rules together:
Task: "Implement broken reference detection in sara-core/src/validation/rules/broken_refs.rs"
Task: "Implement orphan item detection in sara-core/src/validation/rules/orphans.rs"
Task: "Implement duplicate identifier detection in sara-core/src/validation/rules/duplicates.rs"
Task: "Implement circular reference detection in sara-core/src/validation/rules/cycles.rs"
Task: "Implement metadata validation in sara-core/src/validation/rules/metadata.rs"
Task: "Implement relationship type validation in sara-core/src/validation/rules/relationships.rs"
```

## Parallel Example: Interactive Mode (Phase 9)

```bash
# Launch all type definitions together (T111-T115):
Task: "Implement PrefilledFields struct in sara-cli/src/commands/interactive.rs"
Task: "Implement InteractiveSession struct in sara-cli/src/commands/interactive.rs"
Task: "Implement InteractiveInput and TraceabilityInput structs in sara-cli/src/commands/interactive.rs"
Task: "Implement PromptError enum with thiserror in sara-cli/src/commands/interactive.rs"
Task: "Implement SelectOption struct with Display trait in sara-cli/src/commands/interactive.rs"
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (Validation)
4. Complete Phase 4: User Story 2 (Parsing)
5. **STOP and VALIDATE**: Test parsing and validation independently
6. Deploy/demo if ready - this delivers core value proposition

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add US1 + US2 → Test independently → **MVP ready!**
3. Add US3 (Query) → Test independently → Enhanced CLI
4. Add US4 (Reports) → Test independently → Stakeholder communication
5. Add US5 (Init) → Test independently → Author productivity
6. Add US6 (Diff) → Test independently → Version tracking
7. Add US5-IM (Interactive Mode) → Test independently → Enhanced author experience
8. Add FR-053 (CLI Help Grouping) → Verify help output → Improved discoverability
9. Add US7 (Edit Command) → Test independently → Document maintenance capability
10. Each story adds value without breaking previous stories

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- [US5-IM] = User Story 5 Interactive Mode extension
- [US7] = User Story 7 Edit Command
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Performance target: 500 documents in <1 second (SC-001)
- Error messages must include file:line in 95% of cases (SC-005)
- Interactive mode uses `inquire` crate v0.9+ for terminal prompts
- TTY detection uses std::io::IsTerminal (Rust 1.70+)
- Interactive mode requires graph access for traceability suggestions (FR-045)
- Edit command reuses interactive mode prompts with pre-populated defaults (FR-056)
- strsim crate v0.11+ provides Levenshtein distance for fuzzy ID suggestions (FR-061)

---

## Phase 12: User Story 7 - Edit Existing Document Metadata (Priority: P3)

**Goal**: Allow users to edit existing document metadata by item identifier, with interactive mode (pre-populated prompts) and non-interactive mode (CLI flags).

**Independent Test**: Run `sara edit SREQ-001` and verify interactive prompts show current values as defaults. Run `sara edit SREQ-001 --name "New Name"` and verify only the name is updated.

**Requirements**: FR-054 through FR-066 (User Story 7)

### strsim Dependency for Fuzzy Suggestions

- [x] T151 Add strsim = "0.11" dependency in sara-cli/Cargo.toml for Levenshtein distance (FR-061)

### Edit Command Types (sara-core)

- [x] T152 [P] [US7] Implement EditUpdates struct in sara-core/src/model/edit.rs (name, description, refines, derives_from, satisfies, specification, platform options)
- [x] T153 [P] [US7] Implement EditSummary and FieldChange structs in sara-core/src/model/edit.rs
- [x] T154 [P] [US7] Implement EditError enum with thiserror in sara-core/src/error.rs (ItemNotFound with suggestions, NonInteractiveTerminal, Cancelled, InvalidLink)
- [x] T155 [US7] Add edit module exports in sara-core/src/model/mod.rs

### Item Lookup with Fuzzy Suggestions (FR-054, FR-061)

- [x] T156 [US7] Implement find_similar_ids() function using strsim::levenshtein in sara-core/src/query/traceability.rs
- [x] T157 [US7] Implement lookup_item_or_suggest() function returning Item or EditError::ItemNotFound with suggestions in sara-core/src/query/traceability.rs
- [x] T158 [US7] Add similarity and lookup module exports in sara-core/src/query/mod.rs

### Frontmatter Update (FR-064)

- [x] T159 [US7] Implement extract_body() function to extract content after YAML frontmatter in sara-core/src/parser/frontmatter.rs
- [x] T160 [US7] Implement update_frontmatter() function preserving body content in sara-core/src/parser/frontmatter.rs

### Interactive Edit Prompts (Refactor interactive.rs)

- [x] T161 [P] [US7] Refactor prompt_name() to accept optional default parameter in sara-cli/src/commands/interactive.rs
- [x] T162 [P] [US7] Refactor prompt_description() to accept optional default parameter in sara-cli/src/commands/interactive.rs
- [x] T163 [P] [US7] Refactor prompt_specification() to accept optional default parameter in sara-cli/src/commands/interactive.rs
- [x] T164 [P] [US7] Refactor prompt_platform() to accept optional default parameter in sara-cli/src/commands/interactive.rs
- [x] T165 [US7] Refactor prompt_traceability() to accept pre-selected items parameter in sara-cli/src/commands/interactive.rs

### Edit Command Implementation (sara-cli)

- [x] T166 [US7] Implement Edit command struct with clap derive in sara-cli/src/commands/mod.rs (id argument + modification flags)
- [x] T167 [US7] Add "Item Properties" help heading for --name, --description in Edit command in sara-cli/src/commands/mod.rs
- [x] T168 [US7] Add "Traceability" help heading for --refines, --derives-from, --satisfies in Edit command in sara-cli/src/commands/mod.rs
- [x] T169 [US7] Add "Type-Specific" help heading for --specification, --platform in Edit command in sara-cli/src/commands/mod.rs
- [x] T170 [US7] Implement run_edit() entry point in sara-cli/src/commands/edit.rs

### Interactive Edit Mode (FR-055, FR-056, FR-062, FR-063)

- [x] T171 [US7] Implement display_edit_header() showing immutable ID and type in sara-cli/src/commands/edit.rs
- [x] T172 [US7] Implement run_interactive_edit() orchestrating all prompts with defaults in sara-cli/src/commands/edit.rs
- [x] T173 [US7] Implement build_change_summary() comparing old vs new values in sara-cli/src/commands/edit.rs
- [x] T174 [US7] Implement display_change_summary() with diff-style output in sara-cli/src/commands/edit.rs
- [x] T175 [US7] Implement prompt_edit_confirmation() with "Apply changes?" in sara-cli/src/commands/edit.rs

### Non-Interactive Edit Mode (FR-057, FR-058)

- [x] T176 [US7] Implement apply_updates() applying EditUpdates to frontmatter in sara-cli/src/commands/edit.rs
- [x] T177 [US7] Implement type-specific field validation (--specification only for requirements, --platform only for architecture) in sara-cli/src/commands/edit.rs

### Traceability Validation (FR-065)

- [x] T178 [US7] Implement validate_traceability_links() checking references exist in graph in sara-cli/src/commands/edit.rs

### TTY Detection (FR-066)

- [x] T179 [US7] Implement require_tty_for_edit() with edit-specific error message in sara-cli/src/commands/edit.rs

### Ctrl+C Handling (FR-062)

- [x] T180 [US7] Implement Ctrl+C handling via InquireError::OperationInterrupted in sara-cli/src/commands/edit.rs

### CLI Integration

- [x] T181 [US7] Add mod edit; to sara-cli/src/commands/mod.rs
- [x] T182 [US7] Add Edit variant to Commands enum in sara-cli/src/commands/mod.rs
- [x] T183 [US7] Add edit command routing in sara-cli/src/main.rs

### Test Fixtures for US7

- [x] T184 [P] [US7] Create test fixture for existing item to edit in tests/fixtures/edit_tests/
- [x] T185 [P] [US7] Create test fixture with similar IDs for fuzzy suggestion testing in tests/fixtures/edit_tests/

### Integration Tests

- [x] T186 [US7] Add integration test for `sara edit <ID>` interactive mode in tests/cli_tests.rs
- [x] T187 [US7] Add integration test for `sara edit <ID> --name "..."` non-interactive mode in tests/cli_tests.rs
- [x] T188 [US7] Add integration test for item not found with suggestions in tests/cli_tests.rs
- [x] T189 [US7] Add integration test for traceability link validation in tests/cli_tests.rs
- [x] T190 [US7] Run edit command quickstart.md validation scenarios

**Checkpoint**: User Story 7 complete - `sara edit` works in both interactive and non-interactive modes

---

## Updated Dependencies & Execution Order

### Phase Dependencies (Updated)

- **Phase 12 (Edit Command)**: Depends on Phase 9 (Interactive Mode) for prompt reuse, Phase 4 (Parsing) for graph access

### User Story Dependencies (Updated)

```
Phase 1: Setup
    ↓
Phase 2: Foundational (BLOCKING)
    ↓
    ├── US1: Validate (P1) ──────────┬──→ US3: Query (P2)
    │                                │
    ├── US2: Parse (P1) ─────────────┼──→ US4: Report (P2)
    │                                │
    │                                └──→ US6: Diff (P3)
    │
    └── US5: Init (P3) ──────────────────→ US5-IM: Interactive Mode (P3)
                                                        ↓
                                           US7: Edit Command (P3)
    ↓
Phase 10: Polish
    ↓
Phase 11: CLI Help Grouping (FR-053)
```

### Parallel Opportunities for Phase 12

**Edit Types (T152-T154)**:
```bash
# Launch all type definitions together:
Task: "Implement EditUpdates struct in sara-core/src/model/edit.rs"
Task: "Implement EditSummary and FieldChange structs in sara-core/src/model/edit.rs"
Task: "Implement EditError enum in sara-core/src/error.rs"
```

**Interactive Prompt Refactoring (T161-T164)**:
```bash
# Launch all prompt refactors together:
Task: "Refactor prompt_name() to accept optional default parameter"
Task: "Refactor prompt_description() to accept optional default parameter"
Task: "Refactor prompt_specification() to accept optional default parameter"
Task: "Refactor prompt_platform() to accept optional default parameter"
```

**Test Fixtures (T184-T185)**:
```bash
# Launch test fixture creation together:
Task: "Create test fixture for existing item to edit"
Task: "Create test fixture with similar IDs for fuzzy suggestion"
```

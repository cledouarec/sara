# Tasks: ADR Management with Design Linking

**Input**: Design documents from `/specs/002-adr-management/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/cli-interface.md

**Tests**: Not explicitly requested - test tasks omitted (add if needed)

**Organization**: Tasks grouped by user story for independent implementation and testing

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

- **sara-core**: `sara-core/src/` (library crate)
- **sara-cli**: `sara-cli/src/` (binary crate)
- **Templates**: `sara-core/templates/`
- **Tests**: `sara-core/tests/`, `sara-cli/tests/`

---

## Phase 1: Setup

**Purpose**: Branch preparation and verification

- [X] T001 Verify on feature branch `002-adr-management` and run `cargo build && cargo test`
- [X] T002 Run `cargo clippy` to ensure clean baseline before changes

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core model layer changes that MUST be complete before ANY user story can be implemented

**CRITICAL**: No user story work can begin until this phase is complete

### Model Layer - Types and Enums

- [X] T003 [P] Add `AdrStatus` enum (Proposed, Accepted, Deprecated, Superseded) with serde in `sara-core/src/model/item.rs`
- [X] T004 [P] Add `ArchitectureDecisionRecord` variant to `ItemType` enum in `sara-core/src/model/item.rs`
- [X] T005 [P] Add `Justifies`, `IsJustifiedBy`, `Supersedes`, `IsSupersededBy` variants to `RelationshipType` enum in `sara-core/src/model/relationship.rs`
- [X] T006 [P] Add `Status`, `Deciders`, `Justifies`, `Supersedes`, `SupersededBy` variants to `FieldName` enum in `sara-core/src/model/field.rs`

### Model Layer - ItemType Implementation

- [X] T007 Implement `ItemType::ArchitectureDecisionRecord` methods: `prefix()` returns "ADR", `display_name()` returns "Architecture Decision Record" in `sara-core/src/model/item.rs`
- [X] T008 Implement `ItemType::ArchitectureDecisionRecord` in `required_parent_type()` returning `None` in `sara-core/src/model/item.rs`
- [X] T009 Implement `ItemType::ArchitectureDecisionRecord` in `as_str()` returning "architecture_decision_record" in `sara-core/src/model/item.rs`
- [X] T010 Add `ItemType::ArchitectureDecisionRecord` to `ItemType::all()` array in `sara-core/src/model/item.rs`
- [X] T011 Implement `ItemType::ArchitectureDecisionRecord` in `traceability_configs()` returning empty vec (no hierarchical parent) in `sara-core/src/model/item.rs`

### Model Layer - RelationshipType Implementation

- [X] T012 Implement `inverse()` for new relationship types (Justifies↔IsJustifiedBy, Supersedes↔IsSupersededBy) in `sara-core/src/model/relationship.rs`
- [X] T013 Add ADR relationship validation rules: `is_valid_justification()` and `is_valid_supersession()` in `sara-core/src/model/relationship.rs`

### Model Layer - Attributes Refactor (Based on research.md)

- [X] T014 Add `TypeSpecificAttributes::Adr { status: AdrStatus, deciders: Vec<String> }` variant in `sara-core/src/model/item.rs`
- [X] T015 Add `UpstreamRefs::Adr { justifies: Vec<ItemId> }` variant for ADR upstream relationships in `sara-core/src/model/item.rs`
- [X] T016 Add `justified_by: Vec<ItemId>` field to `UpstreamRefs::Design` variant in `sara-core/src/model/item.rs`
- [X] T017 Add `DownstreamRefs::IsJustificationFor { is_justification_for: Vec<ItemId> }` variant in `sara-core/src/model/item.rs`
- [X] T018 Add `PeerRefs::Supersedes { supersedes: Vec<ItemId>, superseded_by: Option<ItemId> }` variant in `sara-core/src/model/item.rs`

### Verification

- [X] T019 Run `cargo check -p sara-core` to verify model layer compiles
- [X] T020 Run `cargo test -p sara-core model::` to verify existing model tests pass

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Create and Link ADR to Design Artifact (Priority: P1)

**Goal**: Enable architects to create ADRs and link them to design artifacts (SYSARCH, SWDD, HWDD)

**Independent Test**: Create a new ADR document, link it to a design artifact, verify the link is persisted

### Parser Layer

- [X] T021 [US1] Extend `RawFrontmatter` with ADR fields: `status`, `deciders`, `justifies`, `supersedes`, `superseded_by` in `sara-core/src/parser/markdown.rs`
- [X] T022 [US1] Add ADR field parsing logic in `parse_item()` to create `TypeSpecificAttributes::Adr` in `sara-core/src/parser/markdown.rs`
- [X] T023 [US1] Add ADR validation: require `deciders` non-empty for ADR type in `sara-core/src/parser/markdown.rs`

### Graph Layer

- [X] T024 [US1] Extend `GraphBuilder::add_item_relationships()` to handle `UpstreamRefs::Adr { justifies }` in `sara-core/src/graph/builder.rs`
- [X] T025 [US1] Extend `GraphBuilder` to handle `UpstreamRefs::Design { justified_by }` for IsJustifiedBy edges in `sara-core/src/graph/builder.rs`
- [X] T026 [US1] Extend `GraphBuilder` to handle `PeerRefs::Supersedes` for Supersedes edges in `sara-core/src/graph/builder.rs`

### Template Layer

- [X] T027 [P] [US1] Create ADR Tera template with MADR-based structure in `sara-core/templates/adr.tera`
- [X] T028 [US1] Register ADR template in `TemplateGenerator` for item type `ArchitectureDecisionRecord` in `sara-core/src/template/generator.rs`

### CLI Layer - Create ADR

- [X] T029 [US1] Add ADR support to `sara init` with options: `--status`, `--deciders`, `--justifies`, `--supersedes` in `sara-cli/src/commands/init.rs`
- [X] T030 [US1] Implement ADR number auto-generation (next available ADR-nnn) via `generate_id(ItemType::ArchitectureDecisionRecord, None)`
- [ ] T031 [US1] Implement ADR file path generation: `docs/adr/ADR-{number}-{slug}.md` in `sara-cli/src/commands/new.rs`

### CLI Layer - Link ADR

- [ ] T032 [US1] Create `sara link adr <ADR-ID> <ARTIFACT-ID>...` command in `sara-cli/src/commands/link.rs` (new file)
- [ ] T033 [US1] Implement link validation: ADR must exist, artifacts must be SYSARCH/SWDD/HWDD in `sara-cli/src/commands/link.rs`
- [ ] T034 [US1] Implement bidirectional link update: add to ADR's `justifies` and artifact's `justified_by` in `sara-cli/src/commands/link.rs`

### Verification

- [X] T035 [US1] Run `cargo check -p sara-cli` to verify CLI compiles
- [ ] T036 [US1] Manual test: `sara init docs/adr/ADR-001.md --type adr --name "Test Decision" --deciders "Alice" --status proposed`

**Checkpoint**: User Story 1 complete - ADRs can be created and linked to design artifacts

---

## Phase 4: User Story 2 - Navigate Between ADRs and Design Artifacts (Priority: P2)

**Goal**: Enable bidirectional navigation from ADRs to artifacts and vice versa

**Independent Test**: View a design artifact, see related ADRs; view an ADR, navigate to linked artifacts

### CLI Layer - Show ADR

- [ ] T037 [US2] Extend `sara show` to detect ADR items and display ADR-specific fields (status, deciders, justifies) in `sara-cli/src/commands/show.rs`
- [ ] T038 [US2] Format ADR display showing linked artifacts with their names and types in `sara-cli/src/commands/show.rs`
- [ ] T039 [US2] Add `--with-content` flag to include markdown body in output in `sara-cli/src/commands/show.rs`

### CLI Layer - Show Design Artifact with ADRs

- [ ] T040 [US2] Extend `sara show` for SYSARCH/SWDD/HWDD to display `justified_by` ADR list in `sara-cli/src/commands/show.rs`
- [ ] T041 [US2] Format "No ADRs reference this artifact" message when `justified_by` is empty in `sara-cli/src/commands/show.rs`

### CLI Layer - Query Relationships

- [ ] T042 [US2] Create `sara query adr` command with `--justifying`, `--justified-by` options in `sara-cli/src/commands/query.rs` (new file or extend existing)
- [ ] T043 [US2] Implement `--justifying <ARTIFACT-ID>`: find all ADRs that justify given artifact in `sara-cli/src/commands/query.rs`
- [ ] T044 [US2] Implement `--justified-by <ADR-ID>`: find all artifacts justified by given ADR in `sara-cli/src/commands/query.rs`
- [ ] T045 [US2] Implement `--supersession-chain <ADR-ID>`: trace full supersession history in `sara-cli/src/commands/query.rs`

### Verification

- [ ] T046 [US2] Manual test: `sara show SYSARCH-001` displays linked ADRs
- [ ] T047 [US2] Manual test: `sara query adr --justifying SYSARCH-001` returns expected ADRs

**Checkpoint**: User Story 2 complete - Bidirectional navigation works

---

## Phase 5: User Story 3 - Update ADR Status and Lifecycle (Priority: P2)

**Goal**: Enable status updates (Proposed→Accepted→Deprecated→Superseded) and supersession tracking

**Independent Test**: Change ADR status, link superseding ADR, verify changes persisted

### CLI Layer - Update ADR

- [ ] T048 [US3] Create `sara update adr <ADR-ID>` command with `--status`, `--superseded-by`, `--add-justifies`, `--remove-justifies` options in `sara-cli/src/commands/update.rs` (new file or extend existing)
- [ ] T049 [US3] Implement status update: modify frontmatter `status` field in `sara-cli/src/commands/update.rs`
- [ ] T050 [US3] Implement `--superseded-by <ADR-ID>`: set `superseded_by` and update status to `superseded` in `sara-cli/src/commands/update.rs`
- [ ] T051 [US3] Implement `--add-justifies <ARTIFACT-ID>`: append to `justifies` list in `sara-cli/src/commands/update.rs`
- [ ] T052 [US3] Implement `--remove-justifies <ARTIFACT-ID>`: remove from `justifies` list in `sara-cli/src/commands/update.rs`

### Frontmatter Update Logic

- [ ] T053 [US3] Create utility function to read, modify, and write YAML frontmatter preserving markdown body in `sara-cli/src/utils/frontmatter.rs` (new file)
- [ ] T054 [US3] Handle concurrent edit scenarios: validate ADR exists before update in `sara-cli/src/commands/update.rs`

### Verification

- [ ] T055 [US3] Manual test: `sara update adr ADR-001 --status accepted`
- [ ] T056 [US3] Manual test: `sara update adr ADR-001 --superseded-by ADR-002`

**Checkpoint**: User Story 3 complete - ADR lifecycle management works

---

## Phase 6: User Story 4 - Search and Filter ADRs (Priority: P3)

**Goal**: Enable searching and filtering ADRs by status, linked artifact type, or keywords

**Independent Test**: Create multiple ADRs, filter by status, verify correct results

### CLI Layer - List ADRs

- [ ] T057 [US4] Create `sara list adr` subcommand with `--status`, `--justifies`, `--format`, `--sort` options in `sara-cli/src/commands/list.rs`
- [ ] T058 [US4] Implement `--status <STATUS>` filter: show only ADRs with given status in `sara-cli/src/commands/list.rs`
- [ ] T059 [US4] Implement `--justifies <ARTIFACT-ID>` filter: show only ADRs justifying given artifact in `sara-cli/src/commands/list.rs`
- [ ] T060 [US4] Implement `--format table` output (default): ID, Status, Name columns in `sara-cli/src/commands/list.rs`
- [ ] T061 [US4] Implement `--format json` output: structured JSON with full ADR details in `sara-cli/src/commands/list.rs`
- [ ] T062 [US4] Implement `--sort` option: sort by id (default), status, or name in `sara-cli/src/commands/list.rs`

### CLI Layer - Active ADRs Query

- [ ] T063 [US4] Implement `--active-for <ARTIFACT-ID>` in `sara query adr`: show only accepted ADRs for artifact in `sara-cli/src/commands/query.rs`

### Verification

- [ ] T064 [US4] Manual test: `sara list adr --status accepted`
- [ ] T065 [US4] Manual test: `sara list adr --format json`

**Checkpoint**: User Story 4 complete - Search and filter works

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Validation, integration, and documentation

### Validation Extension

- [ ] T066 [P] Add ADR validation rules to `sara validate`: required fields, valid status, valid justifies targets in `sara-cli/src/commands/validate.rs`
- [ ] T067 [P] Add warning for superseded ADR without `superseded_by` in `sara-cli/src/commands/validate.rs`
- [ ] T068 [P] Add warning for orphan ADR (no justifies links) in `sara-cli/src/commands/validate.rs`
- [ ] T069 [P] Add error for circular supersession detection in `sara-cli/src/commands/validate.rs`

### Integration

- [ ] T070 Extend `sara build` to include ADRs in graph construction in `sara-cli/src/commands/build.rs`
- [ ] T071 Extend `sara graph` to include ADR nodes and justification edges in `sara-cli/src/commands/graph.rs`

### Testing

- [ ] T072 [P] Create ADR model unit tests in `sara-core/tests/unit/adr_tests.rs`
- [ ] T073 [P] Create ADR parser unit tests in `sara-core/tests/unit/adr_parser_tests.rs`
- [ ] T074 Create ADR graph integration tests in `sara-core/tests/integration/adr_graph_tests.rs`
- [ ] T075 Run full test suite: `cargo test --workspace`

### Final Verification

- [ ] T076 Run `cargo clippy --workspace` and fix any warnings
- [ ] T077 Run quickstart.md validation: execute all sample commands
- [ ] T078 Update CLAUDE.md if needed with ADR-specific commands

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - start immediately
- **Foundational (Phase 2)**: Depends on Setup - BLOCKS all user stories
- **User Stories (Phase 3-6)**: All depend on Foundational phase completion
  - US1 (Phase 3): Can start after Foundational
  - US2 (Phase 4): Can start after US1 (needs ADRs to exist for navigation)
  - US3 (Phase 5): Can start after US1 (needs ADRs to exist for updates)
  - US4 (Phase 6): Can start after US1 (needs ADRs to exist for listing)
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational - No dependencies on other stories
- **User Story 2 (P2)**: Depends on US1 (needs create/link functionality)
- **User Story 3 (P2)**: Depends on US1 (needs ADRs to exist)
- **User Story 4 (P3)**: Depends on US1 (needs ADRs to exist)

### Within Each User Story

- Parser before graph builder
- Graph builder before CLI commands
- Template before CLI `new` command
- Core implementation before integration

### Parallel Opportunities

- All Foundational tasks marked [P] can run in parallel (T003-T006)
- Template creation (T027) can run in parallel with parser/graph work
- US2, US3, US4 can potentially run in parallel after US1 completes
- All Polish validation tasks marked [P] can run in parallel (T066-T069)
- Unit test tasks marked [P] can run in parallel (T072-T073)

---

## Parallel Example: Foundational Phase

```bash
# Launch all model type additions together:
Task: "Add AdrStatus enum in sara-core/src/model/item.rs"
Task: "Add ArchitectureDecisionRecord variant in sara-core/src/model/item.rs"
Task: "Add Justifies/IsJustifiedBy/Supersedes/IsSupersededBy in sara-core/src/model/relationship.rs"
Task: "Add Status/Deciders/Justifies/Supersedes/SupersededBy in sara-core/src/model/field.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Test ADR creation and linking independently
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational -> Foundation ready
2. Add User Story 1 -> Test independently -> Deploy (MVP: create ADRs!)
3. Add User Story 2 -> Test independently -> Deploy (navigation)
4. Add User Story 3 -> Test independently -> Deploy (lifecycle)
5. Add User Story 4 -> Test independently -> Deploy (search/filter)
6. Each story adds value without breaking previous stories

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence

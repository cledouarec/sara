# Implementation Plan: Requirements Knowledge Graph CLI

**Branch**: `001-requirements-knowledge-graph` | **Date**: 2026-01-14 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/001-requirements-knowledge-graph/spec.md`

## Summary

Build a command-line application (Sara) that manages Architecture documents and Requirements as a unified interconnected knowledge graph. The tool parses Markdown files with YAML frontmatter from multiple Git repositories, validates traceability chains, and provides querying and reporting capabilities. Built in Rust for cross-platform support (Windows, macOS, Linux) with a performance target of processing 500 documents in under 1 second.

## Technical Context

**Language/Version**: Rust 1.75+ (2021 edition)
**Primary Dependencies**:
- `petgraph` - Graph representation and traversal
- `clap` v4 - CLI argument parsing with derive macros
- `serde` + `serde_yaml` - YAML frontmatter deserialization
- `git2` - Git repository operations
- `thiserror` - Error type definitions
- `colored` + `console` - Terminal output with colors and emojis
- `tracing` + `tracing-subscriber` - Structured logging
- `toml` - Configuration file parsing
- `inquire` - Interactive terminal prompts
- `strsim` - String similarity for ID suggestions

**Storage**: File-based (Markdown files with YAML frontmatter)
**Testing**: `cargo test` (unit + integration tests)
**Target Platform**: Windows, macOS, Linux (cross-platform CLI)
**Project Type**: Single project with workspace (sara-core library + sara-cli binary)
**Performance Goals**: 500 documents in <1 second (SC-001)
**Constraints**: <2ms per document average processing time
**Scale/Scope**: Up to 10 repositories, 500+ documents

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| **P1: Simplicity First** | ✅ PASS | Single crate workspace, direct graph implementation, no over-abstraction |
| **P2: Modern Standards** | ✅ PASS | Rust 2021 edition, all dependencies actively maintained (2024-2026) |
| **P3: Code Quality** | ✅ PASS | clippy + rustfmt enforced, thiserror for explicit error handling |
| **P4: Testing Standards** | ✅ PASS | Unit tests for core logic, integration tests for CLI commands |
| **P5: UX Consistency** | ✅ PASS | Consistent CLI patterns, colored output, helpful error messages with file:line |

## Project Structure

### Documentation (this feature)

```text
specs/001-requirements-knowledge-graph/
├── plan.md              # This file
├── spec.md              # Feature specification (66 FRs, 7 user stories)
├── research.md          # Technology decisions (30 decisions)
├── data-model.md        # Entity definitions and types
├── quickstart.md        # User guide and examples
├── contracts/           # CLI interface contracts
└── tasks.md             # Implementation tasks (150 tasks across 12 phases)
```

### Source Code (repository root)

```text
sara-core/
├── src/
│   ├── lib.rs           # Library entry point
│   ├── model/           # Domain entities (Item, ItemType, Relationship)
│   ├── graph/           # KnowledgeGraph, traversal, diff
│   ├── parser/          # Markdown and frontmatter parsing
│   ├── validation/      # Validation rules and reports
│   ├── query/           # Traceability queries
│   ├── report/          # Coverage and matrix reports
│   ├── repository/      # File scanning and Git operations
│   ├── template/        # Document generation
│   └── config/          # Configuration loading
└── Cargo.toml

sara-cli/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── commands/        # Subcommand implementations
│   │   ├── mod.rs       # Command enum and routing
│   │   ├── parse.rs     # Parse command
│   │   ├── validate.rs  # Validate command
│   │   ├── query.rs     # Query command
│   │   ├── report.rs    # Report command
│   │   ├── init.rs      # Init command
│   │   ├── interactive.rs # Interactive mode prompts
│   │   ├── edit.rs      # Edit command (NEW)
│   │   └── diff.rs      # Diff command
│   ├── output/          # Output formatting
│   └── logging/         # Logging configuration
├── tests/
│   └── cli_tests.rs     # Integration tests
└── Cargo.toml

tests/
└── fixtures/            # Test document fixtures
```

**Structure Decision**: Workspace with two crates (sara-core library, sara-cli binary) to separate core logic from CLI concerns. This enables potential future use of sara-core as a library.

## Implementation Status

### Completed Phases

| Phase | Description | Status |
|-------|-------------|--------|
| 1 | Setup (project structure, dependencies) | ✅ Complete |
| 2 | Foundational (data model, parsing) | ✅ Complete |
| 3 | User Story 1 - Validation | ✅ Complete |
| 4 | User Story 2 - Parsing | ✅ Complete |
| 5 | User Story 3 - Query | ✅ Complete |
| 6 | User Story 4 - Reports | ✅ Complete |
| 7 | User Story 5 - Init Command | ✅ Complete |
| 8 | User Story 6 - Diff Command | ✅ Complete |
| 9 | Interactive Mode (FR-040 to FR-052) | ✅ Complete |
| 10 | Polish (clippy, edge cases) | ✅ Complete |
| 11 | CLI Help Grouping (FR-053) | ✅ Complete |

### Pending Phases

| Phase | Description | Status | Requirements |
|-------|-------------|--------|--------------|
| 12 | Edit Command (FR-054 to FR-066) | 🔲 Pending | User Story 7 |

## Next Implementation: Edit Command (Phase 12)

**Requirements**: FR-054 to FR-066 (User Story 7)

**Key Decisions** (from research.md):
- Reuse interactive mode infrastructure for prompts
- Enter interactive mode when no modification flags provided
- Pre-populate prompts with current values as defaults
- Type and ID are immutable (read-only display)
- Show diff-style change summary before applying
- Use Levenshtein distance for "not found" suggestions

**New Files**:
- `sara-cli/src/commands/edit.rs` - Edit command implementation

**Modified Files**:
- `sara-cli/src/commands/mod.rs` - Add Edit command to enum
- `sara-cli/src/commands/interactive.rs` - Refactor prompt functions for reuse with defaults

**Test Coverage**:
- Unit tests for edit logic
- Integration tests for `sara edit` command
- Tests for item not found with suggestions
- Tests for non-interactive edit mode

## Complexity Tracking

No constitution violations requiring justification. The project follows all 5 principles.

## Artifacts Generated

- [research.md](research.md) - 30 technology decisions including Edit Command research
- [data-model.md](data-model.md) - Complete entity definitions including Edit Command types
- [quickstart.md](quickstart.md) - User guide with Interactive Mode and Edit Command sections
- [contracts/cli.md](contracts/cli.md) - CLI interface specification

# Implementation Plan: ADR Management with Design Linking

**Branch**: `002-adr-management` | **Date**: 2026-01-26 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/002-adr-management/spec.md`

## Summary

Extend the SARA knowledge graph system to support Architecture Decision Records (ADRs) linked to design artifacts (SYSARCH, SWDD, HWDD). ADRs are a new ItemType with bidirectional Justifies/IsJustifiedBy relationships to design artifacts and Supersedes/IsSupersededBy relationships between ADRs. Implementation follows existing patterns: Rust model types, petgraph integration, YAML frontmatter parsing, and Tera templates.

## Technical Context

**Language/Version**: Rust 1.75+ (2021 edition)
**Primary Dependencies**: petgraph (graph), clap (CLI), pulldown-cmark (Markdown), serde_yaml (YAML), git2 (Git), inquire (interactive prompts), tera (templates)
**Storage**: File-based (Markdown files with YAML frontmatter in Git repository)
**Testing**: cargo test (unit + integration)
**Target Platform**: CLI tool (Linux, macOS, Windows)
**Project Type**: Single Rust workspace (sara-core library + sara CLI)
**Performance Goals**: Parse 1000+ items in <5 seconds, graph traversal <100ms
**Constraints**: No runtime dependencies, offline-capable, Git-native workflow
**Scale/Scope**: Supports repositories with 10,000+ items and relationships

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| **Simplicity First** | PASS | Extends existing ItemType pattern; no new abstractions; follows established graph model |
| **Modern Standards** | PASS | Rust 2021 edition; all dependencies actively maintained; standard serde/petgraph patterns |
| **Code Quality** | PASS | Will follow existing code style; clippy-clean; explicit error handling via thiserror |
| **Testing Standards** | PASS | Unit tests for new types; integration tests for graph building; parser tests for ADR frontmatter |
| **UX Consistency** | PASS | CLI follows existing command patterns; ADR template consistent with other item templates |

**Gate Result**: PASS - No violations. Proceed to Phase 0.

## Project Structure

### Documentation (this feature)

```text
specs/002-adr-management/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (CLI interface contracts)
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
sara-core/
├── src/
│   ├── model/
│   │   ├── item.rs          # Add ItemType::ArchitectureDecisionRecord, AdrAttributes
│   │   ├── relationship.rs  # Add Justifies/Supersedes relationship types
│   │   └── field.rs         # Add ADR-specific field names
│   ├── graph/
│   │   ├── knowledge_graph.rs  # No changes (generic graph)
│   │   └── builder.rs          # Extend for ADR relationship building
│   ├── parser/
│   │   └── markdown.rs      # Extend RawFrontmatter for ADR fields
│   └── template/
│       └── generator.rs     # Register ADR template
├── templates/
│   └── adr.tera             # NEW: ADR document template
└── tests/
    ├── unit/
    │   └── adr_tests.rs     # NEW: ADR model unit tests
    └── integration/
        └── adr_graph_tests.rs  # NEW: ADR graph integration tests

sara-cli/
└── src/
    └── commands/
        └── new.rs           # Extend to support ADR item creation
```

**Structure Decision**: Single Rust workspace with sara-core library (models, graph, parsing) and sara-cli binary. ADR implementation follows existing ItemType extension pattern - no new crates or architectural changes needed.

## Complexity Tracking

> No violations - table not required.

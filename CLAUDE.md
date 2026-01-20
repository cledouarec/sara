# sara Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-01-11

## Active Technologies
- Rust 1.75+ (2021 edition) + petgraph (graph), clap (CLI), pulldown-cmark (Markdown), serde_yaml (YAML), git2 (Git), inquire (interactive prompts) (001-requirements-knowledge-graph)
- N/A (in-memory graph, reads from filesystem/Git) (001-requirements-knowledge-graph)
- File-based (Markdown files with YAML frontmatter) (001-requirements-knowledge-graph)
- Rust 1.92 (2024 edition) - matching existing workspace configuration + rmcp (MCP SDK), sara-core (existing), tokio (async runtime), serde (serialization) (002-mcp-connector)
- N/A - leverages sara-core which reads from filesystem (002-mcp-connector)

- Rust 1.75+ (2021 edition) (001-requirements-knowledge-graph)

## Project Structure

```text
src/
tests/
```

## Commands

cargo test [ONLY COMMANDS FOR ACTIVE TECHNOLOGIES][ONLY COMMANDS FOR ACTIVE TECHNOLOGIES] cargo clippy

## Code Style

Rust 1.75+ (2021 edition): Follow standard conventions

## Recent Changes
- 002-mcp-connector: Added Rust 1.92 (2024 edition) - matching existing workspace configuration + rmcp (MCP SDK), sara-core (existing), tokio (async runtime), serde (serialization)
- 001-requirements-knowledge-graph: Added Rust 1.75+ (2021 edition)
- 001-requirements-knowledge-graph: Added Rust 1.75+ (2021 edition) + petgraph (graph), clap (CLI), pulldown-cmark (Markdown), serde_yaml (YAML), git2 (Git), inquire (interactive prompts)

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->

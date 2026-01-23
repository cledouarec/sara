# Contributing to SARA

Contributions are welcome! Please feel free to submit a Pull Request.

## Table of Contents

<details>
<summary>Expand contents</summary>

- [Development](#development)
  - [Prerequisites](#prerequisites)
  - [Building](#building)
  - [Project Structure](#project-structure)
- [How to Contribute](#how-to-contribute)
  - [Commit Convention](#commit-convention)
- [Code of Conduct](#code-of-conduct)

</details>

## Development

### Prerequisites

- Rust 1.85+ (2024 edition)

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run clippy
cargo clippy
```

### Project Structure

```
sara-core/       # Library crate (business logic)
├── src/
│   ├── model/       # Domain entities
│   ├── graph/       # Knowledge graph operations
│   ├── parser/      # Markdown and YAML parsing
│   ├── repository/  # Multi-repo file discovery
│   ├── validation/  # Integrity checks
│   ├── query/       # Query operations
│   ├── report/      # Report generation
│   ├── config/      # Configuration handling
│   └── template/    # Document templates

sara-cli/        # Binary crate (CLI interface)
├── src/
│   ├── commands/    # CLI subcommands
│   └── output/      # Output formatting

tests/           # Integration tests
└── fixtures/    # Test documents
```

## How to Contribute

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes using [Conventional Commits](https://www.conventionalcommits.org/) format
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Commit Convention

We use [Conventional Commits](https://www.conventionalcommits.org/) format:

| Prefix | Description |
|--------|-------------|
| `feat:` | New features |
| `fix:` | Bug fixes |
| `docs:` | Documentation changes |
| `refactor:` | Code refactoring |
| `test:` | Test additions |
| `build:` | Build system and dependencies |
| `ci:` | CI/CD configuration changes |

## Code of Conduct

Please read our [Code of Conduct](CODE_OF_CONDUCT.md) before participating in this project.

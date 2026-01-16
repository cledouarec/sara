# sara-cli

Command-line interface for [SARA](https://github.com/cledouarec/sara) - **S**olution **A**rchitecture **R**equirement for **A**lignment.

SARA is a CLI tool that manages architecture documents and requirements as an interconnected knowledge graph, providing a single source of truth for all teams and contributors.

## Installation

```bash
cargo install sara-cli
```

## Quick Start

1. Create a Markdown file with YAML frontmatter:

```markdown
---
id: "SOL-001"
type: solution
name: "Customer Portal"
description: "Web-based customer self-service portal"
---

# Customer Portal Solution

This solution provides a self-service portal for customers.
```

2. Create a `sara.toml` configuration file:

```toml
[repositories]
paths = ["./docs"]

[validation]
strict_orphans = false

[output]
colors = true
emojis = true
```

3. Parse and validate:

```bash
sara parse
sara validate
sara query SOL-001 --downstream
```

## Commands

| Command | Description |
|---------|-------------|
| `sara diff <REF1> <REF2>` | Compare graphs between Git references |
| `sara edit <ID>` | Edit existing document metadata by item ID |
| `sara init <FILE>` | Initialize metadata in a Markdown file |
| `sara parse` | Parse documents and build the knowledge graph |
| `sara query <ID>` | Query items and traceability chains |
| `sara report coverage` | Generate coverage report |
| `sara report matrix` | Generate traceability matrix |
| `sara validate` | Validate graph integrity |

## Output Formats

Most commands support multiple output formats:

```bash
# Text output (default)
sara report coverage

# JSON output
sara report coverage --format json

# CSV output
sara report matrix --format csv -o matrix.csv
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `SARA_CONFIG` | Path to configuration file |
| `NO_COLOR` | Disable colored output when set |

## Documentation

For full documentation, see the [SARA repository](https://github.com/cledouarec/sara).

## License

Licensed under the Apache-2.0 License. See [LICENSE](https://github.com/cledouarec/sara/blob/main/LICENSE) for details.

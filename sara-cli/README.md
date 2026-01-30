# sara-cli

Command-line interface for [SARA](https://github.com/cledouarec/sara) - **S**olution **A**rchitecture **R**equirement for **A**lignment.

SARA is a CLI tool that manages architecture documents and requirements as an interconnected knowledge graph, providing a single source of truth for all teams and contributors.

## Installation

```bash
cargo install sara-cli
```

## Quick Start

![quick start demo init](../assets/generated/demo-init.gif)

## Commands

| Command | Description |
|---------|-------------|
| `sara check` | Parse documents and validate graph integrity |
| `sara diff <REF1> <REF2>` | Compare graphs between Git references |
| `sara edit <ID>` | Edit existing document metadata by item ID |
| `sara init <FILE>` | Initialize metadata in a Markdown file |
| `sara query <ID>` | Query items and traceability chains |
| `sara report coverage` | Generate coverage report |
| `sara report matrix` | Generate traceability matrix |

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

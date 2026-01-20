# Quickstart: SARA MCP Server

This guide shows how to set up and use the SARA MCP server with AI assistants.

## Prerequisites

- Rust 1.92+ installed
- SARA project with a valid `sara.toml` configuration
- An MCP-compatible AI assistant (Claude Desktop, Cursor, VS Code, etc.)

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/cledouarec/sara.git
cd sara

# Build and install
cargo install --path sara-mcp

# Verify installation
sara-mcp --version
```

## Configuration

Ensure you have a `sara.toml` file in your project:

```toml
[repositories]
paths = ["./docs"]

[validation]
strict_orphans = false

[output]
colors = true
emojis = true
```

## Usage

### Stdio Mode (Local IDE Integration)

For Claude Desktop, Cursor, or VS Code, configure the MCP server in stdio mode:

**Claude Desktop** (`~/Library/Application Support/Claude/claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "sara": {
      "command": "sara-mcp",
      "args": ["--mode", "stdio"],
      "cwd": "/path/to/your/project"
    }
  }
}
```

**Cursor** (`.cursor/mcp.json` in project root):

```json
{
  "servers": {
    "sara": {
      "command": "sara-mcp",
      "args": ["--mode", "stdio"]
    }
  }
}
```

### HTTP Mode (Network Access)

For remote access or team sharing:

```bash
# Start server on default port (3000)
sara-mcp --mode http

# Start with custom port
sara-mcp --mode http --port 8080

# Start with OAuth enabled (production)
sara-mcp --mode http --port 443 --oauth --oauth-issuer https://auth.example.com
```

## Available Tools

Once connected, your AI assistant can use these tools:

| Tool | Description | Example Prompt |
|------|-------------|----------------|
| `sara_query` | Query item by ID | "What is SOL-001?" |
| `sara_validate` | Validate knowledge graph | "Validate my requirements" |
| `sara_coverage_report` | Generate coverage report | "Show coverage metrics" |
| `sara_matrix_report` | Generate traceability matrix | "Show traceability from scenarios to system requirements" |
| `sara_init` | Create new document | "Create a new system requirement" |
| `sara_edit` | Edit document metadata | "Add SCEN-001 as parent of SYSREQ-005" |
| `sara_diff` | Compare Git versions | "What changed since last week?" |
| `sara_parse` | Refresh knowledge graph | "Reload the requirements" |
| `sara_list_items` | List all items | "List all solutions" |
| `sara_stats` | Get graph statistics | "How many requirements do we have?" |

## Example Conversations

### Querying Requirements

**You**: "What is SOL-001 and what derives from it?"

**AI**: *Uses sara_query with item_id="SOL-001" and direction="downstream"*

### Validating the Graph

**You**: "Are there any broken references in my requirements?"

**AI**: *Uses sara_validate with strict=false*

### Creating a New Requirement

**You**: "Create a new software requirement for authentication that derives from SCEN-001"

**AI**: *Uses sara_init with item_type="software_requirement", derives_from=["SCEN-001"]*

### Understanding Coverage

**You**: "Which requirements don't have complete traceability?"

**AI**: *Uses sara_coverage_report with include_uncovered=true*

## Troubleshooting

### "Configuration not found"

The server couldn't find `sara.toml`. Make sure:
1. You're running from the project directory
2. The file exists and is valid TOML
3. Check parent directories - SARA searches upward

### "Item not found"

The requested item ID doesn't exist. Use `sara_list_items` to see available items.

### Connection Refused (HTTP mode)

1. Check the server is running: `ps aux | grep sara-mcp`
2. Verify the port is correct
3. For OAuth, ensure token is valid

## Next Steps

- Read the [SARA documentation](../README.md) to understand the requirements hierarchy
- Explore the [tool contracts](contracts/mcp-tools.json) for detailed input/output schemas
- Configure OAuth for production HTTP deployments

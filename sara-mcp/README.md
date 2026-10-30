# sara-mcp

Model Context Protocol server for [SARA](https://github.com/cledouarec/sara) - **S**olution **A**rchitecture **R**equirement for **A**lignment.

This crate lets AI coding agents query a SARA knowledge graph through the [Model Context Protocol](https://modelcontextprotocol.io) instead of reading and cross-referencing Markdown files. It is built on [`rmcp`](https://crates.io/crates/rmcp), the official Rust SDK.

## Usage

Most users run the server through the CLI, which installs it by default:

```bash
cargo install sara-cli
sara mcp
```

See [AI Agents (MCP)](https://github.com/cledouarec/sara#ai-agents-mcp) for the configuration of Claude Code, Codex CLI, Gemini CLI, Cursor and other clients.

To embed the server, call `serve_stdio`, which runs until the client disconnects:

```rust,no_run
use std::path::PathBuf;

use sara_mcp::{ServerOptions, serve_stdio};

fn main() -> Result<(), sara_mcp::ServeError> {
    serve_stdio(ServerOptions {
        repositories: vec![PathBuf::from("docs")],
        strict: false,
    })
}
```

Standard output carries protocol messages only: write logs to standard error. A custom model schema must be installed with `sara_core::schema::install` before the server starts.

## Tools

| Tool | Description |
|------|-------------|
| `sara_search_items` | Find items by the words of a query, best matches first |
| `sara_get_item` | Read one item, its direct links and its Markdown body |
| `sara_trace_item` | Follow traceability links upstream, downstream or both |
| `sara_analyze_impact` | List the items a change may impact |
| `sara_validate` | Check the whole graph |
| `sara_describe_model` | Describe the item types, fields and relations of the model |
| `sara_create_item` | Create an item |
| `sara_update_item` | Change an item's frontmatter, rewriting only the given entries |

## License

Licensed under the Apache-2.0 License. See [LICENSE](https://github.com/cledouarec/sara/blob/main/LICENSE) for details.

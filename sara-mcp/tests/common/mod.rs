//! Harness running a [`SaraServer`] in process with a client connected to it.

// Each test binary uses only some of these helpers.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use rmcp::model::{CallToolRequestParams, CallToolResult, Tool};
use rmcp::service::RunningService;
use rmcp::{RoleClient, ServiceExt};
use sara_mcp::{SaraServer, ServerOptions};
use serde_json::Value;

/// Capacity of the in-memory pipe between the client and the server.
const PIPE_CAPACITY: usize = 64 * 1024;

/// A client connected to a server under test.
pub type Client = RunningService<RoleClient, ()>;

/// Starts a server with `options` and returns a client connected to it.
pub async fn connect(options: ServerOptions) -> Client {
    let (server_io, client_io) = tokio::io::duplex(PIPE_CAPACITY);
    tokio::spawn(async move {
        let server = SaraServer::new(options)
            .serve(server_io)
            .await
            .expect("server starts");
        let _ = server.waiting().await;
    });
    ().serve(client_io).await.expect("client connects")
}

/// Returns the path of a shared test fixture repository.
pub fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("tests")
        .join("fixtures")
        .join(name)
}

/// Starts a server on a shared fixture repository and connects to it.
pub async fn connect_to_fixture(name: &str) -> Client {
    connect_to_dir(&fixture(name)).await
}

/// Calls the tool `name` with `arguments`, which must be a JSON object.
pub async fn call(client: &Client, name: &str, arguments: Value) -> CallToolResult {
    let Value::Object(arguments) = arguments else {
        panic!("tool arguments must be a JSON object");
    };
    client
        .call_tool(CallToolRequestParams::new(name.to_string()).with_arguments(arguments))
        .await
        .expect("tool call completes")
}

/// Returns the concatenated text content of a tool result.
pub fn text(result: &CallToolResult) -> String {
    result
        .content
        .iter()
        .filter_map(|content| content.as_text())
        .map(|text| text.text.as_str())
        .collect()
}

/// Returns the description of the tool `name`, panicking when absent.
pub async fn tool(client: &Client, name: &str) -> Tool {
    client
        .list_all_tools()
        .await
        .expect("tools are listed")
        .into_iter()
        .find(|tool| tool.name == name)
        .unwrap_or_else(|| panic!("tool {name} is listed"))
}

/// Starts a server on the repository at `path` and connects to it.
pub async fn connect_to_dir(path: &Path) -> Client {
    connect(ServerOptions {
        repositories: vec![path.to_path_buf()],
        strict: false,
    })
    .await
}

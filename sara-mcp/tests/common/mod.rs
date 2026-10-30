//! Harness running a [`SaraServer`] in process with a client connected to it.

// Each test binary uses only some of these helpers.
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use rmcp::model::{CallToolRequestParams, CallToolResult, Tool};
use rmcp::service::RunningService;
use rmcp::{RoleClient, ServiceExt};
use sara_mcp::{SaraServer, ServerOptions};
use serde_json::Value;
use tempfile::TempDir;

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

/// Calls the tool `name` and returns its structured result, panicking when
/// there is none.
pub async fn structured(client: &Client, name: &str, arguments: Value) -> Value {
    let result = call(client, name, arguments).await;
    match result.structured_content {
        Some(structured) => structured,
        None => panic!("{name} returned no structured content: {}", text(&result)),
    }
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

/// Copies a shared fixture repository into a new temporary directory.
pub fn copy_fixture(name: &str) -> TempDir {
    let dir = TempDir::new().expect("temporary directory");
    for entry in fs::read_dir(fixture(name)).expect("fixture directory") {
        let entry = entry.expect("fixture entry");
        fs::copy(entry.path(), dir.path().join(entry.file_name())).expect("fixture copy");
    }
    dir
}

/// Starts a server on the repository at `path` and connects to it.
pub async fn connect_to_dir(path: &Path) -> Client {
    connect(ServerOptions {
        repositories: vec![path.to_path_buf()],
        strict: false,
    })
    .await
}

/// Starts a server on a copy of a shared fixture repository, which tools
/// may change, and connects to it.
pub async fn connect_to_copy(name: &str) -> (TempDir, Client) {
    let repository = copy_fixture(name);
    let client = connect_to_dir(repository.path()).await;
    (repository, client)
}

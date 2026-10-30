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

/// Keywords [`conformance`] does not check: a schema using one of them could
/// accept results that break it.
const UNCHECKED_KEYWORDS: [&str; 10] = [
    "allOf",
    "not",
    "if",
    "then",
    "else",
    "prefixItems",
    "unevaluatedProperties",
    "patternProperties",
    "propertyNames",
    "dependentSchemas",
];

/// Keywords whose value maps names to schemas.
const SCHEMA_MAPS: [&str; 2] = ["properties", "$defs"];

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

/// Calls the tool `name` and returns its structured result, after checking
/// that the result conforms to the output schema the tool declares.
pub async fn call_conforming(client: &Client, name: &str, arguments: Value) -> Value {
    let schema = tool(client, name)
        .await
        .output_schema
        .map(|schema| Value::Object((*schema).clone()))
        .unwrap_or_else(|| panic!("{name} declares an output schema"));
    if let Err(error) = checked_keywords_only(&schema, "$") {
        panic!("{name} output schema cannot be checked: {error}");
    }
    let structured = structured(client, name, arguments).await;
    if let Err(error) = conformance(&schema, &schema, &structured, "$") {
        panic!("{name} result does not match its output schema: {error}\n{structured}");
    }
    structured
}

/// Checks that `schema` and its subschemas use none of the
/// [`UNCHECKED_KEYWORDS`].
fn checked_keywords_only(schema: &Value, path: &str) -> Result<(), String> {
    let Value::Object(object) = schema else {
        return Ok(());
    };
    for (key, value) in object {
        if UNCHECKED_KEYWORDS.contains(&key.as_str()) {
            return Err(format!("{path}: unchecked keyword `{key}`"));
        }
        let key_path = format!("{path}/{key}");
        match value {
            Value::Object(map) if SCHEMA_MAPS.contains(&key.as_str()) => {
                for (name, subschema) in map {
                    checked_keywords_only(subschema, &format!("{key_path}/{name}"))?;
                }
            }
            Value::Object(_) => checked_keywords_only(value, &key_path)?,
            Value::Array(branches) => {
                for (index, branch) in branches.iter().enumerate() {
                    checked_keywords_only(branch, &format!("{key_path}[{index}]"))?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// Checks `value` against `schema`: types, enumerations, constants, required
/// and declared properties, map values, array items, `anyOf`/`oneOf` branches
/// and `$ref`s into the `$defs` of `root`.
fn conformance(root: &Value, schema: &Value, value: &Value, path: &str) -> Result<(), String> {
    if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
        let name = reference
            .strip_prefix("#/$defs/")
            .ok_or_else(|| format!("{path}: unsupported $ref {reference}"))?;
        return conformance(root, &root["$defs"][name], value, path);
    }
    for keyword in ["anyOf", "oneOf"] {
        if let Some(branches) = schema.get(keyword).and_then(Value::as_array)
            && !branches
                .iter()
                .any(|branch| conformance(root, branch, value, path).is_ok())
        {
            return Err(format!("{path}: {value} matches no branch of {keyword}"));
        }
    }
    if let Some(allowed) = schema.get("enum").and_then(Value::as_array)
        && !allowed.contains(value)
    {
        return Err(format!("{path}: {value} is not one of {allowed:?}"));
    }
    if let Some(expected) = schema.get("const")
        && value != expected
    {
        return Err(format!("{path}: {value} is not {expected}"));
    }
    if let Some(types) = schema.get("type") {
        let types: Vec<&str> = match types {
            Value::String(single) => vec![single.as_str()],
            Value::Array(several) => several.iter().filter_map(Value::as_str).collect(),
            _ => Vec::new(),
        };
        if !types.iter().any(|kind| has_type(kind, value)) {
            return Err(format!("{path}: {value} is not of type {types:?}"));
        }
    }
    if let Value::Object(object) = value {
        for key in schema
            .get("required")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            if !object.contains_key(key) {
                return Err(format!("{path}: missing required property `{key}`"));
            }
        }
        let properties = schema.get("properties").and_then(Value::as_object);
        for (key, field) in object {
            let field_path = format!("{path}.{key}");
            match properties.and_then(|properties| properties.get(key)) {
                Some(property) => conformance(root, property, field, &field_path)?,
                None => {
                    if let Some(additional) =
                        schema.get("additionalProperties").filter(|a| a.is_object())
                    {
                        conformance(root, additional, field, &field_path)?;
                    }
                }
            }
        }
    }
    if let (Value::Array(items), Some(item_schema)) = (value, schema.get("items")) {
        for (index, item) in items.iter().enumerate() {
            conformance(root, item_schema, item, &format!("{path}[{index}]"))?;
        }
    }
    Ok(())
}

/// Whether `value` has the JSON Schema type `kind`.
fn has_type(kind: &str, value: &Value) -> bool {
    match kind {
        "object" => value.is_object(),
        "array" => value.is_array(),
        "string" => value.is_string(),
        "boolean" => value.is_boolean(),
        "integer" => value.is_i64() || value.is_u64(),
        "number" => value.is_number(),
        "null" => value.is_null(),
        _ => true,
    }
}

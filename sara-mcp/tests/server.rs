//! Tests of the server identity and instructions.

mod common;

use std::path::PathBuf;

use sara_mcp::ServerOptions;

#[tokio::test]
async fn test_server_introduces_itself() {
    let client = common::connect(ServerOptions {
        repositories: vec![PathBuf::from("docs/requirements")],
        strict: false,
    })
    .await;

    let info = client.peer_info().expect("server info");
    let server = info.server_info.as_ref().expect("server identity");
    assert_eq!(server.name, "sara");
    assert_eq!(server.version, env!("CARGO_PKG_VERSION"));

    let instructions = info.instructions.as_deref().unwrap_or_default();
    assert!(instructions.contains("docs/requirements"), "{instructions}");
    assert!(instructions.contains("solution (SOL)"), "{instructions}");

    client.cancel().await.unwrap();
}

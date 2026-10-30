//! Model Context Protocol server for Sara.
//!
//! Exposes a Sara knowledge graph to AI agents (Claude Code, Codex CLI,
//! Gemini CLI, Cursor and others) through the [Model Context Protocol]:
//! instead of reading and cross-referencing Markdown files, an agent asks the
//! server for the items, links and checks it needs.
//!
//! [`serve_stdio`] runs the server on standard input and output, the
//! transport local agents use. [`SaraServer`] is the protocol handler itself,
//! for tests or for serving another transport.
//!
//! Standard output carries protocol messages only: a program embedding the
//! server must write its logs elsewhere, such as standard error.
//!
//! The server reads the graph again on every request, so its answers always
//! reflect the files on disk. The model schema, however, is installed once
//! per process with [`sara_core::schema::install`]: restart the server after
//! changing it.
//!
//! [Model Context Protocol]: https://modelcontextprotocol.io

mod inputs;
mod params;
mod paths;
mod server;
mod views;

use std::io;
use std::path::PathBuf;

use rmcp::ServiceExt;
use rmcp::transport::stdio;
use tokio::runtime::Builder;

#[doc(inline)]
pub use server::SaraServer;

/// Settings of a Sara MCP server.
#[derive(Debug, Clone, Default)]
pub struct ServerOptions {
    /// Repositories scanned for items on every request.
    pub repositories: Vec<PathBuf>,
    /// Whether validation reports warnings as errors.
    pub strict: bool,
}

/// Error ending a stdio MCP session.
#[derive(Debug, thiserror::Error)]
pub enum ServeError {
    /// The async runtime could not be started.
    #[error("failed to start the async runtime: {0}")]
    Runtime(#[source] io::Error),

    /// The client did not complete the protocol handshake.
    #[error("failed to initialize the MCP session: {0}")]
    Initialize(String),

    /// The session stopped on an internal failure.
    #[error("the MCP session failed: {0}")]
    Session(String),
}

/// Runs a Sara MCP server on standard input and output until the client
/// disconnects.
///
/// Blocks the calling thread on a multi-threaded async runtime of its own,
/// so callers need no runtime.
///
/// # Examples
///
/// ```no_run
/// use std::path::PathBuf;
///
/// use sara_mcp::{ServerOptions, serve_stdio};
///
/// fn main() -> Result<(), sara_mcp::ServeError> {
///     serve_stdio(ServerOptions {
///         repositories: vec![PathBuf::from("docs")],
///         strict: false,
///     })
/// }
/// ```
///
/// # Errors
///
/// Returns [`ServeError::Runtime`] when the runtime cannot start,
/// [`ServeError::Initialize`] when the client does not complete the protocol
/// handshake, and [`ServeError::Session`] when the session stops on an
/// internal failure.
pub fn serve_stdio(options: ServerOptions) -> Result<(), ServeError> {
    let runtime = Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(ServeError::Runtime)?;

    runtime.block_on(async {
        let service = SaraServer::new(options)
            .serve(stdio())
            .await
            .map_err(|e| ServeError::Initialize(e.to_string()))?;
        service
            .waiting()
            .await
            .map_err(|e| ServeError::Session(e.to_string()))?;
        Ok(())
    })
}

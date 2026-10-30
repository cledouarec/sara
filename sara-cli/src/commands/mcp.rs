//! Implementation of the mcp command.

use std::error::Error;
use std::process::ExitCode;

use sara_core::config::Config;
use sara_mcp::{ServerOptions, serve_stdio};

/// Serves the knowledge graph over the Model Context Protocol on standard
/// input and output, until the client disconnects.
///
/// Standard output carries protocol messages only, so failures are logged
/// to standard error and reported through the exit code.
pub fn run(config: &Config) -> Result<ExitCode, Box<dyn Error>> {
    let repositories = match super::resolve_repositories(config) {
        Ok(repositories) => repositories,
        Err(error) => {
            tracing::error!("Cannot resolve the repositories: {error}");
            return Ok(ExitCode::FAILURE);
        }
    };

    let options = ServerOptions {
        repositories,
        strict: config.validation.strict_mode,
    };
    match serve_stdio(options) {
        Ok(()) => Ok(ExitCode::SUCCESS),
        Err(error) => {
            tracing::error!("MCP server stopped: {error}");
            Ok(ExitCode::FAILURE)
        }
    }
}

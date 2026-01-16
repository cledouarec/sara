//! Logging configuration for the CLI.

use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;

/// Verbosity levels for CLI output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verbosity {
    /// Errors only
    Quiet,
    /// Warnings and errors (default)
    Normal,
    /// Info, warnings, and errors
    Verbose,
    /// Debug output
    Debug,
    /// Full trace output
    Trace,
}

impl Verbosity {
    /// Returns the log level filter string for this verbosity.
    pub fn as_filter(&self) -> &'static str {
        match self {
            Verbosity::Quiet => "error",
            Verbosity::Normal => "warn",
            Verbosity::Verbose => "info",
            Verbosity::Debug => "debug",
            Verbosity::Trace => "trace",
        }
    }
}

/// Initializes the tracing subscriber with the given verbosity level.
pub fn init(verbosity: Verbosity) {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(verbosity.as_filter()));

    let subscriber = tracing_subscriber::registry().with(
        fmt::layer()
            .with_target(false)
            .with_level(true)
            .with_filter(filter),
    );

    if tracing::subscriber::set_global_default(subscriber).is_err() {
        // Subscriber already set, ignore
    }
}

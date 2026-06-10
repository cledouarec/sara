//! Logging configuration for the CLI.

use std::sync::OnceLock;

use tracing_subscriber::registry::Registry;
use tracing_subscriber::{EnvFilter, fmt, prelude::*, reload};

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

/// Handle used to adjust the log level once the command line is parsed.
static RELOAD_HANDLE: OnceLock<reload::Handle<EnvFilter, Registry>> = OnceLock::new();

/// Initializes the tracing subscriber at the default verbosity.
///
/// Called before the command line is parsed so configuration loading can log
/// directly; [`set_verbosity`] adjusts the level once the flags are known.
/// An environment filter (`RUST_LOG`) takes precedence when set.
pub fn init() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(Verbosity::Normal.as_filter()));
    let (filter, handle) = reload::Layer::new(filter);

    let subscriber = tracing_subscriber::registry().with(
        fmt::layer()
            .with_target(false)
            .with_level(true)
            .with_filter(filter),
    );

    if tracing::subscriber::set_global_default(subscriber).is_ok() {
        let _ = RELOAD_HANDLE.set(handle);
    }
}

/// Adjusts the log level to the verbosity parsed from the command line.
///
/// Keeps the environment filter (`RUST_LOG`) when one is set, mirroring
/// [`init`]. Has no effect if the subscriber could not be installed.
pub fn set_verbosity(verbosity: Verbosity) {
    if EnvFilter::try_from_default_env().is_ok() {
        return;
    }
    if let Some(handle) = RELOAD_HANDLE.get() {
        let _ = handle.reload(EnvFilter::new(verbosity.as_filter()));
    }
}

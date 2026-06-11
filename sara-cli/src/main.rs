//! Sara CLI - Requirements Knowledge Graph CLI

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

mod commands;
mod logging;
mod output;

use commands::Commands;
use sara_core::config::{OutputConfig, load_config};

/// Help heading for global options
const GLOBAL_OPTIONS: &str = "Global Options";

/// Default configuration file, relative to the working directory.
const DEFAULT_CONFIG_FILE: &str = "sara.toml";

/// Environment variable overriding the configuration path.
const CONFIG_ENV_VAR: &str = "SARA_CONFIG";

/// Long form of the configuration flag, mirrored by the pre-parse scan.
const CONFIG_FLAG_LONG: &str = "--config";

/// Short form of the configuration flag, mirrored by the pre-parse scan.
const CONFIG_FLAG_SHORT: &str = "-c";

/// Sara - Requirements Knowledge Graph CLI
///
/// Manages Architecture documents and Requirements as a unified interconnected knowledge graph.
#[derive(Parser, Debug)]
#[command(name = "sara", version, about, long_about = None, disable_help_flag = true, disable_version_flag = true)]
pub struct Cli {
    /// Path to configuration file
    #[arg(short, long, global = true, default_value = DEFAULT_CONFIG_FILE, help_heading = GLOBAL_OPTIONS)]
    pub config: PathBuf,

    /// Print help
    #[arg(short, long, action = clap::ArgAction::Help, global = true, help_heading = GLOBAL_OPTIONS)]
    help: Option<bool>,

    /// Disable colored output
    #[arg(long, global = true, help_heading = GLOBAL_OPTIONS)]
    pub no_color: bool,

    /// Disable emoji output
    #[arg(long, global = true, help_heading = GLOBAL_OPTIONS)]
    pub no_emoji: bool,

    /// Suppress all output except errors
    #[arg(short, long, global = true, help_heading = GLOBAL_OPTIONS)]
    pub quiet: bool,

    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(short, long, global = true, action = clap::ArgAction::Count, help_heading = GLOBAL_OPTIONS)]
    pub verbose: u8,

    /// Additional repository paths
    #[arg(short, long, global = true, help_heading = GLOBAL_OPTIONS)]
    pub repository: Vec<PathBuf>,

    /// Print version
    #[arg(short = 'V', long, action = clap::ArgAction::Version, help_heading = GLOBAL_OPTIONS)]
    version: Option<bool>,

    #[command(subcommand)]
    pub command: Commands,
}

impl Cli {
    /// Returns the effective verbosity level.
    pub fn verbosity(&self) -> logging::Verbosity {
        if self.quiet {
            logging::Verbosity::Quiet
        } else {
            match self.verbose {
                0 => logging::Verbosity::Normal,
                1 => logging::Verbosity::Verbose,
                2 => logging::Verbosity::Debug,
                _ => logging::Verbosity::Trace,
            }
        }
    }

    /// Returns the output configuration, merging config file settings with CLI flags.
    /// CLI flags override config file settings.
    pub fn output_config(&self, file_config: Option<&sara_core::config::Config>) -> OutputConfig {
        // Check environment variables
        let env_no_color = std::env::var("NO_COLOR").is_ok();

        // Start with config file values, or defaults if no config
        let (config_colors, config_emojis) = file_config
            .map(|c| (c.output.colors, c.output.emojis))
            .unwrap_or((true, true));

        // CLI flags override config file (--no-color and --no-emoji disable the feature)
        // Environment variable NO_COLOR also disables colors
        let colors = config_colors && !self.no_color && !env_no_color;
        let emojis = config_emojis && !self.no_emoji;

        OutputConfig { colors, emojis }
    }

    /// Returns the config file path, checking SARA_CONFIG env var first.
    pub fn config_path(&self) -> PathBuf {
        if let Ok(env_config) = std::env::var(CONFIG_ENV_VAR) {
            PathBuf::from(env_config)
        } else {
            self.config.clone()
        }
    }

    /// Returns the repository paths, defaulting to current directory if none specified.
    pub fn repositories(&self) -> Result<Vec<PathBuf>, std::io::Error> {
        if self.repository.is_empty() {
            Ok(vec![std::env::current_dir()?])
        } else {
            Ok(self.repository.clone())
        }
    }
}

/// Returns the configuration path without parsing the full command line.
///
/// The `init` subcommands are built from the active schema, so the schema
/// must be installed before clap sees the arguments. This lightweight scan
/// mirrors [`Cli::config_path`]: `SARA_CONFIG`, then `-c`/`--config`, then
/// the default `sara.toml`.
fn early_config_path() -> PathBuf {
    if let Ok(env_config) = std::env::var(CONFIG_ENV_VAR) {
        return PathBuf::from(env_config);
    }

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == CONFIG_FLAG_SHORT || arg == CONFIG_FLAG_LONG {
            if let Some(value) = args.next() {
                return PathBuf::from(value);
            }
        } else if let Some(value) = arg
            .strip_prefix(CONFIG_FLAG_LONG)
            .and_then(|v| v.strip_prefix('='))
        {
            return PathBuf::from(value);
        } else if !arg.starts_with("--")
            && let Some(value) = arg.strip_prefix(CONFIG_FLAG_SHORT)
            && !value.is_empty()
        {
            return PathBuf::from(value.trim_start_matches('='));
        }
    }

    PathBuf::from(DEFAULT_CONFIG_FILE)
}

fn main() -> ExitCode {
    // Logging starts at the default verbosity so the preload steps below can
    // report problems; the level is adjusted once the flags are parsed.
    logging::init();

    // Load the config and install the active schema before parsing the
    // command line: the init subcommands are derived from the schema.
    // Failures are non-fatal; the built-in defaults apply.
    let config_path = early_config_path();
    let file_config = if config_path.exists() {
        match load_config(&config_path) {
            Ok(config) => Some(config),
            Err(e) => {
                tracing::warn!(
                    "Failed to load config file {}: {}",
                    config_path.display(),
                    e
                );
                None
            }
        }
    } else {
        None
    };

    if let Some(cfg) = file_config.as_ref() {
        match cfg.load_schema() {
            Ok(schema) => {
                // Discard `Err`: another caller may have installed first
                // (no-op for the CLI entry point but defensive).
                let _ = sara_core::schema::install(schema);
            }
            Err(e) => {
                output::print_warning(
                    &cfg.output,
                    &format!(
                        "Failed to load model schema: {}; continuing with the built-in model",
                        e
                    ),
                );
            }
        }

        // Install document template overrides before the first generation so
        // configured `.tera` templates take effect.
        match sara_core::generator::discover_overrides(&cfg.templates) {
            Ok(overrides) => {
                let _ = sara_core::generator::install_overrides(overrides);
            }
            Err(e) => {
                tracing::warn!("Failed to load template overrides: {}", e);
            }
        }
    }

    let cli = Cli::parse();
    logging::set_verbosity(cli.verbosity());

    // Run the command
    let result = commands::run(&cli, file_config.as_ref());

    match result {
        Ok(code) => code,
        Err(e) => {
            let config = cli.output_config(file_config.as_ref());
            output::print_error(&config, &format!("{}", e));
            ExitCode::FAILURE
        }
    }
}

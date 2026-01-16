//! Sara CLI - Requirements Knowledge Graph CLI

use clap::Parser;
use std::path::PathBuf;
use std::process::ExitCode;

mod commands;
mod logging;
mod output;

use commands::Commands;
use sara_core::config::load_config;

/// Help heading for global options
const GLOBAL_OPTIONS: &str = "Global Options";

/// Sara - Requirements Knowledge Graph CLI
///
/// Manages Architecture documents and Requirements as a unified interconnected knowledge graph.
#[derive(Parser, Debug)]
#[command(name = "sara", version, about, long_about = None, disable_help_flag = true, disable_version_flag = true)]
pub struct Cli {
    /// Path to configuration file
    #[arg(short, long, global = true, default_value = "sara.toml", help_heading = GLOBAL_OPTIONS)]
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
    pub fn output_config(&self, file_config: Option<&sara_core::config::Config>) -> output::OutputConfig {
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

        output::OutputConfig { colors, emojis }
    }

    /// Returns the config file path, checking SARA_CONFIG env var first.
    pub fn config_path(&self) -> PathBuf {
        if let Ok(env_config) = std::env::var("SARA_CONFIG") {
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

fn main() -> ExitCode {
    let cli = Cli::parse();

    // Initialize logging
    logging::init(cli.verbosity());

    // Load config file (optional - use defaults if not found or error)
    let config_path = cli.config_path();
    let file_config = if config_path.exists() {
        match load_config(&config_path) {
            Ok(config) => Some(config),
            Err(e) => {
                tracing::warn!("Failed to load config file {}: {}", config_path.display(), e);
                None
            }
        }
    } else {
        None
    };

    // Run the command
    let result = commands::run(&cli, file_config.as_ref());

    match result {
        Ok(code) => code,
        Err(e) => {
            let config = cli.output_config(file_config.as_ref());
            output::print_error(&config, &format!("{}", e));
            ExitCode::from(1)
        }
    }
}

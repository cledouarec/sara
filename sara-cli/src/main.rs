//! Sara CLI - Requirements Knowledge Graph CLI

use clap::Parser;
use std::path::PathBuf;
use std::process::ExitCode;

mod commands;
mod logging;
mod output;

use commands::Commands;

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

    /// Returns the output configuration.
    pub fn output_config(&self) -> output::OutputConfig {
        // Check environment variables
        let no_color = self.no_color || std::env::var("NO_COLOR").is_ok();

        output::OutputConfig {
            colors: !no_color,
            emojis: !self.no_emoji,
        }
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

    // Run the command
    let result = commands::run(&cli);

    match result {
        Ok(code) => code,
        Err(e) => {
            let config = cli.output_config();
            output::print_error(&config, &format!("{}", e));
            ExitCode::from(1)
        }
    }
}

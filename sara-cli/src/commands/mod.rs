//! CLI command implementations.

mod diff;
mod edit;
mod init;
mod interactive;
mod parse;
mod query;
mod report;
mod validate;

use std::env;
use std::error::Error;
use std::io;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Subcommand;
use sara_core::config::Config;

use crate::Cli;
use crate::output::OutputConfig;

use self::diff::DiffArgs;
use self::edit::EditArgs;
use self::init::InitArgs;
use self::parse::ParseArgs;
use self::query::QueryArgs;
use self::report::ReportArgs;
use self::validate::ValidateArgs;

/// Shared context for command execution.
#[derive(Debug, Clone)]
pub struct CommandContext {
    /// Output configuration (colors, emojis).
    pub output: OutputConfig,
    /// Repository paths to operate on.
    pub repositories: Vec<PathBuf>,
}

/// Available CLI commands.
#[derive(Subcommand, Debug)]
#[command(disable_help_subcommand = true)]
pub enum Commands {
    /// Compare graphs between Git references
    Diff(DiffArgs),

    /// Edit existing document metadata by item ID (interactive mode if no flags provided)
    ///
    /// When modification flags are omitted, enters interactive mode which guides you through
    /// editing with current values shown as defaults. Interactive mode requires a TTY terminal.
    ///
    /// Examples:
    ///   sara edit SREQ-001                    # Interactive mode
    ///   sara edit SREQ-001 --name "New Name"  # Non-interactive mode
    Edit(EditArgs),

    /// Initialize metadata in a Markdown file (interactive mode if --type not provided)
    ///
    /// When --type is omitted, enters interactive mode which guides you through
    /// creating a new traceability item with prompts for type, name, ID, and
    /// upstream references. Interactive mode requires a TTY terminal.
    ///
    /// Examples:
    ///   sara init                    # Interactive mode
    ///   sara init doc.md -t use_case # Non-interactive mode
    Init(InitArgs),

    /// Parse documents and build the knowledge graph
    Parse(ParseArgs),

    /// Query items and traceability chains
    Query(QueryArgs),

    /// Generate coverage and traceability reports
    Report(ReportArgs),

    /// Validate graph integrity
    Validate(ValidateArgs),
}

/// Returns repositories: CLI args take precedence, then config file, then current directory.
fn get_repositories(cli: &Cli, file_config: Option<&Config>) -> Result<Vec<PathBuf>, io::Error> {
    if !cli.repository.is_empty() {
        Ok(cli.repository.clone())
    } else if let Some(config) = file_config {
        if !config.repositories.paths.is_empty() {
            Ok(config.repositories.paths.clone())
        } else {
            Ok(vec![env::current_dir()?])
        }
    } else {
        Ok(vec![env::current_dir()?])
    }
}

/// Runs the appropriate command.
pub fn run(cli: &Cli, file_config: Option<&Config>) -> Result<ExitCode, Box<dyn Error>> {
    let output = cli.output_config(file_config);
    let repositories = get_repositories(cli, file_config)?;

    let ctx = CommandContext {
        output: output.clone(),
        repositories,
    };

    match &cli.command {
        Commands::Diff(args) => diff::run(args, ctx.repositories.clone(), &ctx),
        Commands::Edit(args) => edit::run(args, &ctx),
        Commands::Init(args) => init::run(args, &ctx),
        Commands::Parse(args) => parse::run(args, &ctx),
        Commands::Query(args) => query::run(args, &ctx),
        Commands::Report(args) => report::run(args, &ctx),
        Commands::Validate(args) => validate::run(args, &ctx),
    }
}

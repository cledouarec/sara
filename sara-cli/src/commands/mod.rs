//! CLI command implementations.

mod check;
mod diff;
mod edit;
mod init;
mod interactive;
mod query;
mod report;

use std::env;
use std::error::Error;
use std::io;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Subcommand;
use sara_core::config::Config;
use sara_core::model::Item;
use sara_core::repository::{GitReader, GitRef, parse_repositories};

use crate::Cli;
use crate::output::OutputConfig;

use self::check::CheckArgs;
use self::diff::DiffArgs;
use self::edit::EditArgs;
use self::init::InitArgs;
use self::query::QueryArgs;
use self::report::ReportArgs;

/// Shared context for command execution.
#[derive(Debug, Clone)]
pub struct CommandContext {
    /// Output configuration (colors, emojis).
    pub output: OutputConfig,
    /// Repository paths to operate on.
    pub repositories: Vec<PathBuf>,
}

impl CommandContext {
    /// Parses items from the configured repositories.
    ///
    /// If `git_ref` is provided, reads from that git reference instead of the filesystem.
    pub fn parse_items(&self, git_ref: Option<&str>) -> Result<Vec<Item>, Box<dyn Error>> {
        let repos = if self.repositories.is_empty() {
            vec![env::current_dir()?]
        } else {
            self.repositories.clone()
        };

        if let Some(ref_str) = git_ref {
            let git_ref = GitRef::parse(ref_str);
            let mut all_items = Vec::new();

            for repo_path in &repos {
                if !repo_path.exists() {
                    tracing::warn!("Repository path does not exist: {}", repo_path.display());
                    continue;
                }

                let reader = GitReader::open(repo_path)?;
                let items = reader.parse_commit(&git_ref)?;
                all_items.extend(items);
            }

            Ok(all_items)
        } else {
            Ok(parse_repositories(&repos)?)
        }
    }
}

/// Available CLI commands.
#[derive(Subcommand, Debug)]
#[command(disable_help_subcommand = true)]
pub enum Commands {
    /// Parse documents, build knowledge graph, and validate integrity
    Check(CheckArgs),

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

    /// Initialize metadata in a Markdown file
    ///
    /// When no subcommand is provided, enters interactive mode which guides you through
    /// creating a new traceability item with prompts for type, name, ID, and
    /// upstream references. Interactive mode requires a TTY terminal.
    ///
    /// Use a subcommand for non-interactive mode with type-specific options:
    ///   sara init adr, solution, use-case, scenario, system-requirement, etc.
    ///
    /// Examples:
    ///   sara init                                  # Interactive mode
    ///   sara init adr doc.md --status proposed     # Create ADR
    ///   sara init sysreq doc.md --specification "" # Create system requirement
    Init(InitArgs),

    /// Query items and traceability chains
    Query(QueryArgs),

    /// Generate coverage and traceability reports
    Report(ReportArgs),
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
        Commands::Check(args) => check::run(args, &ctx),
        Commands::Diff(args) => diff::run(args, &ctx),
        Commands::Edit(args) => edit::run(args, &ctx),
        Commands::Init(args) => init::run(args, &ctx),
        Commands::Query(args) => query::run(args, &ctx),
        Commands::Report(args) => report::run(args, &ctx),
    }
}

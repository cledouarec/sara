//! CLI command implementations.

mod diff;
mod edit;
mod init;
mod interactive;
mod parse;
mod query;
mod report;
mod validate;

use clap::Subcommand;
use std::path::PathBuf;
use std::process::ExitCode;

use crate::Cli;
use crate::output::OutputConfig;

/// Shared context for command execution.
#[derive(Debug, Clone)]
pub struct CommandContext {
    /// Output configuration (colors, emojis).
    pub output: OutputConfig,
    /// Repository paths to operate on.
    pub repositories: Vec<PathBuf>,
}

/// Arguments for the edit command.
#[derive(Debug)]
struct EditArgs<'a> {
    item_id: &'a str,
    description: &'a Option<String>,
    derives_from: &'a Option<Vec<String>>,
    name: &'a Option<String>,
    platform: &'a Option<String>,
    refines: &'a Option<Vec<String>>,
    satisfies: &'a Option<Vec<String>>,
    specification: &'a Option<String>,
}

/// Arguments for the init command.
#[derive(Debug)]
struct InitArgs<'a> {
    file: &'a Option<PathBuf>,
    derives_from: &'a [String],
    description: &'a Option<String>,
    force: bool,
    id: &'a Option<String>,
    name: &'a Option<String>,
    platform: &'a Option<String>,
    refines: &'a [String],
    satisfies: &'a [String],
    specification: &'a Option<String>,
    item_type: Option<&'a str>,
}

/// Available CLI commands.
#[derive(Subcommand, Debug)]
#[command(disable_help_subcommand = true)]
pub enum Commands {
    /// Compare graphs between Git references
    Diff {
        /// First Git reference
        ref1: String,

        /// Second Git reference
        ref2: String,

        /// Output format
        #[arg(long, default_value = "text", help_heading = "Output")]
        format: OutputFormat,

        /// Show summary statistics only
        #[arg(long, help_heading = "Output")]
        stat: bool,
    },

    /// Edit existing document metadata by item ID (interactive mode if no flags provided)
    ///
    /// When modification flags are omitted, enters interactive mode which guides you through
    /// editing with current values shown as defaults. Interactive mode requires a TTY terminal.
    ///
    /// Examples:
    ///   sara edit SREQ-001                    # Interactive mode
    ///   sara edit SREQ-001 --name "New Name"  # Non-interactive mode
    #[command(verbatim_doc_comment)]
    Edit {
        /// The item identifier to edit
        item_id: String,

        /// New item description
        #[arg(short = 'd', long, help_heading = "Item Properties")]
        description: Option<String>,

        /// New item name
        #[arg(long, help_heading = "Item Properties")]
        name: Option<String>,

        /// New upstream references (for requirements) - replaces existing
        #[arg(long, num_args = 1.., help_heading = "Traceability")]
        derives_from: Option<Vec<String>>,

        /// New upstream references (for use_case, scenario) - replaces existing
        #[arg(long, num_args = 1.., help_heading = "Traceability")]
        refines: Option<Vec<String>>,

        /// New upstream references (for architectures, designs) - replaces existing
        #[arg(long, num_args = 1.., help_heading = "Traceability")]
        satisfies: Option<Vec<String>>,

        /// New target platform (for system_architecture)
        #[arg(long, help_heading = "Type-Specific")]
        platform: Option<String>,

        /// New specification statement (for requirements)
        #[arg(long, help_heading = "Type-Specific")]
        specification: Option<String>,
    },

    /// Initialize metadata in a Markdown file (interactive mode if --type not provided)
    ///
    /// When --type is omitted, enters interactive mode which guides you through
    /// creating a new traceability item with prompts for type, name, ID, and
    /// upstream references. Interactive mode requires a TTY terminal.
    ///
    /// Examples:
    ///   sara init                    # Interactive mode
    ///   sara init doc.md -t use_case # Non-interactive mode
    #[command(verbatim_doc_comment)]
    Init {
        /// Markdown file to initialize (prompted if not provided in interactive mode)
        file: Option<std::path::PathBuf>,

        /// Item description
        #[arg(short = 'd', long, help_heading = "Item Properties")]
        description: Option<String>,

        /// Item identifier (auto-generated if not provided)
        #[arg(long, help_heading = "Item Properties")]
        id: Option<String>,

        /// Item name (extracted from title if not provided)
        #[arg(long, help_heading = "Item Properties")]
        name: Option<String>,

        /// Item type (omit for interactive mode)
        #[arg(short = 't', long = "type", help_heading = "Item Properties")]
        item_type: Option<String>,

        /// Upstream references (for requirements)
        #[arg(long, num_args = 1.., help_heading = "Traceability")]
        derives_from: Vec<String>,

        /// Upstream references (for use_case, scenario)
        #[arg(long, num_args = 1.., help_heading = "Traceability")]
        refines: Vec<String>,

        /// Upstream references (for architectures, designs)
        #[arg(long, num_args = 1.., help_heading = "Traceability")]
        satisfies: Vec<String>,

        /// Target platform (for system_architecture)
        #[arg(long, help_heading = "Type-Specific")]
        platform: Option<String>,

        /// Specification statement (for requirements)
        #[arg(long, help_heading = "Type-Specific")]
        specification: Option<String>,

        /// Overwrite existing frontmatter
        #[arg(long, help_heading = "Global Options")]
        force: bool,
    },

    /// Parse documents and build the knowledge graph
    Parse {
        /// Read from specific Git commit/branch
        #[arg(long, value_name = "GIT_REF", help_heading = "Input")]
        at: Option<String>,

        /// Output parsed graph to file (JSON format)
        #[arg(short, long, help_heading = "Output")]
        output: Option<std::path::PathBuf>,
    },

    /// Query items and traceability chains
    Query {
        /// The item identifier to query
        item_id: String,

        /// Limit traversal depth
        #[arg(long, help_heading = "Filters")]
        depth: Option<usize>,

        /// Filter by item type(s)
        #[arg(short = 't', long = "type", help_heading = "Filters")]
        item_types: Vec<String>,

        /// Show downstream chain (toward Detailed Designs)
        #[arg(short, long, help_heading = "Traversal")]
        downstream: bool,

        /// Show upstream chain (toward Solution)
        #[arg(short, long, help_heading = "Traversal")]
        upstream: bool,

        /// Output format
        #[arg(long, default_value = "tree", help_heading = "Output")]
        format: QueryFormat,
    },

    /// Generate coverage and traceability reports
    Report {
        /// Report type
        #[command(subcommand)]
        report_type: ReportType,
    },

    /// Validate graph integrity
    Validate {
        /// Validate at specific Git commit/branch
        #[arg(long, value_name = "GIT_REF", help_heading = "Input")]
        at: Option<String>,

        /// Output format
        #[arg(long, default_value = "text", help_heading = "Output")]
        format: OutputFormat,

        /// Write validation report to file
        #[arg(short, long, help_heading = "Output")]
        output: Option<PathBuf>,

        /// Treat orphan items as errors (default: warnings)
        #[arg(long, help_heading = "Validation")]
        strict: bool,
    },
}

/// Output format for validation and diff.
#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
}

/// Output format for queries.
#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum QueryFormat {
    #[default]
    Tree,
    Json,
}

/// Report types.
#[derive(Subcommand, Debug)]
#[command(disable_help_subcommand = true)]
pub enum ReportType {
    /// Generate coverage report
    Coverage {
        /// Output format
        #[arg(long, default_value = "text", help_heading = "Output")]
        format: ReportFormat,

        /// Write report to file
        #[arg(short, long, help_heading = "Output")]
        output: Option<std::path::PathBuf>,
    },

    /// Generate traceability matrix
    Matrix {
        /// Output format
        #[arg(long, default_value = "text", help_heading = "Output")]
        format: ReportFormat,

        /// Write report to file
        #[arg(short, long, help_heading = "Output")]
        output: Option<std::path::PathBuf>,
    },
}

/// Report output format.
#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum ReportFormat {
    #[default]
    Text,
    Json,
    Csv,
    Html,
}

/// Returns repositories: CLI args take precedence, then config file, then current directory.
fn get_repositories(
    cli: &Cli,
    file_config: Option<&sara_core::config::Config>,
) -> Result<Vec<PathBuf>, std::io::Error> {
    if !cli.repository.is_empty() {
        Ok(cli.repository.clone())
    } else if let Some(config) = file_config {
        if !config.repositories.paths.is_empty() {
            Ok(config.repositories.paths.clone())
        } else {
            Ok(vec![std::env::current_dir()?])
        }
    } else {
        Ok(vec![std::env::current_dir()?])
    }
}

/// Runs the appropriate command.
pub fn run(
    cli: &Cli,
    file_config: Option<&sara_core::config::Config>,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let output = cli.output_config(file_config);
    let repositories = get_repositories(cli, file_config)?;

    let ctx = CommandContext {
        output: output.clone(),
        repositories,
    };

    match &cli.command {
        Commands::Diff {
            ref1,
            ref2,
            format,
            stat,
        } => run_diff(ref1, ref2, *format, *stat, &ctx),
        Commands::Edit {
            item_id,
            description,
            derives_from,
            name,
            platform,
            refines,
            satisfies,
            specification,
        } => {
            let args = EditArgs {
                item_id,
                description,
                derives_from,
                name,
                platform,
                refines,
                satisfies,
                specification,
            };
            run_edit(args, &ctx)
        }
        Commands::Init {
            file,
            derives_from,
            description,
            force,
            id,
            name,
            platform,
            refines,
            satisfies,
            specification,
            item_type,
        } => {
            let args = InitArgs {
                file,
                derives_from,
                description,
                force: *force,
                id,
                name,
                platform,
                refines,
                satisfies,
                specification,
                item_type: item_type.as_deref(),
            };
            run_init(args, &ctx)
        }
        Commands::Parse { at, output } => run_parse(at, output, &ctx),
        Commands::Query {
            item_id,
            depth,
            downstream,
            format,
            item_types,
            upstream,
        } => run_query(
            item_id,
            *depth,
            *downstream,
            *format,
            item_types,
            *upstream,
            &ctx,
        ),
        Commands::Report { report_type } => run_report(report_type, &ctx),
        Commands::Validate {
            at: _,
            format,
            output,
            strict,
        } => run_validate(*format, output, *strict, &ctx),
    }
}

fn run_diff(
    ref1: &str,
    ref2: &str,
    format: OutputFormat,
    stat: bool,
    ctx: &CommandContext,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let opts = diff::CliDiffOptions {
        ref1: ref1.to_string(),
        ref2: ref2.to_string(),
        format,
        stat,
    };
    diff::run(opts, ctx)
}

fn run_edit(args: EditArgs, ctx: &CommandContext) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let opts = edit::CliEditOptions {
        item_id: args.item_id.to_string(),
        description: args.description.clone(),
        derives_from: args.derives_from.clone(),
        name: args.name.clone(),
        platform: args.platform.clone(),
        refines: args.refines.clone(),
        satisfies: args.satisfies.clone(),
        specification: args.specification.clone(),
    };
    edit::run(opts, ctx)
}

fn run_init(args: InitArgs, ctx: &CommandContext) -> Result<ExitCode, Box<dyn std::error::Error>> {
    match args.item_type {
        None => run_init_interactive(&args, ctx),
        Some(item_type) => run_init_non_interactive(&args, item_type, ctx),
    }
}

fn run_init_interactive(
    args: &InitArgs,
    ctx: &CommandContext,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let prefilled = interactive::PrefilledFields {
        file: args.file.clone(),
        item_type: None,
        id: args.id.clone(),
        name: args.name.clone(),
        description: args.description.clone(),
        refines: args.refines.to_vec(),
        derives_from: args.derives_from.to_vec(),
        satisfies: args.satisfies.to_vec(),
        specification: args.specification.clone(),
        platform: args.platform.clone(),
    };

    let mut session = interactive::InteractiveSession {
        graph: None,
        prefilled,
        repositories: &ctx.repositories,
    };

    let result = interactive::run_interactive_session(&mut session);
    match interactive::handle_interactive_result(result, &ctx.output) {
        Ok(Some(input)) => {
            let opts = init::CliInitOptions {
                file: input.file,
                item_type: input.item_type,
                id: Some(input.id),
                name: Some(input.name),
                description: input.description,
                refines: input.traceability.refines,
                derives_from: input.traceability.derives_from,
                satisfies: input.traceability.satisfies,
                specification: input.type_specific.specification,
                platform: input.type_specific.platform,
                force: args.force,
            };
            init::run(opts, ctx)
        }
        Ok(None) => Ok(ExitCode::from(130)),
        Err(interactive::PromptError::NonInteractiveTerminal) => Ok(ExitCode::from(1)),
        Err(interactive::PromptError::MissingParent(_)) => Ok(ExitCode::from(1)),
        Err(_) => Ok(ExitCode::from(1)),
    }
}

fn run_init_non_interactive(
    args: &InitArgs,
    item_type_str: &str,
    ctx: &CommandContext,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let file = args.file.clone().ok_or(
        "File path is required in non-interactive mode. Usage: sara init <FILE> --type <TYPE>",
    )?;

    let parsed_type = sara_core::parse_item_type(item_type_str).ok_or_else(|| {
        format!(
            "Invalid item type: {}. Valid types: solution, use_case, scenario, \
             system_requirement, system_architecture, hardware_requirement, \
             software_requirement, hardware_detailed_design, software_detailed_design",
            item_type_str
        )
    })?;

    let opts = init::CliInitOptions {
        file,
        item_type: parsed_type,
        id: args.id.clone(),
        name: args.name.clone(),
        description: args.description.clone(),
        refines: args.refines.to_vec(),
        derives_from: args.derives_from.to_vec(),
        satisfies: args.satisfies.to_vec(),
        specification: args.specification.clone(),
        platform: args.platform.clone(),
        force: args.force,
    };
    init::run(opts, ctx)
}

fn run_parse(
    at: &Option<String>,
    output: &Option<PathBuf>,
    ctx: &CommandContext,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let opts = parse::ParseOptions {
        at: at.clone(),
        output: output.clone(),
    };
    parse::run(opts, ctx)
}

fn run_query(
    item_id: &str,
    depth: Option<usize>,
    downstream: bool,
    format: QueryFormat,
    item_types: &[String],
    upstream: bool,
    ctx: &CommandContext,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let opts = query::QueryOptions {
        item_id: item_id.to_string(),
        upstream,
        downstream,
        types: query::parse_item_types(item_types),
        depth,
        format,
    };
    query::run(opts, ctx)
}

fn run_report(
    report_type: &ReportType,
    ctx: &CommandContext,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    match report_type {
        ReportType::Coverage { format, output } => {
            let opts = report::CoverageOptions {
                format: *format,
                output: output.clone(),
            };
            report::run_coverage(opts, ctx)
        }
        ReportType::Matrix { format, output } => {
            let opts = report::MatrixOptions {
                format: *format,
                output: output.clone(),
            };
            report::run_matrix(opts, ctx)
        }
    }
}

fn run_validate(
    format: OutputFormat,
    output: &Option<PathBuf>,
    strict: bool,
    ctx: &CommandContext,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let opts = validate::ValidateOptions {
        strict,
        format,
        output: output.clone(),
    };
    validate::run(opts, ctx)
}

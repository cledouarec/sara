//! Init command implementation.

use std::error::Error;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Args;

use sara_core::init::{InitError, InitOptions, InitResult, InitService, parse_item_type};

use super::CommandContext;
use super::interactive::{
    InteractiveSession, PrefilledFields, PromptError, handle_interactive_result,
    run_interactive_session,
};
use crate::output::{OutputConfig, print_error, print_success, print_warning};

/// Arguments for the init command.
#[derive(Args, Debug)]
#[command(verbatim_doc_comment)]
pub struct InitArgs {
    /// Markdown file to initialize (prompted if not provided in interactive mode)
    pub file: Option<PathBuf>,

    /// Item description
    #[arg(short = 'd', long, help_heading = "Item Properties")]
    pub description: Option<String>,

    /// Item identifier (auto-generated if not provided)
    #[arg(long, help_heading = "Item Properties")]
    pub id: Option<String>,

    /// Item name (extracted from title if not provided)
    #[arg(long, help_heading = "Item Properties")]
    pub name: Option<String>,

    /// Item type (omit for interactive mode)
    #[arg(short = 't', long = "type", help_heading = "Item Properties")]
    pub item_type: Option<String>,

    /// Upstream references (for requirements)
    #[arg(long, num_args = 1.., help_heading = "Traceability")]
    pub derives_from: Vec<String>,

    /// Upstream references (for use_case, scenario)
    #[arg(long, num_args = 1.., help_heading = "Traceability")]
    pub refines: Vec<String>,

    /// Upstream references (for architectures, designs)
    #[arg(long, num_args = 1.., help_heading = "Traceability")]
    pub satisfies: Vec<String>,

    /// Peer dependencies (for requirements)
    #[arg(long, num_args = 1.., help_heading = "Traceability")]
    pub depends_on: Vec<String>,

    /// Target platform (for system_architecture)
    #[arg(long, help_heading = "Type-Specific")]
    pub platform: Option<String>,

    /// Specification statement (for requirements)
    #[arg(long, help_heading = "Type-Specific")]
    pub specification: Option<String>,

    /// Overwrite existing frontmatter
    #[arg(long, help_heading = "Global Options")]
    pub force: bool,
}

/// Exit code for invalid option for item type.
const EXIT_INVALID_OPTION: u8 = 3;

/// Exit code for frontmatter already exists.
const EXIT_FRONTMATTER_EXISTS: u8 = 2;

/// Runs the init command.
pub fn run(args: &InitArgs, ctx: &CommandContext) -> Result<ExitCode, Box<dyn Error>> {
    match &args.item_type {
        None => run_interactive(args, ctx),
        Some(item_type) => run_non_interactive(args, item_type, ctx),
    }
}

fn run_interactive(args: &InitArgs, ctx: &CommandContext) -> Result<ExitCode, Box<dyn Error>> {
    let prefilled = PrefilledFields {
        file: args.file.clone(),
        item_type: None,
        id: args.id.clone(),
        name: args.name.clone(),
        description: args.description.clone(),
        refines: args.refines.clone(),
        derives_from: args.derives_from.clone(),
        satisfies: args.satisfies.clone(),
        depends_on: args.depends_on.clone(),
        specification: args.specification.clone(),
        platform: args.platform.clone(),
    };

    let mut session = InteractiveSession {
        graph: None,
        prefilled,
        repositories: &ctx.repositories,
        output: &ctx.output,
    };

    let result = run_interactive_session(&mut session);
    match handle_interactive_result(result, &ctx.output) {
        Ok(Some(input)) => {
            let opts = InitOptions::new(input.file, input.item_type)
                .with_id(input.id)
                .with_name(input.name)
                .maybe_description(input.description)
                .with_refines(input.traceability.refines)
                .with_derives_from(input.traceability.derives_from)
                .with_satisfies(input.traceability.satisfies)
                .with_depends_on(input.traceability.depends_on)
                .maybe_specification(input.type_specific.specification)
                .maybe_platform(input.type_specific.platform)
                .with_force(args.force);
            run_with_options(opts, ctx)
        }
        Ok(None) => Ok(ExitCode::from(130)),
        Err(PromptError::NonInteractiveTerminal) => Ok(ExitCode::from(1)),
        Err(PromptError::MissingParent(_)) => Ok(ExitCode::from(1)),
        Err(_) => Ok(ExitCode::from(1)),
    }
}

fn run_non_interactive(
    args: &InitArgs,
    item_type_str: &str,
    ctx: &CommandContext,
) -> Result<ExitCode, Box<dyn Error>> {
    let file = args.file.clone().ok_or(
        "File path is required in non-interactive mode. Usage: sara init <FILE> --type <TYPE>",
    )?;

    let parsed_type = parse_item_type(item_type_str).ok_or_else(|| {
        format!(
            "Invalid item type: {}. Valid types: solution, use_case, scenario, \
             system_requirement, system_architecture, hardware_requirement, \
             software_requirement, hardware_detailed_design, software_detailed_design",
            item_type_str
        )
    })?;

    let opts = InitOptions::new(file, parsed_type)
        .maybe_id(args.id.clone())
        .maybe_name(args.name.clone())
        .maybe_description(args.description.clone())
        .with_refines(args.refines.clone())
        .with_derives_from(args.derives_from.clone())
        .with_satisfies(args.satisfies.clone())
        .with_depends_on(args.depends_on.clone())
        .maybe_specification(args.specification.clone())
        .maybe_platform(args.platform.clone())
        .with_force(args.force);
    run_with_options(opts, ctx)
}

fn run_with_options(opts: InitOptions, ctx: &CommandContext) -> Result<ExitCode, Box<dyn Error>> {
    let config = &ctx.output;

    let service = InitService::new();

    match service.init(&opts) {
        Ok(result) => {
            print_result(config, &result);
            Ok(ExitCode::SUCCESS)
        }
        Err(InitError::FrontmatterExists(path)) => {
            print_error(
                config,
                &format!(
                    "File {} already has frontmatter. Use --force to overwrite.",
                    path.display()
                ),
            );
            Ok(ExitCode::from(EXIT_FRONTMATTER_EXISTS))
        }
        Err(InitError::InvalidOption(msg)) => {
            print_error(config, &msg);
            Ok(ExitCode::from(EXIT_INVALID_OPTION))
        }
        Err(InitError::Io(e)) => {
            print_error(config, &format!("IO error: {}", e));
            Ok(ExitCode::FAILURE)
        }
    }
}

/// Prints the result of a successful initialization.
fn print_result(config: &OutputConfig, result: &InitResult) {
    if result.updated_existing {
        if result.replaced_frontmatter {
            print_success(
                config,
                &format!("Replaced frontmatter in {}", result.file.display()),
            );
        } else {
            print_success(
                config,
                &format!("Added frontmatter to {}", result.file.display()),
            );
        }
    } else {
        print_success(
            config,
            &format!(
                "Created {} with {} template",
                result.file.display(),
                result.item_type.display_name()
            ),
        );
    }

    print_item_info(config, result);

    if result.needs_specification {
        print_warning(config, "Don't forget to update the specification field!");
    }
}

/// Prints item information after initialization.
fn print_item_info(_config: &OutputConfig, result: &InitResult) {
    let output = format!(
        "\n  ID:   {}\n  Name: {}\n  Type: {}",
        result.id,
        result.name,
        result.item_type.display_name()
    );

    println!("{}", output);
}

#[cfg(test)]
mod tests {
    use sara_core::init::parse_item_type;
    use sara_core::model::ItemType;

    #[test]
    fn test_parse_item_type() {
        assert_eq!(parse_item_type("solution"), Some(ItemType::Solution));
        assert_eq!(parse_item_type("SOL"), Some(ItemType::Solution));
        assert_eq!(parse_item_type("use_case"), Some(ItemType::UseCase));
        assert_eq!(parse_item_type("UC"), Some(ItemType::UseCase));
        assert_eq!(parse_item_type("invalid"), None);
    }
}

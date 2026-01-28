//! Init command implementation with type-specific subcommands.

use std::error::Error;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Subcommand};

use sara_core::model::{AdrStatus, ItemType, TypeFields};
use sara_core::service::{InitError, InitFileOptions, InitResult, create_item};

use super::CommandContext;
use super::interactive::{
    InteractiveSession, PrefilledFields, PromptError, handle_interactive_result,
    run_interactive_session,
};
use crate::output::{OutputConfig, print_error, print_success, print_warning};

/// Common options shared by all init subcommands.
#[derive(Args, Debug, Clone)]
pub struct CommonOptions {
    /// Markdown file to initialize
    pub file: PathBuf,

    /// Item identifier (auto-generated if not provided)
    #[arg(long)]
    pub id: Option<String>,

    /// Item name (extracted from title if not provided)
    #[arg(long)]
    pub name: Option<String>,

    /// Item description
    #[arg(short = 'd', long)]
    pub description: Option<String>,

    /// Overwrite existing frontmatter
    #[arg(long)]
    pub force: bool,
}

/// Arguments for the init command.
#[derive(Args, Debug)]
#[command(verbatim_doc_comment)]
pub struct InitArgs {
    #[command(subcommand)]
    pub command: Option<InitSubcommand>,
}

/// Item type subcommands for init.
#[derive(Subcommand, Debug)]
pub enum InitSubcommand {
    /// Create an Architecture Decision Record
    #[command(name = "architecture-decision-record", visible_alias = "adr")]
    Adr(AdrArgs),

    /// Create a Solution
    #[command(name = "solution", visible_alias = "sol")]
    Solution(SolutionArgs),

    /// Create a Use Case
    #[command(name = "use-case", visible_alias = "uc")]
    UseCase(UseCaseArgs),

    /// Create a Scenario
    #[command(name = "scenario", visible_alias = "scen")]
    Scenario(ScenarioArgs),

    /// Create a System Requirement
    #[command(name = "system-requirement", visible_alias = "sysreq")]
    SystemRequirement(SystemRequirementArgs),

    /// Create a System Architecture
    #[command(name = "system-architecture", visible_alias = "sysarch")]
    SystemArchitecture(SystemArchitectureArgs),

    /// Create a Software Requirement
    #[command(name = "software-requirement", visible_alias = "swreq")]
    SoftwareRequirement(SoftwareRequirementArgs),

    /// Create a Hardware Requirement
    #[command(name = "hardware-requirement", visible_alias = "hwreq")]
    HardwareRequirement(HardwareRequirementArgs),

    /// Create a Software Detailed Design
    #[command(name = "software-detailed-design", visible_alias = "swdd")]
    SoftwareDetailedDesign(SoftwareDetailedDesignArgs),

    /// Create a Hardware Detailed Design
    #[command(name = "hardware-detailed-design", visible_alias = "hwdd")]
    HardwareDetailedDesign(HardwareDetailedDesignArgs),
}

// =============================================================================
// Type-specific argument structs
// =============================================================================

/// Arguments for creating an ADR.
#[derive(Args, Debug)]
pub struct AdrArgs {
    #[command(flatten)]
    pub common: CommonOptions,

    /// ADR status (proposed, accepted, deprecated, superseded)
    #[arg(long, short = 's')]
    pub status: Option<String>,

    /// ADR deciders
    #[arg(long, num_args = 1..)]
    pub deciders: Vec<String>,

    /// Design artifacts this ADR justifies
    #[arg(long, short = 'j', num_args = 1..)]
    pub justifies: Vec<String>,

    /// Older ADRs this decision supersedes
    #[arg(long, num_args = 1..)]
    pub supersedes: Vec<String>,
}

/// Arguments for creating a Solution.
#[derive(Args, Debug)]
pub struct SolutionArgs {
    #[command(flatten)]
    pub common: CommonOptions,
}

/// Arguments for creating a Use Case.
#[derive(Args, Debug)]
pub struct UseCaseArgs {
    #[command(flatten)]
    pub common: CommonOptions,

    /// Solution this use case refines
    #[arg(long, num_args = 1..)]
    pub refines: Vec<String>,
}

/// Arguments for creating a Scenario.
#[derive(Args, Debug)]
pub struct ScenarioArgs {
    #[command(flatten)]
    pub common: CommonOptions,

    /// Use case this scenario refines
    #[arg(long, num_args = 1..)]
    pub refines: Vec<String>,
}

/// Arguments for creating a System Requirement.
#[derive(Args, Debug)]
pub struct SystemRequirementArgs {
    #[command(flatten)]
    pub common: CommonOptions,

    /// Specification statement
    #[arg(long)]
    pub specification: Option<String>,

    /// Upstream references (scenarios this requirement derives from)
    #[arg(long, num_args = 1..)]
    pub derives_from: Vec<String>,

    /// Peer dependencies
    #[arg(long, num_args = 1..)]
    pub depends_on: Vec<String>,
}

/// Arguments for creating a System Architecture.
#[derive(Args, Debug)]
pub struct SystemArchitectureArgs {
    #[command(flatten)]
    pub common: CommonOptions,

    /// Target platform
    #[arg(long)]
    pub platform: Option<String>,

    /// System requirements this architecture satisfies
    #[arg(long, num_args = 1..)]
    pub satisfies: Vec<String>,
}

/// Arguments for creating a Software Requirement.
#[derive(Args, Debug)]
pub struct SoftwareRequirementArgs {
    #[command(flatten)]
    pub common: CommonOptions,

    /// Specification statement
    #[arg(long)]
    pub specification: Option<String>,

    /// Upstream references (system requirements this derives from)
    #[arg(long, num_args = 1..)]
    pub derives_from: Vec<String>,

    /// Peer dependencies
    #[arg(long, num_args = 1..)]
    pub depends_on: Vec<String>,
}

/// Arguments for creating a Hardware Requirement.
#[derive(Args, Debug)]
pub struct HardwareRequirementArgs {
    #[command(flatten)]
    pub common: CommonOptions,

    /// Specification statement
    #[arg(long)]
    pub specification: Option<String>,

    /// Upstream references (system requirements this derives from)
    #[arg(long, num_args = 1..)]
    pub derives_from: Vec<String>,

    /// Peer dependencies
    #[arg(long, num_args = 1..)]
    pub depends_on: Vec<String>,
}

/// Arguments for creating a Software Detailed Design.
#[derive(Args, Debug)]
pub struct SoftwareDetailedDesignArgs {
    #[command(flatten)]
    pub common: CommonOptions,

    /// Software requirements this design satisfies
    #[arg(long, num_args = 1..)]
    pub satisfies: Vec<String>,
}

/// Arguments for creating a Hardware Detailed Design.
#[derive(Args, Debug)]
pub struct HardwareDetailedDesignArgs {
    #[command(flatten)]
    pub common: CommonOptions,

    /// Hardware requirements this design satisfies
    #[arg(long, num_args = 1..)]
    pub satisfies: Vec<String>,
}

// =============================================================================
// Exit codes
// =============================================================================

/// Exit code for frontmatter already exists.
const EXIT_FRONTMATTER_EXISTS: u8 = 2;

/// Exit code for invalid option for item type.
const EXIT_INVALID_OPTION: u8 = 3;

// =============================================================================
// Command execution
// =============================================================================

/// Runs the init command.
pub fn run(args: &InitArgs, ctx: &CommandContext) -> Result<ExitCode, Box<dyn Error>> {
    match &args.command {
        None => run_interactive(ctx),
        Some(subcommand) => run_subcommand(subcommand, ctx),
    }
}

fn run_interactive(ctx: &CommandContext) -> Result<ExitCode, Box<dyn Error>> {
    let prefilled = PrefilledFields {
        file: None,
        item_type: None,
        id: None,
        name: None,
        description: None,
        refines: Vec::new(),
        derives_from: Vec::new(),
        satisfies: Vec::new(),
        depends_on: Vec::new(),
        specification: None,
        platform: None,
        deciders: Vec::new(),
        justifies: Vec::new(),
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
            // Build TypeFields from interactive input
            let fields = build_type_fields_from_interactive(&input);

            let opts = InitFileOptions::with_fields(input.file, input.item_type, fields)
                .with_id(input.id)
                .with_name(input.name)
                .maybe_description(input.description)
                .with_force(false);

            run_with_options(opts, ctx)
        }
        Ok(None) => Ok(ExitCode::from(130)),
        Err(PromptError::NonInteractiveTerminal) => Ok(ExitCode::from(1)),
        Err(PromptError::MissingParent(_)) => Ok(ExitCode::from(1)),
        Err(_) => Ok(ExitCode::from(1)),
    }
}

/// Builds TypeFields from interactive session input.
fn build_type_fields_from_interactive(input: &super::interactive::InteractiveInput) -> TypeFields {
    use super::interactive::TypeSpecificInput;

    let mut fields = TypeFields::new()
        .with_refines(input.traceability.refines.clone())
        .with_derives_from(input.traceability.derives_from.clone())
        .with_satisfies(input.traceability.satisfies.clone())
        .with_depends_on(input.traceability.depends_on.clone())
        .with_justifies(input.traceability.justifies.clone());

    // Add type-specific fields
    match &input.type_specific {
        TypeSpecificInput::Requirement { specification } => {
            if let Some(spec) = specification {
                fields = fields.with_specification(spec);
            }
        }
        TypeSpecificInput::SystemArchitecture { platform } => {
            if let Some(plat) = platform {
                fields = fields.with_platform(plat);
            }
        }
        TypeSpecificInput::Adr { deciders } => {
            fields = fields.with_deciders(deciders.clone());
        }
        TypeSpecificInput::None => {}
    }

    fields
}

fn run_subcommand(
    subcommand: &InitSubcommand,
    ctx: &CommandContext,
) -> Result<ExitCode, Box<dyn Error>> {
    let opts = match subcommand {
        InitSubcommand::Adr(args) => {
            let mut fields = TypeFields::new()
                .with_deciders(args.deciders.clone())
                .with_justifies(args.justifies.clone())
                .with_supersedes(args.supersedes.clone());

            if let Some(ref status_str) = args.status
                && let Ok(status) = status_str.parse::<AdrStatus>()
            {
                fields = fields.with_status(status);
            }

            InitFileOptions::with_fields(
                args.common.file.clone(),
                ItemType::ArchitectureDecisionRecord,
                fields,
            )
            .maybe_id(args.common.id.clone())
            .maybe_name(args.common.name.clone())
            .maybe_description(args.common.description.clone())
            .with_force(args.common.force)
        }

        InitSubcommand::Solution(args) => {
            InitFileOptions::new(args.common.file.clone(), ItemType::Solution)
                .maybe_id(args.common.id.clone())
                .maybe_name(args.common.name.clone())
                .maybe_description(args.common.description.clone())
                .with_force(args.common.force)
        }

        InitSubcommand::UseCase(args) => {
            let fields = TypeFields::new().with_refines(args.refines.clone());

            InitFileOptions::with_fields(args.common.file.clone(), ItemType::UseCase, fields)
                .maybe_id(args.common.id.clone())
                .maybe_name(args.common.name.clone())
                .maybe_description(args.common.description.clone())
                .with_force(args.common.force)
        }

        InitSubcommand::Scenario(args) => {
            let fields = TypeFields::new().with_refines(args.refines.clone());

            InitFileOptions::with_fields(args.common.file.clone(), ItemType::Scenario, fields)
                .maybe_id(args.common.id.clone())
                .maybe_name(args.common.name.clone())
                .maybe_description(args.common.description.clone())
                .with_force(args.common.force)
        }

        InitSubcommand::SystemRequirement(args) => {
            let mut fields = TypeFields::new()
                .with_derives_from(args.derives_from.clone())
                .with_depends_on(args.depends_on.clone());

            if let Some(ref spec) = args.specification {
                fields = fields.with_specification(spec);
            }

            InitFileOptions::with_fields(
                args.common.file.clone(),
                ItemType::SystemRequirement,
                fields,
            )
            .maybe_id(args.common.id.clone())
            .maybe_name(args.common.name.clone())
            .maybe_description(args.common.description.clone())
            .with_force(args.common.force)
        }

        InitSubcommand::SystemArchitecture(args) => {
            let mut fields = TypeFields::new().with_satisfies(args.satisfies.clone());

            if let Some(ref plat) = args.platform {
                fields = fields.with_platform(plat);
            }

            InitFileOptions::with_fields(
                args.common.file.clone(),
                ItemType::SystemArchitecture,
                fields,
            )
            .maybe_id(args.common.id.clone())
            .maybe_name(args.common.name.clone())
            .maybe_description(args.common.description.clone())
            .with_force(args.common.force)
        }

        InitSubcommand::SoftwareRequirement(args) => {
            let mut fields = TypeFields::new()
                .with_derives_from(args.derives_from.clone())
                .with_depends_on(args.depends_on.clone());

            if let Some(ref spec) = args.specification {
                fields = fields.with_specification(spec);
            }

            InitFileOptions::with_fields(
                args.common.file.clone(),
                ItemType::SoftwareRequirement,
                fields,
            )
            .maybe_id(args.common.id.clone())
            .maybe_name(args.common.name.clone())
            .maybe_description(args.common.description.clone())
            .with_force(args.common.force)
        }

        InitSubcommand::HardwareRequirement(args) => {
            let mut fields = TypeFields::new()
                .with_derives_from(args.derives_from.clone())
                .with_depends_on(args.depends_on.clone());

            if let Some(ref spec) = args.specification {
                fields = fields.with_specification(spec);
            }

            InitFileOptions::with_fields(
                args.common.file.clone(),
                ItemType::HardwareRequirement,
                fields,
            )
            .maybe_id(args.common.id.clone())
            .maybe_name(args.common.name.clone())
            .maybe_description(args.common.description.clone())
            .with_force(args.common.force)
        }

        InitSubcommand::SoftwareDetailedDesign(args) => {
            let fields = TypeFields::new().with_satisfies(args.satisfies.clone());

            InitFileOptions::with_fields(
                args.common.file.clone(),
                ItemType::SoftwareDetailedDesign,
                fields,
            )
            .maybe_id(args.common.id.clone())
            .maybe_name(args.common.name.clone())
            .maybe_description(args.common.description.clone())
            .with_force(args.common.force)
        }

        InitSubcommand::HardwareDetailedDesign(args) => {
            let fields = TypeFields::new().with_satisfies(args.satisfies.clone());

            InitFileOptions::with_fields(
                args.common.file.clone(),
                ItemType::HardwareDetailedDesign,
                fields,
            )
            .maybe_id(args.common.id.clone())
            .maybe_name(args.common.name.clone())
            .maybe_description(args.common.description.clone())
            .with_force(args.common.force)
        }
    };

    run_with_options(opts, ctx)
}

fn run_with_options(
    opts: InitFileOptions,
    ctx: &CommandContext,
) -> Result<ExitCode, Box<dyn Error>> {
    let config = &ctx.output;

    match create_item(&opts) {
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
        Err(InitError::Validation(e)) => {
            print_error(config, &format!("{}", e));
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
    use super::*;
    use clap::CommandFactory;

    // Implement CommandFactory for InitArgs to enable testing
    impl CommandFactory for InitArgs {
        fn command() -> clap::Command {
            <Self as clap::Args>::augment_args(clap::Command::new("init"))
        }

        fn command_for_update() -> clap::Command {
            <Self as clap::Args>::augment_args_for_update(clap::Command::new("init"))
        }
    }

    #[test]
    fn test_init_subcommands_exist() {
        // Verify all subcommands are properly configured
        let cmd = InitArgs::command();

        // Check that init has subcommands
        let subcommands: Vec<&clap::Command> = cmd.get_subcommands().collect();
        assert!(!subcommands.is_empty(), "Init should have subcommands");

        // Check specific subcommands exist
        let names: Vec<&str> = subcommands.iter().map(|s| s.get_name()).collect();
        assert!(names.contains(&"architecture-decision-record"));
        assert!(names.contains(&"solution"));
        assert!(names.contains(&"use-case"));
        assert!(names.contains(&"scenario"));
        assert!(names.contains(&"system-requirement"));
        assert!(names.contains(&"system-architecture"));
        assert!(names.contains(&"software-requirement"));
        assert!(names.contains(&"hardware-requirement"));
        assert!(names.contains(&"software-detailed-design"));
        assert!(names.contains(&"hardware-detailed-design"));
    }

    #[test]
    fn test_aliases_exist() {
        let cmd = InitArgs::command();

        for sub in cmd.get_subcommands() {
            let aliases: Vec<&str> = sub.get_visible_aliases().collect();
            match sub.get_name() {
                "architecture-decision-record" => assert!(aliases.contains(&"adr")),
                "solution" => assert!(aliases.contains(&"sol")),
                "use-case" => assert!(aliases.contains(&"uc")),
                "scenario" => assert!(aliases.contains(&"scen")),
                "system-requirement" => assert!(aliases.contains(&"sysreq")),
                "system-architecture" => assert!(aliases.contains(&"sysarch")),
                "software-requirement" => assert!(aliases.contains(&"swreq")),
                "hardware-requirement" => assert!(aliases.contains(&"hwreq")),
                "software-detailed-design" => assert!(aliases.contains(&"swdd")),
                "hardware-detailed-design" => assert!(aliases.contains(&"hwdd")),
                _ => {}
            }
        }
    }
}

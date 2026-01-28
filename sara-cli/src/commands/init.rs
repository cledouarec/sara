//! Init command implementation with type-specific subcommands.

use std::error::Error;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Subcommand};

use sara_core::init::{InitError, InitOptions, InitResult, InitService, TypeConfig};

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
            // Build TypeConfig from interactive input
            let type_config = build_type_config_from_interactive(&input);

            let opts = InitOptions::new(input.file, type_config)
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

/// Builds a TypeConfig from interactive session input.
fn build_type_config_from_interactive(input: &super::interactive::InteractiveInput) -> TypeConfig {
    use sara_core::model::ItemType;

    match input.item_type {
        ItemType::Solution => TypeConfig::Solution,
        ItemType::UseCase => TypeConfig::UseCase {
            refines: input.traceability.refines.clone(),
        },
        ItemType::Scenario => TypeConfig::Scenario {
            refines: input.traceability.refines.clone(),
        },
        ItemType::SystemRequirement => TypeConfig::SystemRequirement {
            specification: input.type_specific.specification.clone(),
            derives_from: input.traceability.derives_from.clone(),
            depends_on: input.traceability.depends_on.clone(),
        },
        ItemType::SystemArchitecture => TypeConfig::SystemArchitecture {
            platform: input.type_specific.platform.clone(),
            satisfies: input.traceability.satisfies.clone(),
        },
        ItemType::SoftwareRequirement => TypeConfig::SoftwareRequirement {
            specification: input.type_specific.specification.clone(),
            derives_from: input.traceability.derives_from.clone(),
            depends_on: input.traceability.depends_on.clone(),
        },
        ItemType::HardwareRequirement => TypeConfig::HardwareRequirement {
            specification: input.type_specific.specification.clone(),
            derives_from: input.traceability.derives_from.clone(),
            depends_on: input.traceability.depends_on.clone(),
        },
        ItemType::SoftwareDetailedDesign => TypeConfig::SoftwareDetailedDesign {
            satisfies: input.traceability.satisfies.clone(),
        },
        ItemType::HardwareDetailedDesign => TypeConfig::HardwareDetailedDesign {
            satisfies: input.traceability.satisfies.clone(),
        },
        ItemType::ArchitectureDecisionRecord => TypeConfig::Adr {
            status: None,
            deciders: Vec::new(),
            justifies: Vec::new(),
            supersedes: Vec::new(),
            superseded_by: None,
        },
    }
}

fn run_subcommand(
    subcommand: &InitSubcommand,
    ctx: &CommandContext,
) -> Result<ExitCode, Box<dyn Error>> {
    let opts = match subcommand {
        InitSubcommand::Adr(args) => {
            let type_config = TypeConfig::Adr {
                status: args.status.clone(),
                deciders: args.deciders.clone(),
                justifies: args.justifies.clone(),
                supersedes: args.supersedes.clone(),
                superseded_by: None,
            };

            InitOptions::new(args.common.file.clone(), type_config)
                .maybe_id(args.common.id.clone())
                .maybe_name(args.common.name.clone())
                .maybe_description(args.common.description.clone())
                .with_force(args.common.force)
        }

        InitSubcommand::Solution(args) => {
            InitOptions::new(args.common.file.clone(), TypeConfig::Solution)
                .maybe_id(args.common.id.clone())
                .maybe_name(args.common.name.clone())
                .maybe_description(args.common.description.clone())
                .with_force(args.common.force)
        }

        InitSubcommand::UseCase(args) => {
            let type_config = TypeConfig::UseCase {
                refines: args.refines.clone(),
            };

            InitOptions::new(args.common.file.clone(), type_config)
                .maybe_id(args.common.id.clone())
                .maybe_name(args.common.name.clone())
                .maybe_description(args.common.description.clone())
                .with_force(args.common.force)
        }

        InitSubcommand::Scenario(args) => {
            let type_config = TypeConfig::Scenario {
                refines: args.refines.clone(),
            };

            InitOptions::new(args.common.file.clone(), type_config)
                .maybe_id(args.common.id.clone())
                .maybe_name(args.common.name.clone())
                .maybe_description(args.common.description.clone())
                .with_force(args.common.force)
        }

        InitSubcommand::SystemRequirement(args) => {
            let type_config = TypeConfig::SystemRequirement {
                specification: args.specification.clone(),
                derives_from: args.derives_from.clone(),
                depends_on: args.depends_on.clone(),
            };

            InitOptions::new(args.common.file.clone(), type_config)
                .maybe_id(args.common.id.clone())
                .maybe_name(args.common.name.clone())
                .maybe_description(args.common.description.clone())
                .with_force(args.common.force)
        }

        InitSubcommand::SystemArchitecture(args) => {
            let type_config = TypeConfig::SystemArchitecture {
                platform: args.platform.clone(),
                satisfies: args.satisfies.clone(),
            };

            InitOptions::new(args.common.file.clone(), type_config)
                .maybe_id(args.common.id.clone())
                .maybe_name(args.common.name.clone())
                .maybe_description(args.common.description.clone())
                .with_force(args.common.force)
        }

        InitSubcommand::SoftwareRequirement(args) => {
            let type_config = TypeConfig::SoftwareRequirement {
                specification: args.specification.clone(),
                derives_from: args.derives_from.clone(),
                depends_on: args.depends_on.clone(),
            };

            InitOptions::new(args.common.file.clone(), type_config)
                .maybe_id(args.common.id.clone())
                .maybe_name(args.common.name.clone())
                .maybe_description(args.common.description.clone())
                .with_force(args.common.force)
        }

        InitSubcommand::HardwareRequirement(args) => {
            let type_config = TypeConfig::HardwareRequirement {
                specification: args.specification.clone(),
                derives_from: args.derives_from.clone(),
                depends_on: args.depends_on.clone(),
            };

            InitOptions::new(args.common.file.clone(), type_config)
                .maybe_id(args.common.id.clone())
                .maybe_name(args.common.name.clone())
                .maybe_description(args.common.description.clone())
                .with_force(args.common.force)
        }

        InitSubcommand::SoftwareDetailedDesign(args) => {
            let type_config = TypeConfig::SoftwareDetailedDesign {
                satisfies: args.satisfies.clone(),
            };

            InitOptions::new(args.common.file.clone(), type_config)
                .maybe_id(args.common.id.clone())
                .maybe_name(args.common.name.clone())
                .maybe_description(args.common.description.clone())
                .with_force(args.common.force)
        }

        InitSubcommand::HardwareDetailedDesign(args) => {
            let type_config = TypeConfig::HardwareDetailedDesign {
                satisfies: args.satisfies.clone(),
            };

            InitOptions::new(args.common.file.clone(), type_config)
                .maybe_id(args.common.id.clone())
                .maybe_name(args.common.name.clone())
                .maybe_description(args.common.description.clone())
                .with_force(args.common.force)
        }
    };

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

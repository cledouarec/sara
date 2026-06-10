//! Init command implementation with type-specific subcommands.

use std::error::Error;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Subcommand};
use sara_core::model::ItemType;
use sara_core::service::{InitError, InitOptions, InitResult, InitService, TypeConfig};

use sara_core::config::{Config, OutputConfig};

use super::interactive::{
    InteractiveSession, PrefilledFields, PromptError, handle_interactive_result,
    run_interactive_session,
};
use crate::output::{print_error, print_success, print_warning};

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
pub fn run(args: &InitArgs, config: &Config) -> Result<ExitCode, Box<dyn Error>> {
    match &args.command {
        None => run_interactive(config),
        Some(subcommand) => run_subcommand(subcommand, config),
    }
}

fn run_interactive(config: &Config) -> Result<ExitCode, Box<dyn Error>> {
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
        repositories: &config.repositories.paths,
        output: &config.output,
    };

    let result = run_interactive_session(&mut session);
    match handle_interactive_result(result, &config.output) {
        Ok(Some(input)) => {
            // Build TypeConfig from interactive input
            let type_config = build_type_config_from_interactive(&input);

            let opts = InitOptions::new(input.file, type_config)
                .with_id(input.id)
                .with_name(input.name)
                .maybe_description(input.description)
                .with_force(false);

            run_with_options(opts, config)
        }
        Ok(None) => Ok(ExitCode::from(130)),
        Err(PromptError::NonInteractiveTerminal) => Ok(ExitCode::from(1)),
        Err(PromptError::MissingParent(_)) => Ok(ExitCode::from(1)),
        Err(_) => Ok(ExitCode::from(1)),
    }
}

/// Builds a TypeConfig from interactive session input.
fn build_type_config_from_interactive(input: &super::interactive::InteractiveInput) -> TypeConfig {
    let mut config = TypeConfig::new(input.item_type);
    for (name, field_input) in &input.type_specific {
        config = config.field(name.clone(), field_input.clone());
    }
    config
        .relation("refines", input.traceability.refines.clone())
        .relation("derives_from", input.traceability.derives_from.clone())
        .relation("satisfies", input.traceability.satisfies.clone())
        .relation("depends_on", input.traceability.depends_on.clone())
        .relation("justifies", input.traceability.justifies.clone())
}

fn run_subcommand(
    subcommand: &InitSubcommand,
    config: &Config,
) -> Result<ExitCode, Box<dyn Error>> {
    let (common, type_config) = match subcommand {
        InitSubcommand::Adr(args) => (
            &args.common,
            TypeConfig::new(ItemType::ARCHITECTURE_DECISION_RECORD)
                .maybe_text_field("status", args.status.clone())
                .list_field("deciders", args.deciders.clone())
                .relation("justifies", args.justifies.clone())
                .relation("supersedes", args.supersedes.clone()),
        ),
        InitSubcommand::Solution(args) => (&args.common, TypeConfig::new(ItemType::SOLUTION)),
        InitSubcommand::UseCase(args) => (
            &args.common,
            TypeConfig::new(ItemType::USE_CASE).relation("refines", args.refines.clone()),
        ),
        InitSubcommand::Scenario(args) => (
            &args.common,
            TypeConfig::new(ItemType::SCENARIO).relation("refines", args.refines.clone()),
        ),
        InitSubcommand::SystemRequirement(args) => (
            &args.common,
            TypeConfig::new(ItemType::SYSTEM_REQUIREMENT)
                .maybe_text_field("specification", args.specification.clone())
                .relation("derives_from", args.derives_from.clone())
                .relation("depends_on", args.depends_on.clone()),
        ),
        InitSubcommand::SystemArchitecture(args) => (
            &args.common,
            TypeConfig::new(ItemType::SYSTEM_ARCHITECTURE)
                .maybe_text_field("platform", args.platform.clone())
                .relation("satisfies", args.satisfies.clone()),
        ),
        InitSubcommand::SoftwareRequirement(args) => (
            &args.common,
            TypeConfig::new(ItemType::SOFTWARE_REQUIREMENT)
                .maybe_text_field("specification", args.specification.clone())
                .relation("derives_from", args.derives_from.clone())
                .relation("depends_on", args.depends_on.clone()),
        ),
        InitSubcommand::HardwareRequirement(args) => (
            &args.common,
            TypeConfig::new(ItemType::HARDWARE_REQUIREMENT)
                .maybe_text_field("specification", args.specification.clone())
                .relation("derives_from", args.derives_from.clone())
                .relation("depends_on", args.depends_on.clone()),
        ),
        InitSubcommand::SoftwareDetailedDesign(args) => (
            &args.common,
            TypeConfig::new(ItemType::SOFTWARE_DETAILED_DESIGN)
                .relation("satisfies", args.satisfies.clone()),
        ),
        InitSubcommand::HardwareDetailedDesign(args) => (
            &args.common,
            TypeConfig::new(ItemType::HARDWARE_DETAILED_DESIGN)
                .relation("satisfies", args.satisfies.clone()),
        ),
    };

    let opts = InitOptions::new(common.file.clone(), type_config)
        .maybe_id(common.id.clone())
        .maybe_name(common.name.clone())
        .maybe_description(common.description.clone())
        .with_force(common.force);

    run_with_options(opts, config)
}

fn run_with_options(opts: InitOptions, config: &Config) -> Result<ExitCode, Box<dyn Error>> {
    let output = &config.output;

    let service = InitService::new();

    match service.init(&opts) {
        Ok(result) => {
            print_result(output, &result);
            Ok(ExitCode::SUCCESS)
        }
        Err(InitError::FrontmatterExists(path)) => {
            print_error(
                output,
                &format!(
                    "File {} already has frontmatter. Use --force to overwrite.",
                    path.display()
                ),
            );
            Ok(ExitCode::from(EXIT_FRONTMATTER_EXISTS))
        }
        Err(InitError::InvalidOption(msg)) => {
            print_error(output, &msg);
            Ok(ExitCode::from(EXIT_INVALID_OPTION))
        }
        Err(InitError::Io(e)) => {
            print_error(output, &format!("IO error: {}", e));
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
    use clap::CommandFactory;

    use super::*;

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

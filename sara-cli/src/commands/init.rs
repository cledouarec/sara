//! Init command implementation with schema-driven type subcommands.
//!
//! One subcommand is generated per item type of the active schema: its name
//! is the kebab-case type id, its visible alias the lowercase id prefix, and
//! its flags are derived from the declared fields and relations. Types added
//! by a custom schema therefore get their `sara init <type>` command without
//! recompiling.

use std::error::Error;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Arg, ArgAction, ArgMatches, Args, Command, FromArgMatches, value_parser};
use sara_core::model::{FIELD_DESCRIPTION, FIELD_ID, FIELD_NAME, ItemType};
use sara_core::schema::FieldType;
use sara_core::service::{InitError, InitOptions, InitResult, InitService, TypeConfig};

use sara_core::config::{Config, OutputConfig};

use super::interactive::{
    InteractiveSession, PromptError, handle_interactive_result, run_interactive_session,
};
use crate::output::{print_error, print_success, print_warning};

/// Arguments for the init command.
#[derive(Args, Debug)]
#[command(verbatim_doc_comment)]
pub struct InitArgs {
    #[command(subcommand)]
    pub command: Option<InitSubcommand>,
}

/// Parsed `init <type>` subcommand, resolved against the active schema.
#[derive(Debug)]
pub struct InitSubcommand {
    /// Markdown file to initialize.
    file: PathBuf,
    /// Item identifier (auto-generated if not provided).
    id: Option<String>,
    /// Item name (extracted from title if not provided).
    name: Option<String>,
    /// Item description.
    description: Option<String>,
    /// Overwrite existing frontmatter.
    force: bool,
    /// Field and relation inputs for the selected type.
    type_config: TypeConfig,
}

/// Argument id of the positional file path.
const ARG_FILE: &str = "file";

/// Argument id of the overwrite flag.
const ARG_FORCE: &str = "force";

/// Short flags preserved from the historical per-type commands.
const SHORT_FLAGS: &[(&str, char)] = &[("status", 's'), ("justifies", 'j')];

/// Initial letters whose display name takes the article "an" in help texts.
const AN_ARTICLE_INITIALS: [char; 5] = ['A', 'E', 'I', 'O', 'U'];

/// Returns the CLI name of a type's init subcommand (kebab-case type id).
fn subcommand_name(item_type: ItemType) -> String {
    item_type.as_str().replace('_', "-")
}

/// Returns the short flag historically attached to a field or relation.
fn short_flag(name: &str) -> Option<char> {
    SHORT_FLAGS
        .iter()
        .find(|(flag_name, _)| *flag_name == name)
        .map(|(_, short)| *short)
}

/// Builds the clap subcommand for one item type of the active schema.
fn type_command(item_type: ItemType) -> Command {
    let display_name = item_type.display_name();
    let article = if display_name.starts_with(AN_ARTICLE_INITIALS) {
        "an"
    } else {
        "a"
    };
    let mut command = Command::new(subcommand_name(item_type))
        .about(format!("Create {article} {display_name}"))
        .arg(
            Arg::new(ARG_FILE)
                .value_name("FILE")
                .value_parser(value_parser!(PathBuf))
                .required(true)
                .help("Markdown file to initialize"),
        )
        .arg(
            Arg::new(FIELD_ID)
                .long(FIELD_ID)
                .help("Item identifier (auto-generated if not provided)"),
        )
        .arg(
            Arg::new(FIELD_NAME)
                .long(FIELD_NAME)
                .help("Item name (extracted from title if not provided)"),
        )
        .arg(
            Arg::new(FIELD_DESCRIPTION)
                .long(FIELD_DESCRIPTION)
                .short('d')
                .help("Item description"),
        )
        .arg(
            Arg::new(ARG_FORCE)
                .long(ARG_FORCE)
                .action(ArgAction::SetTrue)
                .help("Overwrite existing frontmatter"),
        );

    let alias = item_type.prefix().to_lowercase();
    if !alias.is_empty() && alias != command.get_name() {
        command = command.visible_alias(alias);
    }

    for field in item_type.declared_fields() {
        let mut arg = Arg::new(field.name.clone())
            .long(field.name.replace('_', "-"))
            .help(field_help(field));
        if let Some(short) = short_flag(&field.name) {
            arg = arg.short(short);
        }
        if matches!(field.field_type, FieldType::List(_)) {
            arg = arg.num_args(1..).action(ArgAction::Append);
        }
        command = command.arg(arg);
    }

    for relation in item_type.declared_relations() {
        let mut arg = Arg::new(relation.as_str())
            .long(relation.as_str().replace('_', "-"))
            .num_args(1..)
            .action(ArgAction::Append)
            .help(format!(
                "Items this {} {}",
                item_type.display_name().to_lowercase(),
                relation.as_str().replace('_', " ")
            ));
        if let Some(short) = short_flag(relation.as_str()) {
            arg = arg.short(short);
        }
        command = command.arg(arg);
    }

    command
}

/// Returns the help text of a declared field, listing enum values when known.
pub(super) fn field_help(field: &sara_core::schema::FieldDef) -> String {
    match &field.field_type {
        FieldType::Enum { values } => {
            format!("{} ({})", field.display_name, values.join(", "))
        }
        _ => field.display_name.clone(),
    }
}

impl InitSubcommand {
    /// Resolves the parsed matches of one type subcommand.
    fn from_type_matches(name: &str, matches: &ArgMatches) -> Result<Self, clap::Error> {
        let item_type = ItemType::all()
            .into_iter()
            .find(|t| subcommand_name(*t) == name)
            .ok_or_else(|| {
                clap::Error::raw(
                    clap::error::ErrorKind::InvalidSubcommand,
                    format!("unknown item type subcommand `{name}`"),
                )
            })?;

        let mut type_config = TypeConfig::new(item_type);
        for field in item_type.declared_fields() {
            if matches!(field.field_type, FieldType::List(_)) {
                let values: Vec<String> = matches
                    .get_many::<String>(&field.name)
                    .map(|v| v.cloned().collect())
                    .unwrap_or_default();
                type_config = type_config.list_field(&field.name, values);
            } else {
                type_config = type_config
                    .maybe_text_field(&field.name, matches.get_one::<String>(&field.name).cloned());
            }
        }
        for relation in item_type.declared_relations() {
            let targets: Vec<String> = matches
                .get_many::<String>(relation.as_str())
                .map(|v| v.cloned().collect())
                .unwrap_or_default();
            type_config = type_config.relation(relation.as_str(), targets);
        }

        Ok(Self {
            file: matches
                .get_one::<PathBuf>(ARG_FILE)
                .cloned()
                .unwrap_or_default(),
            id: matches.get_one::<String>(FIELD_ID).cloned(),
            name: matches.get_one::<String>(FIELD_NAME).cloned(),
            description: matches.get_one::<String>(FIELD_DESCRIPTION).cloned(),
            force: matches.get_flag(ARG_FORCE),
            type_config,
        })
    }

    /// Converts the parsed subcommand into init service options.
    fn to_init_options(&self) -> InitOptions {
        InitOptions::new(self.file.clone(), self.type_config.clone())
            .maybe_id(self.id.clone())
            .maybe_name(self.name.clone())
            .maybe_description(self.description.clone())
            .with_force(self.force)
    }
}

impl FromArgMatches for InitSubcommand {
    fn from_arg_matches(matches: &ArgMatches) -> Result<Self, clap::Error> {
        let (name, sub_matches) = matches.subcommand().ok_or_else(|| {
            clap::Error::raw(
                clap::error::ErrorKind::MissingSubcommand,
                "an item type subcommand is required",
            )
        })?;
        Self::from_type_matches(name, sub_matches)
    }

    fn update_from_arg_matches(&mut self, matches: &ArgMatches) -> Result<(), clap::Error> {
        *self = Self::from_arg_matches(matches)?;
        Ok(())
    }
}

impl clap::Subcommand for InitSubcommand {
    fn augment_subcommands(command: Command) -> Command {
        ItemType::all()
            .into_iter()
            .fold(command, |command, item_type| {
                command.subcommand(type_command(item_type))
            })
    }

    fn augment_subcommands_for_update(command: Command) -> Command {
        Self::augment_subcommands(command)
    }

    fn has_subcommand(name: &str) -> bool {
        ItemType::all()
            .into_iter()
            .any(|t| subcommand_name(t) == name || t.prefix().to_lowercase() == name)
    }
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
        Some(subcommand) => run_with_options(subcommand.to_init_options(), config),
    }
}

fn run_interactive(config: &Config) -> Result<ExitCode, Box<dyn Error>> {
    let mut session = InteractiveSession {
        graph: None,
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
    for (relation, ids) in input.traceability.iter() {
        config = config.relation(relation.as_str(), ids.to_vec());
    }
    config
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

//! Init command implementation.

use std::path::PathBuf;
use std::process::ExitCode;

use sara_core::{InitError, InitOptions, InitResult, InitService, ItemType};

use super::CommandContext;
use crate::output::{OutputConfig, print_error, print_success, print_warning};

/// Exit code for invalid option for item type.
const EXIT_INVALID_OPTION: u8 = 3;

/// Exit code for frontmatter already exists.
const EXIT_FRONTMATTER_EXISTS: u8 = 2;

/// CLI-specific init options that will be converted to core InitOptions.
#[derive(Debug)]
pub struct CliInitOptions {
    pub file: PathBuf,
    pub item_type: ItemType,
    pub id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub refines: Vec<String>,
    pub derives_from: Vec<String>,
    pub satisfies: Vec<String>,
    pub specification: Option<String>,
    pub platform: Option<String>,
    pub force: bool,
}

impl From<CliInitOptions> for InitOptions {
    fn from(cli: CliInitOptions) -> Self {
        let mut opts = InitOptions::new(cli.file, cli.item_type);

        if let Some(id) = cli.id {
            opts = opts.with_id(id);
        }
        if let Some(name) = cli.name {
            opts = opts.with_name(name);
        }
        if let Some(desc) = cli.description {
            opts = opts.with_description(desc);
        }
        if !cli.refines.is_empty() {
            opts = opts.with_refines(cli.refines);
        }
        if !cli.derives_from.is_empty() {
            opts = opts.with_derives_from(cli.derives_from);
        }
        if !cli.satisfies.is_empty() {
            opts = opts.with_satisfies(cli.satisfies);
        }
        if let Some(spec) = cli.specification {
            opts = opts.with_specification(spec);
        }
        if let Some(platform) = cli.platform {
            opts = opts.with_platform(platform);
        }
        opts = opts.with_force(cli.force);

        opts
    }
}

/// Runs the init command.
pub fn run(
    cli_opts: CliInitOptions,
    ctx: &CommandContext,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let config = &ctx.output;
    let opts: InitOptions = cli_opts.into();

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
    use sara_core::parse_item_type;

    #[test]
    fn test_parse_item_type() {
        assert_eq!(parse_item_type("solution"), Some(ItemType::Solution));
        assert_eq!(parse_item_type("SOL"), Some(ItemType::Solution));
        assert_eq!(parse_item_type("use_case"), Some(ItemType::UseCase));
        assert_eq!(parse_item_type("UC"), Some(ItemType::UseCase));
        assert_eq!(parse_item_type("invalid"), None);
    }

    #[test]
    fn test_cli_options_to_init_options() {
        let cli_opts = CliInitOptions {
            file: PathBuf::from("test.md"),
            item_type: ItemType::Solution,
            id: Some("SOL-001".to_string()),
            name: Some("Test".to_string()),
            description: None,
            refines: vec![],
            derives_from: vec![],
            satisfies: vec![],
            specification: None,
            platform: None,
            force: false,
        };

        let opts: InitOptions = cli_opts.into();
        assert_eq!(opts.file, PathBuf::from("test.md"));
        assert_eq!(opts.item_type, ItemType::Solution);
        assert_eq!(opts.id, Some("SOL-001".to_string()));
    }
}

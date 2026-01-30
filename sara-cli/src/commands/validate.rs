//! Implementation of the validate command.

use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Args;

use sara_core::graph::GraphBuilder;
use sara_core::repository::parse_repositories;
use sara_core::validation::{ValidationReport, pre_validate, validate};

use super::CommandContext;
use crate::output::{OutputConfig, print_error, print_error_summary, print_success, print_warning};

/// Output format for validate command.
#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum ValidateFormat {
    #[default]
    Text,
    Json,
}

/// Arguments for the validate command.
#[derive(Args, Debug)]
pub struct ValidateArgs {
    /// Validate at specific Git commit/branch
    #[arg(long, value_name = "GIT_REF", help_heading = "Input")]
    pub at: Option<String>,

    /// Output format
    #[arg(long, default_value = "text", help_heading = "Output")]
    pub format: ValidateFormat,

    /// Write validation report to file
    #[arg(short, long, help_heading = "Output")]
    pub output: Option<PathBuf>,

    /// Treat orphan items as errors (default: warnings)
    #[arg(long, help_heading = "Validation")]
    pub strict: bool,
}

/// Runs the validate command.
pub fn run(args: &ValidateArgs, ctx: &CommandContext) -> Result<ExitCode, Box<dyn Error>> {
    let output_config = &ctx.output;
    let items = parse_repositories(&ctx.repositories)?;

    if items.is_empty() {
        print_warning(output_config, "No items found in repositories");
        return Ok(ExitCode::SUCCESS);
    }

    let pre_report = pre_validate(&items, args.strict);
    if !pre_report.is_valid() {
        return handle_report(&pre_report, args, output_config);
    }

    let graph = GraphBuilder::new()
        .with_strict_mode(args.strict)
        .add_items(items)
        .build()?;

    let mut report = validate(&graph, args.strict);
    report.merge(pre_report);
    handle_report(&report, args, output_config)
}

/// Handles the validation report output and returns the appropriate exit code.
fn handle_report(
    report: &ValidationReport,
    args: &ValidateArgs,
    output_config: &OutputConfig,
) -> Result<ExitCode, Box<dyn Error>> {
    match args.format {
        ValidateFormat::Text => print_text_report(report, output_config),
        ValidateFormat::Json => print_json_report(report, args.output.as_ref())?,
    }

    if report.is_valid() {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::from(1))
    }
}

/// Prints the validation report in text format.
fn print_text_report(report: &ValidationReport, config: &OutputConfig) {
    let output = format!(
        "\nValidation Results\n\
         ==================\n\n\
         Items checked:         {}\n\
         Relationships checked: {}\n",
        report.items_checked, report.relationships_checked
    );

    println!("{}", output);

    if report.error_count() > 0 {
        println!();
        for error in report.errors() {
            print_error(config, &error.to_string());
        }
    }
    if report.warning_count() > 0 {
        println!();
        for warning in report.warnings() {
            print_warning(config, &warning.to_string());
        }
    }

    if report.is_valid() {
        if report.warning_count() > 0 {
            print_success(
                config,
                &format!(
                    "Validation passed with {} warning(s)",
                    report.warning_count()
                ),
            );
        } else {
            print_success(config, "Validation passed");
        }
    } else {
        print_error_summary(
            config,
            &format!(
                "Validation failed with {} error(s) and {} warning(s)",
                report.error_count(),
                report.warning_count()
            ),
        );
    }
}

/// Prints the validation report in JSON format.
fn print_json_report(
    report: &ValidationReport,
    output_path: Option<&PathBuf>,
) -> Result<(), Box<dyn Error>> {
    let json = serde_json::to_string_pretty(&report)?;

    if let Some(path) = output_path {
        fs::write(path, &json)?;
    } else {
        println!("{}", json);
    }

    Ok(())
}

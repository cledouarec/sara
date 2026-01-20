//! Implementation of the validate command.

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use sara_core::graph::GraphBuilder;
use sara_core::repository::parse_repositories;
use sara_core::validation::rules::check_duplicate_items;
use sara_core::validation::{ValidationReport, ValidationReportBuilder, validate, validate_strict};

use super::CommandContext;
use crate::output::{OutputConfig, print_error, print_success, print_warning};

pub use super::OutputFormat;

/// Options for the validate command.
pub struct ValidateOptions {
    /// Enable strict orphan checking.
    pub strict: bool,
    /// Output format.
    pub format: OutputFormat,
    /// Output file path (optional).
    pub output: Option<PathBuf>,
}

/// Runs the validate command.
pub fn run(
    opts: ValidateOptions,
    ctx: &CommandContext,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let output_config = &ctx.output;

    // Parse repositories
    let items = parse_repositories(&ctx.repositories)?;

    if items.is_empty() {
        print_warning(output_config, "No items found in repositories");
        return Ok(ExitCode::SUCCESS);
    }

    // Check for duplicates before building the graph
    let duplicate_errors = check_duplicate_items(&items);
    if !duplicate_errors.is_empty() {
        // Build report with duplicate errors
        let report = ValidationReportBuilder::new()
            .items_checked(items.len())
            .errors(duplicate_errors)
            .build();

        // Output and return error
        match opts.format {
            OutputFormat::Text => print_text_report(&report, output_config),
            OutputFormat::Json => print_json_report(&report, opts.output.as_ref())?,
        }
        return Ok(ExitCode::from(1));
    }

    // Build the graph
    let graph = GraphBuilder::new()
        .with_strict_mode(opts.strict)
        .add_items(items)
        .build()?;

    // Validate
    let report = if opts.strict {
        validate_strict(&graph)
    } else {
        validate(&graph)
    };

    // Output results
    match opts.format {
        OutputFormat::Text => print_text_report(&report, output_config),
        OutputFormat::Json => print_json_report(&report, opts.output.as_ref())?,
    }

    // Return appropriate exit code
    if report.is_valid() {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::from(1))
    }
}

/// Prints the validation report in text format.
fn print_text_report(report: &ValidationReport, config: &OutputConfig) {
    // Build header
    let mut output = format!(
        "\nValidation Results\n\
         ==================\n\n\
         Items checked:         {}\n\
         Relationships checked: {}\n",
        report.items_checked, report.relationships_checked
    );

    // Add errors section
    if report.error_count() > 0 {
        output.push_str(&format!(
            "\nErrors ({}):\n----------\n",
            report.error_count()
        ));
        for error in report.errors() {
            output.push_str(&format!("  {}\n", error));
        }
    }

    // Add warnings section
    if report.warning_count() > 0 {
        output.push_str(&format!(
            "\nWarnings ({}):\n-----------\n",
            report.warning_count()
        ));
        for warning in report.warnings() {
            output.push_str(&format!("  {}\n", warning));
        }
    }

    println!("{}", output);

    // Print summary with colors/emojis
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
        print_error(
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
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(&report)?;

    if let Some(path) = output_path {
        fs::write(path, &json)?;
    } else {
        println!("{}", json);
    }

    Ok(())
}

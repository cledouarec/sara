//! Implementation of the check command.

use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{Duration, Instant};

use clap::Args;
use serde::Serialize;

use sara_core::graph::{KnowledgeGraph, KnowledgeGraphBuilder};
use sara_core::model::ItemType;
use sara_core::validation::{ValidationReport, pre_validate, validate};

use super::CommandContext;
use crate::output::{OutputConfig, format_error, format_success, format_warning, print_warning};

/// Output format for check command.
#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum CheckFormat {
    #[default]
    Text,
    Json,
}

/// Arguments for the check command.
#[derive(Args, Debug)]
pub struct CheckArgs {
    /// Read from specific Git commit/branch
    #[arg(long, value_name = "GIT_REF", help_heading = "Input")]
    pub at: Option<String>,

    /// Output format
    #[arg(long, default_value = "text", help_heading = "Output")]
    pub format: CheckFormat,

    /// Write output to file
    #[arg(short, long, help_heading = "Output")]
    pub output: Option<PathBuf>,

    /// Treat orphan items as errors (default: warnings)
    #[arg(long, help_heading = "Validation")]
    pub strict: bool,
}

/// Unified JSON result containing validation results and optionally the graph.
///
/// This structure provides a consistent JSON output format that includes
/// validation errors/warnings and the parsed graph when validation succeeds.
#[derive(Debug, Serialize)]
struct CheckResult {
    /// Whether the check passed without errors.
    valid: bool,
    /// Number of items checked.
    items_checked: usize,
    /// Number of relationships checked.
    relationships_checked: usize,
    /// Count of items by type.
    items_by_type: std::collections::HashMap<ItemType, usize>,
    /// Time taken to parse in milliseconds.
    parse_time_ms: u128,
    /// Validation errors encountered.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    errors: Vec<String>,
    /// Validation warnings encountered.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    warnings: Vec<String>,
    /// The parsed graph (present only when validation passed).
    #[serde(skip_serializing_if = "Option::is_none")]
    graph: Option<GraphExport>,
}

/// Serializable representation of the parsed graph for JSON export.
#[derive(Debug, Serialize)]
struct GraphExport {
    items: Vec<ItemExport>,
    relationships: Vec<RelationshipExport>,
}

/// Serializable item for JSON export.
#[derive(Debug, Serialize)]
struct ItemExport {
    id: String,
    item_type: String,
    name: String,
    description: Option<String>,
    specification: Option<String>,
    file: String,
}

/// Serializable relationship for JSON export.
#[derive(Debug, Serialize)]
struct RelationshipExport {
    from: String,
    to: String,
    relationship_type: String,
}

/// Runs the check command.
pub fn run(args: &CheckArgs, ctx: &CommandContext) -> Result<ExitCode, Box<dyn Error>> {
    let start = Instant::now();
    let output_config = &ctx.output;

    let items = ctx.parse_items(args.at.as_deref())?;

    if items.is_empty() {
        print_warning(output_config, "No items found in repositories");
        return Ok(ExitCode::SUCCESS);
    }

    let pre_report = pre_validate(&items, args.strict);
    if !pre_report.is_valid() {
        let parse_time = start.elapsed();
        return handle_output(args, None, &pre_report, &parse_time, output_config);
    }

    let graph = KnowledgeGraphBuilder::new().add_items(items).build()?;

    let report = validate(&graph, args.strict);
    let report = consolidate_reports(report, pre_report);
    let parse_time = start.elapsed();
    handle_output(args, Some(&graph), &report, &parse_time, output_config)
}

/// Consolidates two validation reports, keeping all data from the main report
/// and prepending issues from the pre-validation report.
fn consolidate_reports(report: ValidationReport, pre_report: ValidationReport) -> ValidationReport {
    ValidationReport {
        issues: [pre_report.issues, report.issues].concat(),
        items_checked: report.items_checked,
        relationships_checked: report.relationships_checked,
        items_by_type: report.items_by_type,
    }
}

/// Handles output based on format and validation results.
fn handle_output(
    args: &CheckArgs,
    graph: Option<&KnowledgeGraph>,
    report: &ValidationReport,
    parse_time: &Duration,
    output_config: &OutputConfig,
) -> Result<ExitCode, Box<dyn Error>> {
    let output = match args.format {
        CheckFormat::Text => build_text_output(report, parse_time, output_config),
        CheckFormat::Json => {
            let result = build_check_result(graph, report, parse_time);
            serde_json::to_string_pretty(&result)?
        }
    };

    write_output(&output, args.output.as_ref())?;

    if report.is_valid() {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::from(1))
    }
}

/// Builds a unified check result from the graph and validation report.
fn build_check_result(
    graph: Option<&KnowledgeGraph>,
    report: &ValidationReport,
    parse_time: &Duration,
) -> CheckResult {
    let errors: Vec<String> = report.errors().iter().map(|e| e.to_string()).collect();
    let warnings: Vec<String> = report.warnings().iter().map(|w| w.to_string()).collect();

    let graph_export = graph.filter(|_| report.is_valid()).map(build_graph_export);

    CheckResult {
        valid: report.is_valid(),
        items_checked: report.items_checked,
        relationships_checked: report.relationships_checked,
        items_by_type: report.items_by_type.clone(),
        parse_time_ms: parse_time.as_millis(),
        errors,
        warnings,
        graph: graph_export,
    }
}

/// Builds a graph export from the knowledge graph.
fn build_graph_export(graph: &KnowledgeGraph) -> GraphExport {
    GraphExport {
        items: graph
            .items()
            .map(|item| ItemExport {
                id: item.id.as_str().to_string(),
                item_type: item.item_type.to_string(),
                name: item.name.clone(),
                description: None,
                specification: item.attributes.specification().map(ToOwned::to_owned),
                file: item.source.file_path.display().to_string(),
            })
            .collect(),
        relationships: graph
            .relationships()
            .into_iter()
            .map(|(from, to, rel_type)| RelationshipExport {
                from: from.as_str().to_string(),
                to: to.as_str().to_string(),
                relationship_type: rel_type.to_string(),
            })
            .collect(),
    }
}

/// Writes output to stdout or a file.
fn write_output(content: &str, output_path: Option<&PathBuf>) -> Result<(), Box<dyn Error>> {
    match output_path {
        Some(path) => {
            fs::write(path, content)?;
        }
        None => {
            println!("{content}");
        }
    }

    Ok(())
}

/// Builds the check output in text format.
fn build_text_output(
    report: &ValidationReport,
    parse_time: &Duration,
    config: &OutputConfig,
) -> String {
    let mut output = String::new();

    let types_section = if report.items_by_type.is_empty() {
        String::new()
    } else {
        let type_lines: Vec<_> = ItemType::all()
            .iter()
            .filter_map(|item_type| {
                report
                    .items_by_type
                    .get(item_type)
                    .map(|count| format!("  {:35} {}", item_type.display_name(), count))
            })
            .collect();
        format!("\nItems by type:\n{}\n", type_lines.join("\n"))
    };

    output.push_str(&format!(
        "\n\
         Check Results\n\
         =============\n\n\
         Items:                 {}\n\
         Relationships:         {}\n\
         Parse time:            {}ms\
         {}",
        report.items_checked,
        report.relationships_checked,
        parse_time.as_millis(),
        types_section
    ));

    if report.error_count() > 0 {
        output.push('\n');
        for error in report.errors() {
            output.push_str(&format_error(config, &error.to_string()));
            output.push('\n');
        }
    }
    if report.warning_count() > 0 {
        output.push('\n');
        for warning in report.warnings() {
            output.push_str(&format_warning(config, &warning.to_string()));
            output.push('\n');
        }
    }

    output.push('\n');
    if report.is_valid() {
        if report.warning_count() > 0 {
            output.push_str(&format_success(
                config,
                &format!("Check passed with {} warning(s)", report.warning_count()),
            ));
        } else {
            output.push_str(&format_success(config, "Check passed"));
        }
    } else {
        output.push_str(&format_error(
            config,
            &format!(
                "Check failed with {} error(s) and {} warning(s)",
                report.error_count(),
                report.warning_count()
            ),
        ));
    }

    output
}

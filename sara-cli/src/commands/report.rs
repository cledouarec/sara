//! Report command implementation.

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

use sara_core::{CoverageReport, GraphBuilder, TraceabilityMatrix, parse_repositories};

use super::CommandContext;
use crate::output::{
    Color, EMOJI_STATS, EMOJI_WARNING, OutputConfig, Style, colorize, get_emoji, print_success,
};

pub use super::ReportFormat as OutputFormat;

/// Coverage report options.
#[derive(Debug)]
pub struct CoverageOptions {
    pub format: OutputFormat,
    pub output: Option<PathBuf>,
}

/// Matrix report options.
#[derive(Debug)]
pub struct MatrixOptions {
    pub format: OutputFormat,
    pub output: Option<PathBuf>,
}

/// Runs the coverage report command.
pub fn run_coverage(
    opts: CoverageOptions,
    ctx: &CommandContext,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let config = &ctx.output;

    // Parse repositories
    let items = parse_repositories(&ctx.repositories)?;

    // Build the graph
    let graph = GraphBuilder::new().add_items(items).build()?;

    // Generate report
    let report = CoverageReport::generate(&graph);

    // Format output
    let output = match opts.format {
        OutputFormat::Text => format_coverage_text(&report, config),
        OutputFormat::Json => format_coverage_json(&report),
        OutputFormat::Csv => format_coverage_csv(&report),
        OutputFormat::Html => format_coverage_html(&report),
    };

    // Write to file or stdout
    if let Some(path) = opts.output {
        let mut file = File::create(&path)?;
        file.write_all(output.as_bytes())?;
        print_success(
            config,
            &format!("Coverage report exported to {}", path.display()),
        );
    } else {
        println!("{}", output);
    }

    Ok(ExitCode::SUCCESS)
}

/// Runs the matrix report command.
pub fn run_matrix(
    opts: MatrixOptions,
    ctx: &CommandContext,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let config = &ctx.output;

    // Parse repositories
    let items = parse_repositories(&ctx.repositories)?;

    // Build the graph
    let graph = GraphBuilder::new().add_items(items).build()?;

    // Generate matrix
    let matrix = TraceabilityMatrix::generate(&graph);

    // Format output
    let output = match opts.format {
        OutputFormat::Text => format_matrix_text(&matrix, config),
        OutputFormat::Json => format_matrix_json(&matrix),
        OutputFormat::Csv => matrix.to_csv(),
        OutputFormat::Html => format_matrix_html(&matrix),
    };

    // Write to file or stdout
    if let Some(path) = opts.output {
        let mut file = File::create(&path)?;
        file.write_all(output.as_bytes())?;
        print_success(
            config,
            &format!("Traceability matrix exported to {}", path.display()),
        );
    } else {
        println!("{}", output);
    }

    Ok(ExitCode::SUCCESS)
}

fn format_coverage_text(report: &CoverageReport, config: &OutputConfig) -> String {
    let emoji = get_emoji(config, &EMOJI_STATS);
    let warning_emoji = get_emoji(config, &EMOJI_WARNING);

    let type_rows: String = report
        .by_type
        .iter()
        .map(|tc| {
            format!(
                "  {:<25} {:>5}   {:>8}   {:>7.1}%",
                tc.type_name, tc.total, tc.complete, tc.coverage_percent
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let incomplete_section = if report.incomplete_items.is_empty() {
        String::new()
    } else {
        let items: String = report
            .incomplete_items
            .iter()
            .map(|item| {
                let line = format!("{}: {}", item.id, item.reason);
                let formatted = colorize(config, &line, Color::Yellow, Style::None);
                format!("  {warning_emoji} {formatted}")
            })
            .collect::<Vec<_>>()
            .join("\n");
        format!("\nIncomplete Items:\n{items}")
    };

    format!(
        "{emoji} Traceability Coverage Report

Overall Coverage: {:.1}%

By Item Type:
  Type                      Items   Complete   Coverage
  ─────────────────────────────────────────────────────
{type_rows}
{incomplete_section}
",
        report.overall_coverage
    )
}

fn format_coverage_json(report: &CoverageReport) -> String {
    serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".to_string())
}

fn format_coverage_csv(report: &CoverageReport) -> String {
    let rows: String = report
        .by_type
        .iter()
        .map(|tc| {
            format!(
                "{},{},{},{},{:.1}",
                tc.type_name, tc.total, tc.complete, tc.incomplete, tc.coverage_percent
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!("Type,Total,Complete,Incomplete,Coverage %\n{rows}\n")
}

fn format_coverage_html(report: &CoverageReport) -> String {
    let type_rows: String = report
        .by_type
        .iter()
        .map(|tc| {
            format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{:.1}%</td></tr>",
                tc.type_name, tc.total, tc.complete, tc.incomplete, tc.coverage_percent
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let incomplete_section = if report.incomplete_items.is_empty() {
        String::new()
    } else {
        let items: String = report
            .incomplete_items
            .iter()
            .map(|item| format!("<li><strong>{}</strong>: {}</li>", item.id, item.reason))
            .collect::<Vec<_>>()
            .join("\n");
        format!("<h2>Incomplete Items</h2>\n<ul>\n{items}\n</ul>")
    };

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
<title>Coverage Report</title>
<style>
body {{ font-family: system-ui, sans-serif; margin: 2rem; }}
table {{ border-collapse: collapse; width: 100%; }}
th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
th {{ background-color: #f4f4f4; }}
.complete {{ color: green; }}
.incomplete {{ color: red; }}
</style>
</head>
<body>
<h1>Traceability Coverage Report</h1>
<p><strong>Overall Coverage:</strong> {:.1}%</p>
<h2>Coverage by Type</h2>
<table>
<thead>
<tr><th>Type</th><th>Total</th><th>Complete</th><th>Incomplete</th><th>Coverage</th></tr>
</thead>
<tbody>
{type_rows}
</tbody>
</table>
{incomplete_section}
</body>
</html>
"#,
        report.overall_coverage
    )
}

fn format_matrix_text(matrix: &TraceabilityMatrix, _config: &OutputConfig) -> String {
    let rows: String = matrix
        .rows
        .iter()
        .map(|row| {
            let header = format!("{} ({})", row.source_id, row.source_type);
            let targets: String = row
                .targets
                .iter()
                .map(|t| {
                    format!(
                        "  └─ {} {} ({}) [{}]",
                        t.relationship, t.id, t.target_type, t.name
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            if targets.is_empty() {
                header
            } else {
                format!("{header}\n{targets}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "Traceability Matrix\nTotal Relationships: {}\n\n{rows}\n",
        matrix.total_relationships
    )
}

fn format_matrix_json(matrix: &TraceabilityMatrix) -> String {
    serde_json::to_string_pretty(matrix).unwrap_or_else(|_| "{}".to_string())
}

fn format_matrix_html(matrix: &TraceabilityMatrix) -> String {
    let rows: String = matrix
        .rows
        .iter()
        .flat_map(|row| {
            if row.targets.is_empty() {
                vec![format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td><td>-</td><td>-</td><td>-</td></tr>",
                    row.source_id, row.source_name, row.source_type
                )]
            } else {
                row.targets
                    .iter()
                    .map(|t| {
                        format!(
                            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                            row.source_id, row.source_name, row.source_type, t.id, t.name, t.relationship
                        )
                    })
                    .collect()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
<title>Traceability Matrix</title>
<style>
body {{ font-family: system-ui, sans-serif; margin: 2rem; }}
table {{ border-collapse: collapse; width: 100%; }}
th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
th {{ background-color: #f4f4f4; }}
</style>
</head>
<body>
<h1>Traceability Matrix</h1>
<p><strong>Total Relationships:</strong> {}</p>
<table>
<thead>
<tr><th>Source ID</th><th>Source Name</th><th>Source Type</th><th>Target ID</th><th>Target Name</th><th>Relationship</th></tr>
</thead>
<tbody>
{rows}
</tbody>
</table>
</body>
</html>
"#,
        matrix.total_relationships
    )
}

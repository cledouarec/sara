//! Implementation of the parse command.

use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use clap::Args;
use serde::Serialize;

use sara_core::graph::{GraphBuilder, GraphStats, KnowledgeGraph};
use sara_core::model::Item;
use sara_core::repository::{GitReader, GitRef, parse_repositories};
use sara_core::validation::{rules::check_duplicate_items, validate};

use super::CommandContext;
use crate::output::{OutputConfig, print_error, print_success, print_warning};

/// Arguments for the parse command.
#[derive(Args, Debug)]
pub struct ParseArgs {
    /// Read from specific Git commit/branch
    #[arg(long, value_name = "GIT_REF", help_heading = "Input")]
    pub at: Option<String>,

    /// Output parsed graph to file (JSON format)
    #[arg(short, long, help_heading = "Output")]
    pub output: Option<PathBuf>,
}

/// CLI-specific parse output including timing information.
#[derive(Debug, Serialize)]
struct ParseOutput {
    #[serde(flatten)]
    stats: GraphStats,
    parse_time_ms: u128,
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

/// Runs the parse command.
pub fn run(args: &ParseArgs, ctx: &CommandContext) -> Result<ExitCode, Box<dyn Error>> {
    let start = Instant::now();
    let output_config = &ctx.output;

    let repos = collect_repositories(ctx)?;
    let items = scan_repositories(&repos, args)?;

    if items.is_empty() {
        print_warning(output_config, "No items found in repositories");
        return Ok(ExitCode::SUCCESS);
    }

    if let Some(exit_code) = check_for_duplicates(&items, output_config) {
        return Ok(exit_code);
    }

    let graph = build_graph(items.clone())?;
    let output = ParseOutput {
        stats: GraphStats::from_graph(&graph),
        parse_time_ms: start.elapsed().as_millis(),
    };

    handle_output(args, &graph, &output, output_config)?;
    warn_validation_errors(&graph, output_config);

    Ok(ExitCode::SUCCESS)
}

/// Collects repository paths, defaulting to current directory if none specified.
fn collect_repositories(ctx: &CommandContext) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    if ctx.repositories.is_empty() {
        Ok(vec![std::env::current_dir()?])
    } else {
        Ok(ctx.repositories.clone())
    }
}

/// Scans repositories and parses items.
fn scan_repositories(repos: &[PathBuf], args: &ParseArgs) -> Result<Vec<Item>, Box<dyn Error>> {
    if let Some(ref git_ref) = args.at {
        parse_from_git(repos, git_ref)
    } else {
        Ok(parse_repositories(repos)?)
    }
}

/// Checks for duplicate items and returns an exit code if duplicates are found.
fn check_for_duplicates(items: &[Item], output_config: &OutputConfig) -> Option<ExitCode> {
    let duplicate_errors = check_duplicate_items(items);
    if duplicate_errors.is_empty() {
        return None;
    }

    for error in &duplicate_errors {
        print_error(output_config, &format!("{}", error));
    }
    Some(ExitCode::from(1))
}

/// Builds the knowledge graph.
fn build_graph(items: Vec<Item>) -> Result<KnowledgeGraph, Box<dyn Error>> {
    Ok(GraphBuilder::new().add_items(items).build()?)
}

/// Handles output: either exports to JSON or prints stats.
fn handle_output(
    args: &ParseArgs,
    graph: &KnowledgeGraph,
    output: &ParseOutput,
    output_config: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    if let Some(ref output_path) = args.output {
        export_graph_to_json(graph, output_path, output_config)?;
    } else {
        print_output(output, output_config);
    }
    Ok(())
}

/// Exports the graph to a JSON file.
fn export_graph_to_json(
    graph: &KnowledgeGraph,
    output_path: &PathBuf,
    output_config: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    let export = GraphExport {
        items: graph
            .items()
            .map(|item| ItemExport {
                id: item.id.as_str().to_string(),
                item_type: item.item_type.to_string(),
                name: item.name.clone(),
                description: None,
                specification: item.attributes.specification.clone(),
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
    };

    let json = serde_json::to_string_pretty(&export)?;
    fs::write(output_path, &json)?;
    print_success(
        output_config,
        &format!("Graph exported to {}", output_path.display()),
    );
    Ok(())
}

/// Warns about validation errors if any are found.
fn warn_validation_errors(graph: &KnowledgeGraph, output_config: &OutputConfig) {
    let report = validate(graph);
    if report.error_count() > 0 {
        println!();
        print_warning(
            output_config,
            &format!(
                "Found {} validation error(s). Run 'sara validate' for details.",
                report.error_count()
            ),
        );
    }
}

/// Parses items from a specific Git reference.
fn parse_from_git(repos: &[PathBuf], git_ref_str: &str) -> Result<Vec<Item>, Box<dyn Error>> {
    let git_ref = GitRef::parse(git_ref_str);
    let mut all_items = Vec::new();

    for repo_path in repos {
        if !repo_path.exists() {
            tracing::warn!("Repository path does not exist: {}", repo_path.display());
            continue;
        }

        let reader = GitReader::open(repo_path)?;
        let items = reader.parse_commit(&git_ref)?;
        all_items.extend(items);
    }

    Ok(all_items)
}

/// Prints parse output to the console.
fn print_output(output: &ParseOutput, config: &OutputConfig) {
    let types_section = if output.stats.items_by_type.is_empty() {
        String::new()
    } else {
        let mut types: Vec<_> = output.stats.items_by_type.iter().collect();
        types.sort_by(|a, b| b.1.cmp(a.1));
        let type_lines: Vec<_> = types
            .iter()
            .map(|(item_type, count)| format!("  {:25} {}", item_type.display_name(), count))
            .collect();
        format!("Items by type:\n{}\n", type_lines.join("\n"))
    };

    println!(
        "\n\
         Parse Results\n\
         =============\n\n\
         Items parsed:          {}\n\
         Relationships found:   {}\n\
         Parse time:            {}ms\n\n\
         {}",
        output.stats.item_count,
        output.stats.relationship_count,
        output.parse_time_ms,
        types_section
    );

    print_success(config, "Parse completed successfully");
}

//! Diff command implementation.

use std::error::Error;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Args;

use sara_core::diff::{DiffOptions, DiffResult, DiffService};
use sara_core::graph::{DiffStats, GraphDiff, ItemDiff, ItemModification, RelationshipDiff};

use super::CommandContext;
use crate::output::{
    Color, OutputConfig, Style, colorize, print_error, print_success, print_warning,
};

/// Output format for diff command.
#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum DiffFormat {
    #[default]
    Text,
    Json,
}

/// Arguments for the diff command.
#[derive(Args, Debug)]
pub struct DiffArgs {
    /// First Git reference
    pub ref1: String,

    /// Second Git reference
    pub ref2: String,

    /// Output format
    #[arg(long, default_value = "text", help_heading = "Output")]
    pub format: DiffFormat,

    /// Show summary statistics only
    #[arg(long, help_heading = "Output")]
    pub stat: bool,
}

/// Runs the diff command.
pub fn run(
    args: &DiffArgs,
    repositories: Vec<PathBuf>,
    ctx: &CommandContext,
) -> Result<ExitCode, Box<dyn Error>> {
    let opts = DiffOptions::new(&args.ref1, &args.ref2)
        .with_repositories(repositories)
        .with_stat(args.stat);
    let service = DiffService::new();

    match service.diff(&opts) {
        Ok(result) => {
            if !result.is_full_comparison {
                print_warning(
                    &ctx.output,
                    "Git reference comparison is not fully implemented. Comparing current state with itself.",
                );
            }

            match args.format {
                DiffFormat::Text => print_diff_text(&result, &opts, &ctx.output),
                DiffFormat::Json => print_diff_json(&result.diff),
            }

            if result.is_empty() {
                print_success(&ctx.output, "No changes detected");
            }

            Ok(ExitCode::SUCCESS)
        }
        Err(e) => {
            print_error(&ctx.output, &e.to_string());
            Ok(ExitCode::FAILURE)
        }
    }
}

fn print_diff_text(result: &DiffResult, opts: &DiffOptions, config: &OutputConfig) {
    print_diff_header(result, config);

    if opts.stat {
        print_diff_stats(&result.diff.stats, config);
        return;
    }

    let diff = &result.diff;

    print_item_section(&diff.added_items, "Added Items:", "+", Color::Green, config);
    print_item_section(
        &diff.removed_items,
        "Removed Items:",
        "-",
        Color::Red,
        config,
    );
    print_modified_items(&diff.modified_items, config);
    print_relationship_section(
        &diff.added_relationships,
        "Added Relationships:",
        "+",
        Color::Green,
        config,
    );
    print_relationship_section(
        &diff.removed_relationships,
        "Removed Relationships:",
        "-",
        Color::Red,
        config,
    );

    print_diff_stats(&diff.stats, config);
}

fn print_diff_header(result: &DiffResult, config: &OutputConfig) {
    let ref1 = colorize(config, &result.ref1, Color::Yellow, Style::None);
    let ref2 = colorize(config, &result.ref2, Color::Green, Style::None);
    println!("Comparing {} → {}", ref1, ref2);
    println!();
}

fn print_item_section(
    items: &[ItemDiff],
    title: &str,
    symbol: &str,
    color: Color,
    config: &OutputConfig,
) {
    if items.is_empty() {
        return;
    }

    println!("{}", colorize(config, title, color, Style::Bold));
    for item in items {
        let sym = colorize(config, symbol, color, Style::None);
        let id = colorize(config, &item.id, color, Style::None);
        println!("  {} {} ({})", sym, id, item.item_type);
    }
    println!();
}

fn print_modified_items(items: &[ItemModification], config: &OutputConfig) {
    if items.is_empty() {
        return;
    }

    println!(
        "{}",
        colorize(config, "Modified Items:", Color::Yellow, Style::Bold)
    );
    for item in items {
        let tilde = colorize(config, "~", Color::Yellow, Style::None);
        let id = colorize(config, &item.id, Color::Yellow, Style::None);
        println!("  {} {} ({})", tilde, id, item.item_type);
        for change in &item.changes {
            let old = colorize(config, &change.old_value, Color::None, Style::Dimmed);
            println!("    {}: {} → {}", change.field, old, change.new_value);
        }
    }
    println!();
}

fn print_relationship_section(
    relationships: &[RelationshipDiff],
    title: &str,
    symbol: &str,
    color: Color,
    config: &OutputConfig,
) {
    if relationships.is_empty() {
        return;
    }

    println!("{}", colorize(config, title, color, Style::Bold));
    for rel in relationships {
        println!(
            "  {} {} {} {}",
            symbol, rel.from_id, rel.relationship_type, rel.to_id
        );
    }
    println!();
}

fn print_diff_stats(stats: &DiffStats, _config: &OutputConfig) {
    println!("Summary:");
    println!(
        "  Items:         +{} -{} ~{}",
        stats.items_added, stats.items_removed, stats.items_modified
    );
    println!(
        "  Relationships: +{} -{}",
        stats.relationships_added, stats.relationships_removed
    );
}

fn print_diff_json(diff: &GraphDiff) {
    let json = serde_json::to_string_pretty(diff).unwrap_or_else(|_| "{}".to_string());
    println!("{}", json);
}

//! Query command implementation.

use std::collections::HashMap;
use std::error::Error;
use std::process::ExitCode;

use clap::Args;
use sara_core::graph::{
    KnowledgeGraph, LookupResult, TraversalOptions, TraversalResult, traverse_downstream,
    traverse_upstream,
};
use sara_core::model::{Item, ItemId, ItemType};

use sara_core::config::{Config, OutputConfig};

use crate::output::{
    Color, EMOJI_ERROR, EMOJI_ITEM, Style, colorize, format_tree_branch, get_emoji, print_header,
};

/// Output format for queries.
#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum QueryFormat {
    #[default]
    Tree,
    Json,
}

/// Arguments for the query command.
#[derive(Args, Debug)]
pub struct QueryArgs {
    /// The item identifier to query
    pub item_id: String,

    /// Limit traversal depth
    #[arg(long, help_heading = "Filters")]
    pub depth: Option<usize>,

    /// Filter by item type(s)
    #[arg(short = 't', long = "type", help_heading = "Filters")]
    pub item_types: Vec<String>,

    /// Show downstream chain (toward Detailed Designs)
    #[arg(short, long, help_heading = "Traversal")]
    pub downstream: bool,

    /// Show upstream chain (toward Solution)
    #[arg(short, long, help_heading = "Traversal")]
    pub upstream: bool,

    /// Output format
    #[arg(long, default_value = "tree", help_heading = "Output")]
    pub format: QueryFormat,
}

/// Runs the query command.
pub fn run(args: &QueryArgs, config: &Config) -> Result<ExitCode, Box<dyn Error>> {
    let graph = super::build_graph(config)?;

    match graph.lookup(&args.item_id) {
        LookupResult::Found(item) => handle_found_item(args, &config.output, item, &graph),
        LookupResult::NotFound { suggestions } => {
            handle_not_found(args, &config.output, &suggestions)
        }
    }
}

/// Handles the case when an item is found.
fn handle_found_item(
    args: &QueryArgs,
    config: &OutputConfig,
    item: &Item,
    graph: &KnowledgeGraph,
) -> Result<ExitCode, Box<dyn Error>> {
    print_item_info(config, item, graph);

    if args.upstream || args.downstream {
        print_traceability(args, config, item, graph);
    } else {
        print_direct_relationships(config, item, graph);
    }

    Ok(ExitCode::SUCCESS)
}

/// Prints upstream and/or downstream traceability for an item.
fn print_traceability(
    args: &QueryArgs,
    config: &OutputConfig,
    item: &Item,
    graph: &KnowledgeGraph,
) {
    let traversal_opts = build_traversal_options(args);

    if args.upstream {
        println!();
        print_header(config, &format!("Upstream Traceability for {}", item.id));
        if let Some(result) = traverse_upstream(graph, &item.id, &traversal_opts) {
            print_traversal(config, &result, graph, args);
        }
    }

    if args.downstream {
        println!();
        print_header(config, &format!("Downstream from {}", item.id));
        if let Some(result) = traverse_downstream(graph, &item.id, &traversal_opts) {
            print_traversal(config, &result, graph, args);
        }
    }
}

/// Handles the case when an item is not found.
fn handle_not_found(
    args: &QueryArgs,
    config: &OutputConfig,
    suggestions: &[&ItemId],
) -> Result<ExitCode, Box<dyn Error>> {
    let emoji = get_emoji(config, &EMOJI_ERROR);
    let id = colorize(config, &args.item_id, Color::Red, Style::None);
    println!("{} Item not found: {}", emoji, id);

    if !suggestions.is_empty() {
        println!();
        println!("Did you mean?");
        for suggestion in suggestions {
            println!("  • {}", suggestion.as_str());
        }
    }

    Ok(ExitCode::FAILURE)
}

fn build_traversal_options(args: &QueryArgs) -> TraversalOptions {
    let mut traversal_opts = TraversalOptions::new();

    if let Some(depth) = args.depth {
        traversal_opts = traversal_opts.with_max_depth(depth);
    }

    let types = parse_item_types(&args.item_types);
    if !types.is_empty() {
        traversal_opts = traversal_opts.with_types(types);
    }

    traversal_opts
}

fn print_item_info(config: &OutputConfig, item: &Item, _graph: &KnowledgeGraph) {
    let emoji = get_emoji(config, &EMOJI_ITEM);
    let id = colorize(config, item.id.as_str(), Color::Cyan, Style::Bold);
    let item_type = colorize(
        config,
        item.item_type.display_name(),
        Color::None,
        Style::Dimmed,
    );
    let desc = item
        .description
        .as_ref()
        .map(|d| format!("\n   Description: {d}"))
        .unwrap_or_default();

    println!(
        "{emoji} {id}: {name}
   Type: {item_type}
   File: {file}{desc}",
        name = item.name,
        file = item.source.file_path.display(),
    );
}

fn print_direct_relationships(config: &OutputConfig, item: &Item, graph: &KnowledgeGraph) {
    for (rel_type, related) in graph.direct_relationships(&item.id) {
        let label = colorize(
            config,
            &format!("{}:", rel_type.display_name()),
            Color::None,
            Style::Bold,
        );
        println!("\n   {label}");
        for (i, related_item) in related.iter().enumerate() {
            let branch = format_tree_branch(i == related.len() - 1);
            let id = colorize(config, related_item.id.as_str(), Color::Cyan, Style::None);
            println!("     {branch} {id}: {name}", name = related_item.name);
        }
    }
}

fn print_traversal(
    config: &OutputConfig,
    result: &TraversalResult,
    graph: &KnowledgeGraph,
    args: &QueryArgs,
) {
    match args.format {
        QueryFormat::Tree => print_traversal_tree(config, result, graph),
        QueryFormat::Json => print_traversal_json(result, graph),
    }
}

fn print_traversal_tree(config: &OutputConfig, result: &TraversalResult, graph: &KnowledgeGraph) {
    // Items come in depth-first preorder, so the parent of an occurrence is
    // the nearest preceding occurrence of its parent id.
    let mut children: Vec<Vec<usize>> = vec![Vec::new(); result.items.len()];
    let mut roots: Vec<usize> = Vec::new();
    let mut last_seen: HashMap<&ItemId, usize> = HashMap::new();

    for (index, node) in result.items.iter().enumerate() {
        match node
            .parent
            .as_ref()
            .and_then(|parent| last_seen.get(parent))
        {
            Some(&parent) => children[parent].push(index),
            None => roots.push(index),
        }
        last_seen.insert(&node.item_id, index);
    }

    let printer = TreePrinter {
        config,
        result,
        graph,
        children,
    };
    for (i, &root) in roots.iter().enumerate() {
        printer.print_node(root, "", i == roots.len() - 1, true);
    }
}

/// Renders a traversal result as an indented tree.
struct TreePrinter<'a> {
    config: &'a OutputConfig,
    result: &'a TraversalResult,
    graph: &'a KnowledgeGraph,
    /// Child occurrences of each occurrence, indexed like `result.items`.
    children: Vec<Vec<usize>>,
}

impl TreePrinter<'_> {
    fn print_node(&self, index: usize, prefix: &str, is_last: bool, is_root: bool) {
        let Some(item) = self.graph.get(&self.result.items[index].item_id) else {
            return;
        };

        // Format the line
        let branch = if is_root {
            ""
        } else if is_last {
            "└── "
        } else {
            "├── "
        };

        let id = colorize(self.config, item.id.as_str(), Color::Cyan, Style::None);
        let type_name = colorize(
            self.config,
            item.item_type.display_name(),
            Color::None,
            Style::Dimmed,
        );
        let item_text = format!("{}: {} ({})", id, item.name, type_name);

        if is_root {
            println!("{}", item_text);
        } else {
            println!("{}{}{}", prefix, branch, item_text);
        }

        // Print children
        let new_prefix = if is_root {
            String::new()
        } else {
            format!("{}{}", prefix, if is_last { "    " } else { "│   " })
        };

        let children = &self.children[index];
        for (i, &child) in children.iter().enumerate() {
            self.print_node(child, &new_prefix, i == children.len() - 1, false);
        }
    }
}

fn print_traversal_json(result: &TraversalResult, graph: &KnowledgeGraph) {
    #[derive(serde::Serialize)]
    struct JsonNode {
        id: String,
        name: String,
        item_type: String,
        depth: usize,
        parent: Option<String>,
    }

    let nodes: Vec<JsonNode> = result
        .items
        .iter()
        .filter_map(|node| {
            graph.get(&node.item_id).map(|item| JsonNode {
                id: item.id.as_str().to_string(),
                name: item.name.clone(),
                item_type: item.item_type.display_name().to_string(),
                depth: node.depth,
                parent: node.parent.as_ref().map(|p| p.as_str().to_string()),
            })
        })
        .collect();

    let json_output = serde_json::json!({
        "origin": result.origin.as_str(),
        "max_depth": result.max_depth,
        "items": nodes
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&json_output).unwrap_or_else(|_| "{}".to_string())
    );
}

/// Parses item type strings into item types known to the active schema.
///
/// Accepts, for every type the schema defines, the schema id (`use_case`),
/// its squashed form (`usecase`) and the type's id prefix in any case
/// (`adr`). Unknown names are ignored.
pub fn parse_item_types(types: &[String]) -> Vec<ItemType> {
    types
        .iter()
        .filter_map(|t| sara_core::service::parse_item_type(t))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_item_types_covers_every_item_type() {
        for item_type in ItemType::all() {
            let parsed = parse_item_types(&[item_type.as_str().to_string()]);
            assert_eq!(
                parsed,
                vec![item_type],
                "type id `{}` must be accepted by --type",
                item_type.as_str()
            );
        }
    }
}

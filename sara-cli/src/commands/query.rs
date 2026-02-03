//! Query command implementation.

use std::error::Error;
use std::process::ExitCode;

use clap::Args;

use sara_core::graph::{
    KnowledgeGraph, KnowledgeGraphBuilder, TraversalNode, TraversalOptions, TraversalResult,
};
use sara_core::model::{Item, ItemId, ItemType};
use sara_core::query::{LookupResult, QueryEngine};

use super::CommandContext;
use crate::output::{
    Color, EMOJI_ERROR, EMOJI_ITEM, OutputConfig, Style, colorize, format_tree_branch, get_emoji,
    print_header,
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
pub fn run(args: &QueryArgs, ctx: &CommandContext) -> Result<ExitCode, Box<dyn Error>> {
    let items = ctx.parse_items(None)?;
    let graph = KnowledgeGraphBuilder::new().add_items(items).build()?;
    let engine = QueryEngine::new(&graph);

    match engine.lookup(&args.item_id) {
        LookupResult::Found(item) => handle_found_item(args, ctx, item, &graph, &engine),
        LookupResult::NotFound { suggestions } => handle_not_found(args, ctx, &suggestions),
    }
}

/// Handles the case when an item is found.
fn handle_found_item(
    args: &QueryArgs,
    ctx: &CommandContext,
    item: &Item,
    graph: &KnowledgeGraph,
    engine: &QueryEngine,
) -> Result<ExitCode, Box<dyn Error>> {
    let config = &ctx.output;

    print_item_info(config, item, graph);

    if args.upstream || args.downstream {
        print_traceability(args, config, item, graph, engine);
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
    engine: &QueryEngine,
) {
    let traversal_opts = build_traversal_options(args);

    if args.upstream {
        println!();
        print_header(config, &format!("Upstream Traceability for {}", item.id));
        if let Some(result) = engine.trace_upstream(&item.id, &traversal_opts) {
            print_traversal(config, &result, graph, args);
        }
    }

    if args.downstream {
        println!();
        print_header(config, &format!("Downstream from {}", item.id));
        if let Some(result) = engine.trace_downstream(&item.id, &traversal_opts) {
            print_traversal(config, &result, graph, args);
        }
    }
}

/// Handles the case when an item is not found.
fn handle_not_found(
    args: &QueryArgs,
    ctx: &CommandContext,
    suggestions: &[&ItemId],
) -> Result<ExitCode, Box<dyn Error>> {
    let config = &ctx.output;
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

    Ok(ExitCode::from(1))
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
    // Print upstream (requires)
    let parents = graph.parents(&item.id);
    if !parents.is_empty() {
        let label = colorize(config, "Requires:", Color::None, Style::Bold);
        println!("\n   {label}");
        for (i, parent) in parents.iter().enumerate() {
            let branch = format_tree_branch(i == parents.len() - 1);
            let id = colorize(config, parent.id.as_str(), Color::Cyan, Style::None);
            println!("     {branch} {id}: {name}", name = parent.name);
        }
    }

    // Print downstream (realized by)
    let children = graph.children(&item.id);
    if !children.is_empty() {
        let label = colorize(config, "Realized by:", Color::None, Style::Bold);
        println!("\n   {label}");
        for (i, child) in children.iter().enumerate() {
            let branch = format_tree_branch(i == children.len() - 1);
            let id = colorize(config, child.id.as_str(), Color::Cyan, Style::None);
            println!("     {branch} {id}: {name}", name = child.name);
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
    // Group items by parent to build tree structure
    let mut children_map: std::collections::HashMap<Option<&ItemId>, Vec<&TraversalNode>> =
        std::collections::HashMap::new();

    for node in &result.items {
        children_map
            .entry(node.parent.as_ref())
            .or_default()
            .push(node);
    }

    // Print the tree starting from items with no parent (roots)
    if let Some(roots) = children_map.get(&None) {
        for (i, root) in roots.iter().enumerate() {
            let is_last = i == roots.len() - 1;
            print_tree_node(config, root, graph, &children_map, "", is_last, true);
        }
    }
}

fn print_tree_node(
    config: &OutputConfig,
    node: &TraversalNode,
    graph: &KnowledgeGraph,
    children_map: &std::collections::HashMap<Option<&ItemId>, Vec<&TraversalNode>>,
    prefix: &str,
    is_last: bool,
    is_root: bool,
) {
    let item = match graph.get(&node.item_id) {
        Some(item) => item,
        None => return,
    };

    // Format the line
    let branch = if is_root {
        ""
    } else if is_last {
        "└── "
    } else {
        "├── "
    };

    let id = colorize(config, item.id.as_str(), Color::Cyan, Style::None);
    let type_name = colorize(
        config,
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
    if let Some(children) = children_map.get(&Some(&node.item_id)) {
        let new_prefix = if is_root {
            String::new()
        } else {
            format!("{}{}", prefix, if is_last { "    " } else { "│   " })
        };

        for (i, child) in children.iter().enumerate() {
            let child_is_last = i == children.len() - 1;
            print_tree_node(
                config,
                child,
                graph,
                children_map,
                &new_prefix,
                child_is_last,
                false,
            );
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

    println!("{}", serde_json::to_string_pretty(&json_output).unwrap());
}

/// Parses item type strings into ItemType enum values.
pub fn parse_item_types(types: &[String]) -> Vec<ItemType> {
    types
        .iter()
        .filter_map(|t| match t.to_lowercase().as_str() {
            "solution" => Some(ItemType::Solution),
            "use_case" | "usecase" => Some(ItemType::UseCase),
            "scenario" => Some(ItemType::Scenario),
            "system_requirement" | "systemrequirement" => Some(ItemType::SystemRequirement),
            "system_architecture" | "systemarchitecture" => Some(ItemType::SystemArchitecture),
            "hardware_requirement" | "hardwarerequirement" => Some(ItemType::HardwareRequirement),
            "software_requirement" | "softwarerequirement" => Some(ItemType::SoftwareRequirement),
            "hardware_detailed_design" | "hardwaredetaileddesign" => {
                Some(ItemType::HardwareDetailedDesign)
            }
            "software_detailed_design" | "softwaredetaileddesign" => {
                Some(ItemType::SoftwareDetailedDesign)
            }
            _ => None,
        })
        .collect()
}

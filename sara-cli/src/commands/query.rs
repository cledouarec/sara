//! Query command implementation.

use std::process::ExitCode;

use sara_core::{
    GraphBuilder, ItemId, ItemType, KnowledgeGraph, LookupResult, QueryEngine, TraversalOptions,
    TraversalResult, parse_repositories,
};

use super::CommandContext;
use crate::output::{
    Color, EMOJI_ERROR, EMOJI_ITEM, OutputConfig, Style, colorize, format_tree_branch, get_emoji,
    print_header,
};

pub use super::QueryFormat as OutputFormat;

/// Query command options.
#[derive(Debug)]
pub struct QueryOptions {
    pub item_id: String,
    pub upstream: bool,
    pub downstream: bool,
    pub types: Vec<ItemType>,
    pub depth: Option<usize>,
    pub format: OutputFormat,
}

/// Runs the query command.
pub fn run(
    opts: QueryOptions,
    ctx: &CommandContext,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let graph = build_graph(ctx)?;
    let engine = QueryEngine::new(&graph);

    match engine.lookup(&opts.item_id) {
        LookupResult::Found(item) => handle_found_item(&opts, ctx, item, &graph, &engine),
        LookupResult::NotFound { suggestions } => handle_not_found(&opts, ctx, &suggestions),
    }
}

/// Builds the knowledge graph from repositories.
fn build_graph(ctx: &CommandContext) -> Result<KnowledgeGraph, Box<dyn std::error::Error>> {
    let items = parse_repositories(&ctx.repositories)?;
    let graph = GraphBuilder::new().add_items(items).build()?;
    Ok(graph)
}

/// Handles the case when an item is found.
fn handle_found_item(
    opts: &QueryOptions,
    ctx: &CommandContext,
    item: &sara_core::Item,
    graph: &KnowledgeGraph,
    engine: &QueryEngine,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let config = &ctx.output;

    print_item_info(config, item, graph);

    if opts.upstream || opts.downstream {
        print_traceability(opts, config, item, graph, engine);
    } else {
        print_direct_relationships(config, item, graph);
    }

    Ok(ExitCode::SUCCESS)
}

/// Prints upstream and/or downstream traceability for an item.
fn print_traceability(
    opts: &QueryOptions,
    config: &OutputConfig,
    item: &sara_core::Item,
    graph: &KnowledgeGraph,
    engine: &QueryEngine,
) {
    let traversal_opts = build_traversal_options(opts);

    if opts.upstream {
        println!();
        print_header(config, &format!("Upstream Traceability for {}", item.id));
        if let Some(result) = engine.trace_upstream(&item.id, &traversal_opts) {
            print_traversal(config, &result, graph, opts);
        }
    }

    if opts.downstream {
        println!();
        print_header(config, &format!("Downstream from {}", item.id));
        if let Some(result) = engine.trace_downstream(&item.id, &traversal_opts) {
            print_traversal(config, &result, graph, opts);
        }
    }
}

/// Handles the case when an item is not found.
fn handle_not_found(
    opts: &QueryOptions,
    ctx: &CommandContext,
    suggestions: &[&ItemId],
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let config = &ctx.output;
    let emoji = get_emoji(config, &EMOJI_ERROR);
    let id = colorize(config, &opts.item_id, Color::Red, Style::None);
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

fn build_traversal_options(opts: &QueryOptions) -> TraversalOptions {
    let mut traversal_opts = TraversalOptions::new();

    if let Some(depth) = opts.depth {
        traversal_opts = traversal_opts.with_max_depth(depth);
    }

    if !opts.types.is_empty() {
        traversal_opts = traversal_opts.with_types(opts.types.clone());
    }

    traversal_opts
}

fn print_item_info(config: &OutputConfig, item: &sara_core::Item, _graph: &KnowledgeGraph) {
    let emoji = get_emoji(config, &EMOJI_ITEM);
    let id = colorize(config, item.id.as_str(), Color::Cyan, Style::Bold);
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
        item_type = item.item_type.display_name(),
        file = item.source.file_path.display(),
    );
}

fn print_direct_relationships(
    _config: &OutputConfig,
    item: &sara_core::Item,
    graph: &KnowledgeGraph,
) {
    // Print upstream (requires)
    let parents = graph.parents(&item.id);
    if !parents.is_empty() {
        println!("\n   Requires:");
        for (i, parent) in parents.iter().enumerate() {
            let branch = format_tree_branch(i == parents.len() - 1);
            println!(
                "     {branch} {id}: {name}",
                id = parent.id.as_str(),
                name = parent.name
            );
        }
    }

    // Print downstream (realized by)
    let children = graph.children(&item.id);
    if !children.is_empty() {
        println!("\n   Realized by:");
        for (i, child) in children.iter().enumerate() {
            let branch = format_tree_branch(i == children.len() - 1);
            println!(
                "     {branch} {id}: {name}",
                id = child.id.as_str(),
                name = child.name
            );
        }
    }
}

fn print_traversal(
    config: &OutputConfig,
    result: &TraversalResult,
    graph: &KnowledgeGraph,
    opts: &QueryOptions,
) {
    match opts.format {
        OutputFormat::Tree => print_traversal_tree(config, result, graph),
        OutputFormat::Json => print_traversal_json(result, graph),
    }
}

fn print_traversal_tree(config: &OutputConfig, result: &TraversalResult, graph: &KnowledgeGraph) {
    // Group items by parent to build tree structure
    let mut children_map: std::collections::HashMap<
        Option<&ItemId>,
        Vec<&sara_core::graph::TraversalNode>,
    > = std::collections::HashMap::new();

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
    node: &sara_core::graph::TraversalNode,
    graph: &KnowledgeGraph,
    children_map: &std::collections::HashMap<
        Option<&ItemId>,
        Vec<&sara_core::graph::TraversalNode>,
    >,
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
        println!("{}", colorize(config, &item_text, Color::None, Style::Bold));
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

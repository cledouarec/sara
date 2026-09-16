//! Query command implementation.

use std::collections::HashMap;
use std::error::Error;
use std::process::ExitCode;

use clap::Args;
use clap::builder::{PossibleValue, PossibleValuesParser, TypedValueParser};
use sara_core::graph::{
    KnowledgeGraph, LookupResult, TraversalOptions, TraversalResult, traverse_downstream,
    traverse_upstream,
};
use sara_core::model::{Item, ItemId, ItemType};

use sara_core::config::{Config, OutputConfig};
use sara_core::generator::{neighborhood_to_mermaid, traversal_to_mermaid};
use sara_core::service::parse_item_type;

use crate::output::{
    Color, EMOJI_ERROR, EMOJI_ITEM, Style, colorize, format_tree_branch, get_emoji, print_header,
};

/// Output format for queries.
#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum QueryFormat {
    #[default]
    Tree,
    Json,
    /// Raw Mermaid flowchart of the traversal, without code fence
    Mermaid,
}

/// Argument group of the traversal directions; a depth limit requires one.
const DIRECTION_GROUP: &str = "direction";

/// Arguments for the query command.
#[derive(Args, Debug)]
#[command(group = clap::ArgGroup::new(DIRECTION_GROUP).multiple(true))]
pub struct QueryArgs {
    /// The item identifier to query
    pub item_id: String,

    /// Limit traversal depth (requires --upstream or --downstream)
    #[arg(long, requires = DIRECTION_GROUP, help_heading = "Filters")]
    pub depth: Option<usize>,

    /// Filter by item type(s)
    #[arg(
        short = 't',
        long = "type",
        value_parser = item_type_parser(),
        ignore_case = true,
        help_heading = "Filters"
    )]
    pub item_types: Vec<ItemType>,

    /// Show downstream chain (toward Detailed Designs)
    #[arg(short, long, group = DIRECTION_GROUP, help_heading = "Traversal")]
    pub downstream: bool,

    /// Show upstream chain (toward Solution)
    #[arg(short, long, group = DIRECTION_GROUP, help_heading = "Traversal")]
    pub upstream: bool,

    /// Output format
    #[arg(long, default_value = "tree", help_heading = "Output")]
    pub format: QueryFormat,
}

/// Runs the query command.
pub fn run(args: &QueryArgs, config: &Config) -> Result<ExitCode, Box<dyn Error>> {
    let graph = super::build_graph(config)?;

    match graph.lookup(&args.item_id) {
        LookupResult::Found(item) => Ok(handle_found_item(args, &config.output, item, &graph)),
        LookupResult::NotFound { suggestions } => {
            handle_not_found(args, &config.output, &suggestions)
        }
    }
}

/// Traversal function shared by the upstream and downstream directions.
type Traverse = fn(&KnowledgeGraph, &ItemId, &TraversalOptions) -> Option<TraversalResult>;

/// Printer of one traversal result in a text format.
type TraversalPrinter = fn(&OutputConfig, &TraversalResult, &KnowledgeGraph);

/// Handles the case when an item is found.
fn handle_found_item(
    args: &QueryArgs,
    config: &OutputConfig,
    item: &Item,
    graph: &KnowledgeGraph,
) -> ExitCode {
    let options = build_traversal_options(args);
    let traversals = requested_traversals(args, item, graph, &options);

    let print_traversal: TraversalPrinter = match args.format {
        QueryFormat::Tree => print_traversal_tree,
        QueryFormat::Json => print_traversal_json,
        QueryFormat::Mermaid => {
            print_mermaid(item, graph, &traversals, &options);
            return ExitCode::SUCCESS;
        }
    };
    print_text(config, item, graph, &traversals, &options, print_traversal);

    ExitCode::SUCCESS
}

/// Runs the traversals the arguments ask for, each with its section title.
///
/// Empty when neither direction is requested.
fn requested_traversals(
    args: &QueryArgs,
    item: &Item,
    graph: &KnowledgeGraph,
    options: &TraversalOptions,
) -> Vec<(String, TraversalResult)> {
    let directions: [(bool, &str, Traverse); 2] = [
        (
            args.upstream,
            "Upstream Traceability for",
            traverse_upstream,
        ),
        (args.downstream, "Downstream from", traverse_downstream),
    ];

    directions
        .into_iter()
        .filter(|(requested, _, _)| *requested)
        .filter_map(|(_, title, traverse)| {
            traverse(graph, &item.id, options)
                .map(|result| (format!("{title} {}", item.id), result))
        })
        .collect()
}

/// Prints the item summary, then either its direct relationships (limited
/// to the type filter of `options`) or each requested traversal under a
/// section header.
fn print_text(
    config: &OutputConfig,
    item: &Item,
    graph: &KnowledgeGraph,
    traversals: &[(String, TraversalResult)],
    options: &TraversalOptions,
    print_traversal: TraversalPrinter,
) {
    print_item_info(config, item, graph);

    if traversals.is_empty() {
        print_direct_relationships(config, item, graph, options);
    }

    for (title, result) in traversals {
        println!();
        print_header(config, title);
        print_traversal(config, result, graph);
    }
}

/// Prints nothing but raw Mermaid diagrams: one per requested traversal, or
/// the direct relationships of the item (limited to the type filter of
/// `options`) when no direction is requested.
fn print_mermaid(
    item: &Item,
    graph: &KnowledgeGraph,
    traversals: &[(String, TraversalResult)],
    options: &TraversalOptions,
) {
    if traversals.is_empty() {
        print!("{}", neighborhood_to_mermaid(graph, &item.id, options));
    }

    for (_, result) in traversals {
        print!("{}", traversal_to_mermaid(result, graph));
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

/// Builds the traversal options from the depth and type arguments.
///
/// The type filter also limits the direct relationships shown when no
/// direction is requested.
fn build_traversal_options(args: &QueryArgs) -> TraversalOptions {
    let options = TraversalOptions::new().with_types(args.item_types.clone());

    match args.depth {
        Some(depth) => options.with_max_depth(depth),
        None => options,
    }
}

/// Value parser of `--type`: the ids of the active schema's item types, each
/// also accepted as its squashed form (`usecase`) or its id prefix (`uc`),
/// in any case.
///
/// Built from the schema installed before the command line is parsed, so
/// `--help` lists the ids a configured model defines.
fn item_type_parser() -> impl TypedValueParser<Value = ItemType> {
    let values = ItemType::all().into_iter().map(|item_type| {
        let id = item_type.as_str();
        PossibleValue::new(id.to_string())
            .alias(id.replace('_', ""))
            .alias(item_type.prefix().to_lowercase())
    });

    PossibleValuesParser::new(values)
        .try_map(|name| parse_item_type(&name).ok_or_else(|| format!("unknown item type `{name}`")))
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

/// Prints the items directly related to `item`, grouped by relation and
/// limited to the type filter of `options`.
fn print_direct_relationships(
    config: &OutputConfig,
    item: &Item,
    graph: &KnowledgeGraph,
    options: &TraversalOptions,
) {
    for (rel_type, related) in graph.direct_relationships(&item.id, options) {
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

fn print_traversal_json(_config: &OutputConfig, result: &TraversalResult, graph: &KnowledgeGraph) {
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

#[cfg(test)]
mod tests {
    use clap::{Command, FromArgMatches};
    use sara_core::schema::builtin;

    use super::*;

    /// Parses `--type` values through the real argument definition.
    fn parse_types(values: &[&str]) -> Result<Vec<ItemType>, clap::Error> {
        let args = ["query", "SOL-001"]
            .into_iter()
            .chain(values.iter().flat_map(|value| ["--type", value]));
        let matches = QueryArgs::augment_args(Command::new("query")).try_get_matches_from(args)?;
        Ok(QueryArgs::from_arg_matches(&matches)?.item_types)
    }

    #[test]
    fn test_type_accepts_every_item_type() {
        for item_type in ItemType::all() {
            assert_eq!(
                parse_types(&[item_type.as_str()]).unwrap(),
                vec![item_type],
                "type id `{}` must be accepted by --type",
                item_type.as_str()
            );
        }
    }

    #[test]
    fn test_type_accepts_prefix_in_any_case() {
        assert_eq!(parse_types(&["UC"]).unwrap(), vec![builtin::USE_CASE]);
    }

    #[test]
    fn test_type_rejects_unknown_type() {
        let error = parse_types(&["bogus"]).unwrap_err().to_string();

        assert!(
            error.contains("bogus"),
            "must name the rejected type: {error}"
        );
        assert!(
            error.contains("use_case"),
            "must list the types the schema defines: {error}"
        );
    }
}

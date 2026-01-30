//! Edit command implementation for modifying existing document metadata.
//!
//! Provides functionality for FR-054 through FR-066 (Edit Command).

use std::error::Error;
use std::io::{IsTerminal, stdin, stdout};
use std::process::ExitCode;

use clap::Args;
use inquire::{Confirm, InquireError};

use sara_core::error::EditError;
use sara_core::graph::KnowledgeGraph;
use sara_core::model::{EditSummary, FieldChange, ItemType};
use sara_core::service::{
    EditOptions, EditedValues, ItemContext, apply_changes, build_change_summary, edit_item,
    get_item_for_edit,
};

use super::CommandContext;
use super::interactive::{
    PrefilledFields, PromptError, prompt_description, prompt_name, prompt_platform,
    prompt_specification, prompt_traceability,
};
use crate::output::{OutputConfig, print_error, print_success};

/// Arguments for the edit command.
#[derive(Args, Debug)]
#[command(verbatim_doc_comment)]
pub struct EditArgs {
    /// The item identifier to edit
    pub item_id: String,

    /// New item description
    #[arg(short = 'd', long, help_heading = "Item Properties")]
    pub description: Option<String>,

    /// New item name
    #[arg(long, help_heading = "Item Properties")]
    pub name: Option<String>,

    /// New upstream references (for requirements) - replaces existing
    #[arg(long, num_args = 1.., help_heading = "Traceability")]
    pub derives_from: Option<Vec<String>>,

    /// New upstream references (for use_case, scenario) - replaces existing
    #[arg(long, num_args = 1.., help_heading = "Traceability")]
    pub refines: Option<Vec<String>>,

    /// New upstream references (for architectures, designs) - replaces existing
    #[arg(long, num_args = 1.., help_heading = "Traceability")]
    pub satisfies: Option<Vec<String>>,

    /// New target platform (for system_architecture)
    #[arg(long, help_heading = "Type-Specific")]
    pub platform: Option<String>,

    /// New specification statement (for requirements)
    #[arg(long, help_heading = "Type-Specific")]
    pub specification: Option<String>,
}

/// Runs the edit command.
pub fn run(args: &EditArgs, ctx: &CommandContext) -> Result<ExitCode, Box<dyn Error>> {
    // Build the knowledge graph
    let graph = ctx.build_graph()?;

    // Build options from args
    let opts = EditOptions::new(&args.item_id)
        .maybe_name(args.name.clone())
        .maybe_description(args.description.clone())
        .maybe_refines(args.refines.clone())
        .maybe_derives_from(args.derives_from.clone())
        .maybe_satisfies(args.satisfies.clone())
        .maybe_specification(args.specification.clone())
        .maybe_platform(args.platform.clone());

    // Check if interactive or non-interactive mode
    if opts.has_updates() {
        run_non_interactive_edit(&graph, &opts, &ctx.output)
    } else {
        // Interactive mode - get item context for prompts
        let item_ctx = match get_item_for_edit(&graph, &args.item_id) {
            Ok(ctx) => ctx,
            Err(e) => {
                print_error(&ctx.output, &format!("{}", e));
                if let Some(suggestions) = e.format_suggestions() {
                    println!("{}", suggestions);
                }
                return Ok(ExitCode::from(1));
            }
        };
        run_interactive_edit(&graph, &item_ctx, &ctx.output)
    }
}

/// Checks if running in a TTY environment (FR-066).
fn require_tty_for_edit() -> Result<(), EditError> {
    if !stdin().is_terminal() || !stdout().is_terminal() {
        return Err(EditError::NonInteractiveTerminal);
    }
    Ok(())
}

/// Runs the interactive edit flow (FR-055, FR-056, FR-062, FR-063).
fn run_interactive_edit(
    graph: &KnowledgeGraph,
    item: &ItemContext,
    config: &OutputConfig,
) -> Result<ExitCode, Box<dyn Error>> {
    if let Err(e) = require_tty_for_edit() {
        print_error(config, &format!("{}", e));
        return Ok(ExitCode::from(1));
    }

    display_edit_header(&item.id, item.item_type);

    match run_edit_prompts(graph, item) {
        Ok(new_values) => process_edit_changes(item, &new_values, config),
        Err(e) => handle_prompt_error(e, config),
    }
}

/// Processes edit changes: displays summary, confirms, and applies (FR-062, FR-063).
fn process_edit_changes(
    item: &ItemContext,
    new_values: &EditedValues,
    config: &OutputConfig,
) -> Result<ExitCode, Box<dyn Error>> {
    let changes = build_change_summary(item, new_values);

    display_change_summary(&changes);

    if !changes.iter().any(|c| c.is_changed()) {
        println!("\nNo changes to apply.");
        return Ok(ExitCode::SUCCESS);
    }

    confirm_and_apply_changes(item, new_values, changes, config)
}

/// Confirms with user and applies changes if confirmed.
fn confirm_and_apply_changes(
    item: &ItemContext,
    new_values: &EditedValues,
    changes: Vec<FieldChange>,
    config: &OutputConfig,
) -> Result<ExitCode, Box<dyn Error>> {
    match prompt_edit_confirmation() {
        Ok(true) => apply_and_report_changes(item, new_values, changes, config),
        Ok(false) | Err(_) => {
            print_cancelled();
            Ok(ExitCode::from(130))
        }
    }
}

/// Applies changes and prints success message.
fn apply_and_report_changes(
    item: &ItemContext,
    new_values: &EditedValues,
    changes: Vec<FieldChange>,
    config: &OutputConfig,
) -> Result<ExitCode, Box<dyn Error>> {
    apply_changes(&item.id, item.item_type, new_values, &item.file_path)?;

    let changed_count = changes.iter().filter(|c| c.is_changed()).count();
    let summary = EditSummary {
        item_id: item.id.clone(),
        file_path: item.file_path.clone(),
        changes: changes.into_iter().filter(|c| c.is_changed()).collect(),
    };

    print_success(
        config,
        &format!(
            "Updated {} ({} field{} changed)",
            summary.item_id,
            changed_count,
            if changed_count == 1 { "" } else { "s" }
        ),
    );
    Ok(ExitCode::SUCCESS)
}

/// Handles prompt errors uniformly.
fn handle_prompt_error(
    error: PromptError,
    config: &OutputConfig,
) -> Result<ExitCode, Box<dyn Error>> {
    match error {
        PromptError::Cancelled | PromptError::InquireError(InquireError::OperationInterrupted) => {
            print_cancelled();
            Ok(ExitCode::from(130))
        }
        e => {
            print_error(config, &format!("{}", e));
            Ok(ExitCode::from(1))
        }
    }
}

/// Prints the cancellation message.
fn print_cancelled() {
    println!("\nCancelled. No changes were made.");
}

/// Displays the edit header with immutable fields (FR-059, FR-060).
fn display_edit_header(item_id: &str, item_type: ItemType) {
    println!(
        "\n  Editing {} ({})\n  ────────────────────────────────────\n",
        item_id,
        item_type.display_name()
    );
}

/// Runs all edit prompts with defaults (FR-056).
fn run_edit_prompts(
    graph: &KnowledgeGraph,
    item: &ItemContext,
) -> Result<EditedValues, PromptError> {
    let name = prompt_name(None, Some(&item.name))?;
    let description = prompt_description(None, item.description.as_deref())?;

    let prefilled = PrefilledFields::default();
    let traceability = prompt_traceability(
        item.item_type,
        Some(graph),
        &prefilled,
        Some(&item.traceability),
        Some(&item.id),
    )?;

    let specification = prompt_specification(item.item_type, None, item.specification.as_deref())?;
    let platform = prompt_platform(item.item_type, None, item.platform.as_deref())?;

    Ok(EditedValues::new(name)
        .with_description(description)
        .with_specification(specification)
        .with_platform(platform)
        .with_traceability(traceability))
}

/// Displays the change summary with diff-style output (FR-063).
fn display_change_summary(changes: &[FieldChange]) {
    println!("\n  Changes to apply:\n  ────────────────────────────────────");

    for change in changes {
        if change.is_changed() {
            println!(
                "  {}: {} → {}",
                change.field.display_name(),
                change.old_value,
                change.new_value
            );
        } else {
            println!("  {}: (unchanged)", change.field.display_name());
        }
    }

    println!();
}

/// Prompts for confirmation before applying changes (FR-063).
fn prompt_edit_confirmation() -> Result<bool, PromptError> {
    let confirmed = Confirm::new("Apply changes?").with_default(true).prompt()?;
    Ok(confirmed)
}

/// Runs the non-interactive edit (FR-057, FR-058).
fn run_non_interactive_edit(
    graph: &KnowledgeGraph,
    opts: &EditOptions,
    config: &OutputConfig,
) -> Result<ExitCode, Box<dyn Error>> {
    match edit_item(graph, opts) {
        Ok(result) => {
            let msg = if result.has_changes() {
                format!(
                    "Updated {} ({} field{} changed)",
                    result.item_id,
                    result.change_count(),
                    if result.change_count() == 1 { "" } else { "s" }
                )
            } else {
                format!("Updated {}", result.item_id)
            };
            print_success(config, &msg);
            Ok(ExitCode::SUCCESS)
        }
        Err(e) => {
            print_error(config, &format!("{}", e));
            if let Some(suggestions) = e.format_suggestions() {
                println!("{}", suggestions);
            }
            Ok(ExitCode::from(1))
        }
    }
}

//! Edit command implementation for modifying existing document metadata.
//!
//! Provides functionality for FR-054 through FR-066 (Edit Command).

use std::io::{IsTerminal, stdin, stdout};
use std::process::ExitCode;

use inquire::{Confirm, InquireError};

use sara_core::{
    EditError, EditService, EditSummary, EditedValues, FieldChange, GraphBuilder, ItemContext,
    ItemType, KnowledgeGraph, TraceabilityLinks, parse_repositories,
};

use super::CommandContext;
use super::interactive::{
    PrefilledFields, PromptError, prompt_description, prompt_name, prompt_platform,
    prompt_specification, prompt_traceability,
};
use crate::output::{OutputConfig, print_error, print_success, progress};

/// CLI-specific edit options.
#[derive(Debug)]
pub struct CliEditOptions {
    pub item_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub refines: Option<Vec<String>>,
    pub derives_from: Option<Vec<String>>,
    pub satisfies: Option<Vec<String>>,
    pub specification: Option<String>,
    pub platform: Option<String>,
}

impl CliEditOptions {
    /// Returns true if any modification flag was provided (non-interactive mode).
    pub fn has_updates(&self) -> bool {
        self.name.is_some()
            || self.description.is_some()
            || self.refines.is_some()
            || self.derives_from.is_some()
            || self.satisfies.is_some()
            || self.specification.is_some()
            || self.platform.is_some()
    }
}

/// Runs the edit command.
pub fn run(
    opts: CliEditOptions,
    ctx: &CommandContext,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let service = EditService::new();

    // Build the knowledge graph
    let spinner = progress::create_spinner("Loading knowledge graph...");
    let items = parse_repositories(&ctx.repositories)?;
    let graph = GraphBuilder::new().add_items(items).build()?;
    progress::finish_and_clear(&spinner);

    // Look up the item (FR-054)
    let item = match service.lookup_item(&graph, &opts.item_id) {
        Ok(item) => item,
        Err(e) => {
            print_error(&ctx.output, &format!("{}", e));
            if let Some(suggestions) = e.format_suggestions() {
                println!("{}", suggestions);
            }
            return Ok(ExitCode::from(1));
        }
    };

    // Build item context
    let item_ctx = service.get_item_context(item);

    // Check if interactive or non-interactive mode
    if opts.has_updates() {
        // Non-interactive mode (FR-057, FR-058)
        run_non_interactive_edit(&service, &opts, &item_ctx, &ctx.output)
    } else {
        // Interactive mode (FR-055, FR-056)
        run_interactive_edit(&service, &graph, &item_ctx, &ctx.output)
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
    service: &EditService,
    graph: &KnowledgeGraph,
    item: &ItemContext,
    config: &OutputConfig,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    if let Err(e) = require_tty_for_edit() {
        print_error(config, &format!("{}", e));
        return Ok(ExitCode::from(1));
    }

    display_edit_header(&item.id, item.item_type);

    match run_edit_prompts(graph, item) {
        Ok(new_values) => process_edit_changes(service, item, &new_values, config),
        Err(e) => handle_prompt_error(e, config),
    }
}

/// Processes edit changes: displays summary, confirms, and applies (FR-062, FR-063).
fn process_edit_changes(
    service: &EditService,
    item: &ItemContext,
    new_values: &EditedValues,
    config: &OutputConfig,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let changes = service.build_change_summary(item, new_values);

    display_change_summary(&changes);

    if !changes.iter().any(|c| c.is_changed()) {
        println!("\nNo changes to apply.");
        return Ok(ExitCode::SUCCESS);
    }

    confirm_and_apply_changes(service, item, new_values, changes, config)
}

/// Confirms with user and applies changes if confirmed.
fn confirm_and_apply_changes(
    service: &EditService,
    item: &ItemContext,
    new_values: &EditedValues,
    changes: Vec<FieldChange>,
    config: &OutputConfig,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    match prompt_edit_confirmation() {
        Ok(true) => apply_and_report_changes(service, item, new_values, changes, config),
        Ok(false) | Err(_) => {
            print_cancelled();
            Ok(ExitCode::from(130))
        }
    }
}

/// Applies changes and prints success message.
fn apply_and_report_changes(
    service: &EditService,
    item: &ItemContext,
    new_values: &EditedValues,
    changes: Vec<FieldChange>,
    config: &OutputConfig,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    service.apply_changes(&item.id, item.item_type, new_values, &item.file_path)?;

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
) -> Result<ExitCode, Box<dyn std::error::Error>> {
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
    service: &EditService,
    opts: &CliEditOptions,
    item: &ItemContext,
    config: &OutputConfig,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    // Validate type-specific fields (FR-058)
    let validation_opts = build_validation_options(opts);
    if let Err(e) = service.validate_options(&validation_opts, item.item_type) {
        print_error(config, &format!("{}", e));
        return Ok(ExitCode::from(1));
    }

    // Merge updates with current values
    let new_values = EditedValues::new(opts.name.clone().unwrap_or_else(|| item.name.clone()))
        .with_description(
            opts.description
                .clone()
                .or_else(|| item.description.clone()),
        )
        .with_specification(
            opts.specification
                .clone()
                .or_else(|| item.specification.clone()),
        )
        .with_platform(opts.platform.clone().or_else(|| item.platform.clone()))
        .with_traceability(TraceabilityLinks {
            refines: opts
                .refines
                .clone()
                .unwrap_or_else(|| item.traceability.refines.clone()),
            derives_from: opts
                .derives_from
                .clone()
                .unwrap_or_else(|| item.traceability.derives_from.clone()),
            satisfies: opts
                .satisfies
                .clone()
                .unwrap_or_else(|| item.traceability.satisfies.clone()),
        });

    service.apply_changes(&item.id, item.item_type, &new_values, &item.file_path)?;
    print_success(config, &format!("Updated {}", item.id));
    Ok(ExitCode::SUCCESS)
}

/// Builds core EditOptions for validation.
fn build_validation_options(opts: &CliEditOptions) -> sara_core::CoreEditOptions {
    let mut core_opts = sara_core::CoreEditOptions::new(&opts.item_id);

    if let Some(ref spec) = opts.specification {
        core_opts = core_opts.with_specification(spec);
    }
    if let Some(ref plat) = opts.platform {
        core_opts = core_opts.with_platform(plat);
    }

    core_opts
}

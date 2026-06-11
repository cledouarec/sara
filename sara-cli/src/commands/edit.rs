//! Edit command implementation for modifying existing document metadata.
//!
//! Provides functionality for FR-054 through FR-066 (Edit Command). One
//! `--<relation>` flag is generated per primary relation of the active
//! schema, so relations added by a custom schema are editable without
//! recompiling.

use std::error::Error;
use std::io::{IsTerminal, stdin, stdout};
use std::process::ExitCode;

use clap::{Arg, ArgAction, ArgMatches, Args, Command, FromArgMatches};
use indexmap::IndexMap;
use inquire::{Confirm, InquireError};
use sara_core::error::SaraError;
use sara_core::graph::KnowledgeGraph;
use sara_core::model::{
    EditSummary, FIELD_DESCRIPTION, FIELD_ID, FIELD_NAME, FieldChange, FieldValue, ItemType,
    RelationshipType,
};
use sara_core::schema::{FieldDef, FieldType};
use sara_core::service::{EditOptions, EditService, EditedValues, FieldInput, ItemContext};

use sara_core::config::{Config, OutputConfig};

use super::EXIT_CANCELLED;
use super::init::field_help;
use super::interactive::{
    PromptError, prompt_description, prompt_field_edits, prompt_name, prompt_traceability,
};
use crate::output::{Color, Style, colorize, print_error, print_success};

/// Help heading of the static item property flags.
const HEADING_PROPERTIES: &str = "Item Properties";

/// Help heading of the schema-driven relation flags.
const HEADING_TRACEABILITY: &str = "Traceability";

/// Help heading of the type-specific field flags.
const HEADING_TYPE_SPECIFIC: &str = "Type-Specific";

/// Arguments for the edit command.
#[derive(Debug)]
pub struct EditArgs {
    /// The item identifier to edit.
    pub item_id: String,
    /// New item description.
    pub description: Option<String>,
    /// New item name.
    pub name: Option<String>,
    /// New field values, one entry per provided field flag.
    pub fields: IndexMap<String, FieldInput>,
    /// New relation targets, one entry per provided relation flag.
    pub relations: IndexMap<RelationshipType, Vec<String>>,
}

/// Argument ids of the static edit flags, reserved against dynamic ones.
const STATIC_ARG_IDS: &[&str] = &[FIELD_ID, FIELD_DESCRIPTION, FIELD_NAME];

/// Returns the primary relations of the active schema, in declaration order.
fn primary_relations() -> Vec<RelationshipType> {
    RelationshipType::all()
        .into_iter()
        .filter(RelationshipType::is_primary)
        .collect()
}

/// Returns the fields declared across the active schema's types, deduplicated
/// by name in declaration order, skipping names taken by another flag.
fn declared_fields_union() -> Vec<&'static FieldDef> {
    let relations = primary_relations();
    let mut fields: Vec<&'static FieldDef> = Vec::new();
    for item_type in ItemType::all() {
        for field in item_type.declared_fields() {
            let taken = STATIC_ARG_IDS.contains(&field.name.as_str())
                || relations.iter().any(|r| r.as_str() == field.name)
                || fields.iter().any(|f| f.name == field.name);
            if !taken {
                fields.push(field);
            }
        }
    }
    fields
}

impl Args for EditArgs {
    fn augment_args(mut command: Command) -> Command {
        command = command
            .arg(
                Arg::new(FIELD_ID)
                    .value_name("ITEM_ID")
                    .required(true)
                    .help("The item identifier to edit"),
            )
            .arg(
                Arg::new(FIELD_DESCRIPTION)
                    .long(FIELD_DESCRIPTION)
                    .short('d')
                    .help_heading(HEADING_PROPERTIES)
                    .help("New item description"),
            )
            .arg(
                Arg::new(FIELD_NAME)
                    .long(FIELD_NAME)
                    .help_heading(HEADING_PROPERTIES)
                    .help("New item name"),
            );

        for field in declared_fields_union() {
            let mut arg = Arg::new(field.name.clone())
                .long(field.name.replace('_', "-"))
                .help_heading(HEADING_TYPE_SPECIFIC)
                .help(field_help(field));
            if matches!(field.field_type, FieldType::List(_)) {
                arg = arg.num_args(1..).action(ArgAction::Append);
            }
            command = command.arg(arg);
        }

        for relation in primary_relations() {
            command = command.arg(
                Arg::new(relation.as_str())
                    .long(relation.as_str().replace('_', "-"))
                    .num_args(1..)
                    .action(ArgAction::Append)
                    .help_heading(HEADING_TRACEABILITY)
                    .help(format!(
                        "New {} references - replaces existing",
                        relation.as_str().replace('_', " ")
                    )),
            );
        }

        command
    }

    fn augment_args_for_update(command: Command) -> Command {
        Self::augment_args(command)
    }
}

impl FromArgMatches for EditArgs {
    fn from_arg_matches(matches: &ArgMatches) -> Result<Self, clap::Error> {
        let mut fields = IndexMap::new();
        for field in declared_fields_union() {
            if matches!(field.field_type, FieldType::List(_)) {
                if let Some(values) = matches.get_many::<String>(&field.name) {
                    fields.insert(
                        field.name.clone(),
                        FieldInput::List(values.cloned().collect()),
                    );
                }
            } else if let Some(value) = matches.get_one::<String>(&field.name) {
                fields.insert(field.name.clone(), FieldInput::Text(value.clone()));
            }
        }

        let mut relations = IndexMap::new();
        for relation in primary_relations() {
            if let Some(ids) = matches.get_many::<String>(relation.as_str()) {
                relations.insert(relation, ids.cloned().collect());
            }
        }

        Ok(Self {
            item_id: matches
                .get_one::<String>(FIELD_ID)
                .cloned()
                .unwrap_or_default(),
            description: matches.get_one::<String>(FIELD_DESCRIPTION).cloned(),
            name: matches.get_one::<String>(FIELD_NAME).cloned(),
            fields,
            relations,
        })
    }

    fn update_from_arg_matches(&mut self, matches: &ArgMatches) -> Result<(), clap::Error> {
        *self = Self::from_arg_matches(matches)?;
        Ok(())
    }
}

/// Runs the edit command.
pub fn run(args: &EditArgs, config: &Config) -> Result<ExitCode, Box<dyn Error>> {
    let service = EditService::new();

    let graph = super::build_graph(config)?;

    // Look up the item (FR-054)
    let item = match service.lookup_item(&graph, &args.item_id) {
        Ok(item) => item,
        Err(e) => {
            print_error(&config.output, &format!("{}", e));
            if let Some(suggestions) = e.format_suggestions() {
                println!(
                    "{}",
                    colorize(&config.output, &suggestions, Color::None, Style::Dimmed)
                );
            }
            return Ok(ExitCode::FAILURE);
        }
    };

    // Build item context
    let item_ctx = service.get_item_context(item);

    // Build options from args
    let mut opts = EditOptions::new(&args.item_id)
        .maybe_name(args.name.clone())
        .maybe_description(args.description.clone());
    for (name, input) in &args.fields {
        opts = opts.with_field(name.clone(), input.clone());
    }
    for (relation, ids) in &args.relations {
        opts = opts.with_relation(*relation, ids.clone());
    }

    // Check if interactive or non-interactive mode
    if opts.has_updates() {
        // Non-interactive mode (FR-057, FR-058)
        run_non_interactive_edit(&service, opts, &item_ctx, &config.output)
    } else {
        // Interactive mode (FR-055, FR-056)
        run_interactive_edit(&service, &graph, &item_ctx, &config.output)
    }
}

/// Checks if running in a TTY environment (FR-066).
fn require_tty_for_edit() -> Result<(), SaraError> {
    if !stdin().is_terminal() || !stdout().is_terminal() {
        return Err(SaraError::NonInteractiveTerminal);
    }
    Ok(())
}

/// Runs the interactive edit flow (FR-055, FR-056, FR-062, FR-063).
fn run_interactive_edit(
    service: &EditService,
    graph: &KnowledgeGraph,
    item: &ItemContext,
    config: &OutputConfig,
) -> Result<ExitCode, Box<dyn Error>> {
    if let Err(e) = require_tty_for_edit() {
        print_error(config, &format!("{}", e));
        return Ok(ExitCode::FAILURE);
    }

    display_edit_header(config, &item.id, item.item_type);

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
) -> Result<ExitCode, Box<dyn Error>> {
    let changes = service.build_change_summary(item, new_values);

    display_change_summary(config, &changes);

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
) -> Result<ExitCode, Box<dyn Error>> {
    match prompt_edit_confirmation() {
        Ok(true) => apply_and_report_changes(service, item, new_values, changes, config),
        Ok(false) | Err(_) => {
            print_cancelled();
            Ok(ExitCode::from(EXIT_CANCELLED))
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
) -> Result<ExitCode, Box<dyn Error>> {
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
) -> Result<ExitCode, Box<dyn Error>> {
    match error {
        PromptError::Cancelled | PromptError::InquireError(InquireError::OperationInterrupted) => {
            print_cancelled();
            Ok(ExitCode::from(EXIT_CANCELLED))
        }
        e => {
            print_error(config, &format!("{}", e));
            Ok(ExitCode::FAILURE)
        }
    }
}

/// Prints the cancellation message.
fn print_cancelled() {
    println!("\nCancelled. No changes were made.");
}

/// Displays the edit header with immutable fields (FR-059, FR-060).
fn display_edit_header(config: &OutputConfig, item_id: &str, item_type: ItemType) {
    let id = colorize(config, item_id, Color::Cyan, Style::Bold);
    let item_type = colorize(config, item_type.display_name(), Color::None, Style::Dimmed);
    println!("\n  Editing {id} ({item_type})");
}

/// Runs all edit prompts with defaults (FR-056).
fn run_edit_prompts(
    graph: &KnowledgeGraph,
    item: &ItemContext,
) -> Result<EditedValues, PromptError> {
    let name = prompt_name(Some(&item.name))?;
    let description = prompt_description(item.description.as_deref())?;

    let traceability = prompt_traceability(
        item.item_type,
        Some(graph),
        Some(&item.traceability),
        Some(&item.id),
    )?;

    let mut attributes = item.attributes.clone();
    for (field, value) in prompt_field_edits(item.item_type, &item.attributes)? {
        match value {
            Some(value) => {
                attributes.insert(field, FieldValue::Text(value));
            }
            None => {
                attributes.remove(&field);
            }
        }
    }

    Ok(EditedValues::new(name)
        .with_description(description)
        .with_traceability(traceability)
        .with_attributes(attributes))
}

/// Displays the change summary with diff-style output (FR-063).
fn display_change_summary(config: &OutputConfig, changes: &[FieldChange]) {
    let header = colorize(config, "Changes to apply:", Color::None, Style::Bold);
    println!("\n  {header}");

    for change in changes {
        if change.is_changed() {
            println!(
                "  {}: {} → {}",
                change.field, change.old_value, change.new_value
            );
        } else {
            let unchanged = colorize(config, "(unchanged)", Color::None, Style::Dimmed);
            println!("  {}: {unchanged}", change.field);
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
    opts: EditOptions,
    item: &ItemContext,
    config: &OutputConfig,
) -> Result<ExitCode, Box<dyn Error>> {
    // Validate type-specific fields (FR-058)
    if let Err(e) = service.validate_options(&opts, item.item_type) {
        print_error(config, &format!("{}", e));
        return Ok(ExitCode::FAILURE);
    }

    // Merge updates with current values
    let new_values = service.merge_values(opts, item);

    service.apply_changes(&item.id, item.item_type, &new_values, &item.file_path)?;
    print_success(config, &format!("Updated {}", item.id));
    Ok(ExitCode::SUCCESS)
}

//! Interactive mode for the init command.
//!
//! Provides terminal prompts for creating requirement documents when
//! the --type argument is not provided (FR-040 through FR-052).

use std::io::{IsTerminal, stdin, stdout};
use std::path::PathBuf;

use inquire::validator::{StringValidator, Validation};
use inquire::{Confirm, InquireError, MultiSelect, Select, Text};
use thiserror::Error;

use sara_core::{
    FieldName, ItemType, KnowledgeGraph, MissingParentError, TraceabilityLinks,
    check_parent_exists, parse_repositories, suggest_next_id,
};

use crate::output::{OutputConfig, print_error, progress};

/// Fields pre-provided via CLI arguments (FR-050).
#[derive(Debug, Default)]
pub struct PrefilledFields {
    pub file: Option<PathBuf>,
    pub item_type: Option<ItemType>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub refines: Vec<String>,
    pub derives_from: Vec<String>,
    pub satisfies: Vec<String>,
    pub specification: Option<String>,
    pub platform: Option<String>,
}

/// Configuration for an interactive init session.
pub struct InteractiveSession<'a> {
    /// Pre-parsed knowledge graph for traceability lookups.
    pub graph: Option<KnowledgeGraph>,

    /// Pre-provided fields from CLI arguments (skip prompts for these).
    pub prefilled: PrefilledFields,

    /// Repository paths for graph building.
    pub repositories: &'a [PathBuf],
}

/// Collected input from interactive session.
#[derive(Debug)]
pub struct InteractiveInput {
    pub file: PathBuf,
    pub item_type: ItemType,
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub traceability: TraceabilityLinks,
    pub type_specific: TypeSpecificInput,
}

/// Type-specific fields.
#[derive(Debug, Default)]
pub struct TypeSpecificInput {
    /// For requirement types.
    pub specification: Option<String>,
    /// For SystemArchitecture.
    pub platform: Option<String>,
}

/// Errors that can occur during interactive prompts.
#[derive(Debug, Error)]
pub enum PromptError {
    #[error("Interactive mode requires a terminal. Use --type <TYPE> to specify the item type.")]
    NonInteractiveTerminal,

    #[error(transparent)]
    MissingParent(#[from] MissingParentError),

    #[error("User cancelled")]
    Cancelled,

    #[error("Prompt error: {0}")]
    InquireError(#[from] InquireError),
}

/// Option displayed in Select/MultiSelect prompts.
#[derive(Debug, Clone)]
pub struct SelectOption {
    /// Item ID (e.g., "SOL-001").
    pub id: String,
    /// Item name for display.
    pub name: String,
}

impl std::fmt::Display for SelectOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.id, self.name)
    }
}

/// Checks if the terminal is interactive (FR-051).
pub fn require_tty() -> Result<(), PromptError> {
    if !stdin().is_terminal() || !stdout().is_terminal() {
        return Err(PromptError::NonInteractiveTerminal);
    }
    Ok(())
}

/// ID format validator (alphanumeric, hyphens, underscores).
#[derive(Clone)]
struct IdValidator;

impl StringValidator for IdValidator {
    fn validate(&self, input: &str) -> Result<Validation, inquire::CustomUserError> {
        if input.is_empty() {
            return Ok(Validation::Invalid("ID cannot be empty".into()));
        }

        if input
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            Ok(Validation::Valid)
        } else {
            Ok(Validation::Invalid(
                "ID must contain only letters, numbers, hyphens, and underscores".into(),
            ))
        }
    }
}

/// Name length validator (non-empty, reasonable length).
#[derive(Clone)]
struct NameValidator;

impl StringValidator for NameValidator {
    fn validate(&self, input: &str) -> Result<Validation, inquire::CustomUserError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Ok(Validation::Invalid("Name is required".into()));
        }
        if trimmed.len() > 200 {
            return Ok(Validation::Invalid(
                "Name must be 200 characters or less".into(),
            ));
        }
        Ok(Validation::Valid)
    }
}

/// Prompts for item type selection (FR-041).
fn prompt_item_type(prefilled: Option<ItemType>) -> Result<ItemType, PromptError> {
    if let Some(item_type) = prefilled {
        return Ok(item_type);
    }

    let options: Vec<ItemType> = ItemType::all().to_vec();
    let selection = Select::new("Select item type:", options)
        .with_help_message("Use arrow keys to navigate, Enter to select")
        .prompt()?;

    Ok(selection)
}

/// Prompts for item name (FR-042, FR-056).
///
/// If `prefilled` is Some, returns that value without prompting.
/// If `default` is Some, shows that value as the default in the prompt.
pub fn prompt_name(
    prefilled: Option<&String>,
    default: Option<&str>,
) -> Result<String, PromptError> {
    if let Some(name) = prefilled {
        return Ok(name.clone());
    }

    let mut prompt = Text::new("Item name:")
        .with_validator(NameValidator)
        .with_help_message("Enter a human-readable name for this item");

    if let Some(def) = default {
        prompt = prompt.with_default(def);
    }

    let name = prompt.prompt()?;
    Ok(name.trim().to_string())
}

/// Prompts for item description (FR-043, FR-056).
///
/// If `prefilled` is Some, returns that value without prompting.
/// If `default` is Some, shows that value as the default in the prompt.
pub fn prompt_description(
    prefilled: Option<&String>,
    default: Option<&str>,
) -> Result<Option<String>, PromptError> {
    if let Some(desc) = prefilled {
        return Ok(Some(desc.clone()));
    }

    let mut prompt = Text::new("Description (optional):")
        .with_help_message("Brief summary of the item (press Enter to skip)");

    if let Some(def) = default {
        prompt = prompt.with_default(def);
    }

    let desc = prompt.prompt()?;
    let trimmed = desc.trim();
    if trimmed.is_empty() {
        Ok(None)
    } else {
        Ok(Some(trimmed.to_string()))
    }
}

/// Prompts for identifier with suggested default (FR-044).
fn prompt_identifier(
    item_type: ItemType,
    graph: Option<&KnowledgeGraph>,
    prefilled: Option<&String>,
) -> Result<String, PromptError> {
    if let Some(id) = prefilled {
        return Ok(id.clone());
    }

    let suggested = suggest_next_id(item_type, graph);
    let id = Text::new("Identifier:")
        .with_default(&suggested)
        .with_validator(IdValidator)
        .with_help_message("Unique identifier (suggested default shown)")
        .prompt()?;

    Ok(id.trim().to_string())
}

/// Gets items of specific types for traceability selection.
fn get_items_of_type(graph: Option<&KnowledgeGraph>, item_type: ItemType) -> Vec<SelectOption> {
    graph
        .map(|g| {
            g.items()
                .filter(|item| item.item_type == item_type)
                .map(|item| SelectOption {
                    id: item.id.as_str().to_string(),
                    name: item.name.clone(),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Type alias for pre-selected traceability items (edit mode, FR-056).
/// Uses TraceabilityLinks from sara-core.
pub type PreselectedTraceability = TraceabilityLinks;

/// Helper to compute default selection indices for MultiSelect.
fn compute_default_indices(options: &[SelectOption], preselected: &[String]) -> Vec<usize> {
    options
        .iter()
        .enumerate()
        .filter(|(_, opt)| preselected.contains(&opt.id))
        .map(|(i, _)| i)
        .collect()
}

/// The type of traceability relationship (CLI-specific enum for prompt handling).
#[derive(Debug, Clone, Copy)]
enum TraceabilityKind {
    Refines,
    DerivesFrom,
    Satisfies,
}

impl TraceabilityKind {
    /// Creates a TraceabilityKind from the FieldName.
    fn from_field(field: FieldName) -> Self {
        match field {
            FieldName::Refines => Self::Refines,
            FieldName::DerivesFrom => Self::DerivesFrom,
            FieldName::Satisfies => Self::Satisfies,
            _ => Self::Refines, // Fallback
        }
    }
}

/// Configuration for a traceability prompt (CLI-specific).
struct TraceabilityPromptConfig {
    kind: TraceabilityKind,
    parent_type: ItemType,
    prompt_message: String,
}

/// Returns the traceability prompt configuration for an item type.
///
/// Uses core's `ItemType::traceability_config()` for domain logic,
/// adds CLI-specific prompt messages.
fn get_traceability_prompt_config(item_type: ItemType) -> Option<TraceabilityPromptConfig> {
    let config = item_type.traceability_config()?;
    let kind = TraceabilityKind::from_field(config.relationship_field);

    let prompt_message = format!(
        "Select {} this {} {}:",
        config.parent_type.display_name(),
        item_type.display_name(),
        config.relationship_field.as_str().replace('_', " ")
    );

    Some(TraceabilityPromptConfig {
        kind,
        parent_type: config.parent_type,
        prompt_message,
    })
}

/// Gets the prefilled values for a traceability kind.
fn get_prefilled_for_kind(prefilled: &PrefilledFields, kind: TraceabilityKind) -> &[String] {
    match kind {
        TraceabilityKind::Refines => &prefilled.refines,
        TraceabilityKind::DerivesFrom => &prefilled.derives_from,
        TraceabilityKind::Satisfies => &prefilled.satisfies,
    }
}

/// Gets the preselected values for a traceability kind.
fn get_preselected_for_kind(
    preselected: Option<&PreselectedTraceability>,
    kind: TraceabilityKind,
) -> Vec<String> {
    preselected
        .map(|p| match kind {
            TraceabilityKind::Refines => p.refines.clone(),
            TraceabilityKind::DerivesFrom => p.derives_from.clone(),
            TraceabilityKind::Satisfies => p.satisfies.clone(),
        })
        .unwrap_or_default()
}

/// Prompts for selecting parent items and returns the selected IDs.
fn prompt_parent_selection(
    options: Vec<SelectOption>,
    prompt_message: &str,
    preselected_ids: &[String],
) -> Result<Vec<String>, PromptError> {
    if options.is_empty() {
        return Ok(Vec::new());
    }

    let defaults = compute_default_indices(&options, preselected_ids);
    let selected = MultiSelect::new(prompt_message, options)
        .with_help_message("Space to select, Enter to confirm")
        .with_default(&defaults)
        .prompt()?;

    Ok(selected.into_iter().map(|s| s.id).collect())
}

/// Applies selected IDs to the appropriate field in TraceabilityLinks.
fn apply_selection_to_input(
    input: &mut TraceabilityLinks,
    kind: TraceabilityKind,
    ids: Vec<String>,
) {
    match kind {
        TraceabilityKind::Refines => input.refines = ids,
        TraceabilityKind::DerivesFrom => input.derives_from = ids,
        TraceabilityKind::Satisfies => input.satisfies = ids,
    }
}

/// Prompts for traceability relationships (FR-045, FR-056).
///
/// If `preselected` is Some, those items will be pre-checked in the MultiSelect.
pub fn prompt_traceability(
    item_type: ItemType,
    graph: Option<&KnowledgeGraph>,
    prefilled: &PrefilledFields,
    preselected: Option<&PreselectedTraceability>,
) -> Result<TraceabilityLinks, PromptError> {
    let mut input = TraceabilityLinks::default();

    let Some(config) = get_traceability_prompt_config(item_type) else {
        return Ok(input);
    };

    let prefilled_values = get_prefilled_for_kind(prefilled, config.kind);
    if !prefilled_values.is_empty() {
        apply_selection_to_input(&mut input, config.kind, prefilled_values.to_vec());
        return Ok(input);
    }

    let options = get_items_of_type(graph, config.parent_type);
    let preselected_ids = get_preselected_for_kind(preselected, config.kind);
    let selected = prompt_parent_selection(options, &config.prompt_message, &preselected_ids)?;
    apply_selection_to_input(&mut input, config.kind, selected);

    Ok(input)
}

/// Prompts for specification (FR-046, FR-056, for requirement types).
///
/// If `prefilled` is Some, returns that value without prompting.
/// If `default` is Some, shows that value as the default in the prompt.
pub fn prompt_specification(
    item_type: ItemType,
    prefilled: Option<&String>,
    default: Option<&str>,
) -> Result<Option<String>, PromptError> {
    if !item_type.requires_specification() {
        return Ok(None);
    }

    if let Some(spec) = prefilled {
        return Ok(Some(spec.clone()));
    }

    let mut prompt = Text::new("Specification:")
        .with_help_message("Enter the SHALL statement (e.g., 'The system SHALL...')")
        .with_validator(NameValidator); // Reuse for non-empty

    if let Some(def) = default {
        prompt = prompt.with_default(def);
    }

    let spec = prompt.prompt()?;
    Ok(Some(spec.trim().to_string()))
}

/// Prompts for platform (FR-046, FR-056, for system_architecture).
///
/// If `prefilled` is Some, returns that value without prompting.
/// If `default` is Some, shows that value as the default in the prompt.
pub fn prompt_platform(
    item_type: ItemType,
    prefilled: Option<&String>,
    default: Option<&str>,
) -> Result<Option<String>, PromptError> {
    if item_type != ItemType::SystemArchitecture {
        return Ok(None);
    }

    if let Some(platform) = prefilled {
        return Ok(Some(platform.clone()));
    }

    let mut prompt = Text::new("Target platform (optional):")
        .with_help_message("e.g., AWS, STM32, Linux (press Enter to skip)");

    if let Some(def) = default {
        prompt = prompt.with_default(def);
    }

    let platform = prompt.prompt()?;
    let trimmed = platform.trim();
    if trimmed.is_empty() {
        Ok(None)
    } else {
        Ok(Some(trimmed.to_string()))
    }
}

/// Displays a summary and prompts for confirmation (FR-048).
fn prompt_confirmation(input: &InteractiveInput) -> Result<bool, PromptError> {
    let summary = build_confirmation_summary(input);
    println!("{}", summary);

    let confirmed = Confirm::new("Create document?")
        .with_default(true)
        .prompt()?;

    Ok(confirmed)
}

/// Builds the confirmation summary string.
fn build_confirmation_summary(input: &InteractiveInput) -> String {
    let description = input
        .description
        .as_ref()
        .map(|d| format!("\n  Description: {}", d))
        .unwrap_or_default();

    let refines = if input.traceability.refines.is_empty() {
        String::new()
    } else {
        format!("\n  Refines: {}", input.traceability.refines.join(", "))
    };

    let derives_from = if input.traceability.derives_from.is_empty() {
        String::new()
    } else {
        format!(
            "\n  Derives from: {}",
            input.traceability.derives_from.join(", ")
        )
    };

    let satisfies = if input.traceability.satisfies.is_empty() {
        String::new()
    } else {
        format!("\n  Satisfies: {}", input.traceability.satisfies.join(", "))
    };

    let specification = input
        .type_specific
        .specification
        .as_ref()
        .map(|s| format!("\n  Specification: {}", s))
        .unwrap_or_default();

    let platform = input
        .type_specific
        .platform
        .as_ref()
        .map(|p| format!("\n  Platform: {}", p))
        .unwrap_or_default();

    format!(
        "\n\
         \x20 Summary:\n\
         \x20 ────────────────────────────────────\n\
         \x20 Type: {}\n\
         \x20 ID:   {}\n\
         \x20 Name: {}\n\
         \x20 File: {}{}{}{}{}{}{}\n",
        input.item_type.display_name(),
        input.id,
        input.name,
        input.file.display(),
        description,
        refines,
        derives_from,
        satisfies,
        specification,
        platform,
    )
}

/// Prompts for file path if not provided.
fn prompt_file(prefilled: Option<&PathBuf>) -> Result<PathBuf, PromptError> {
    if let Some(file) = prefilled {
        return Ok(file.clone());
    }

    let file = Text::new("File path:")
        .with_help_message("Path for the new document (e.g., docs/SOL-001.md)")
        .with_validator(|input: &str| {
            let trimmed = input.trim();
            if trimmed.is_empty() {
                Ok(Validation::Invalid("File path is required".into()))
            } else {
                Ok(Validation::Valid)
            }
        })
        .prompt()?;

    Ok(PathBuf::from(file.trim()))
}

/// Runs the interactive session, orchestrating all prompts (FR-040).
pub fn run_interactive_session(
    session: &mut InteractiveSession<'_>,
) -> Result<InteractiveInput, PromptError> {
    require_tty()?;
    ensure_graph_loaded(session);

    let item_type = prompt_item_type(session.prefilled.item_type)?;
    check_parent_exists(item_type, session.graph.as_ref())?;

    let input = collect_item_input(session, item_type)?;
    confirm_creation(&input)?;

    Ok(input)
}

/// Ensures the knowledge graph is loaded for traceability suggestions.
fn ensure_graph_loaded(session: &mut InteractiveSession<'_>) {
    if session.graph.is_some() || session.repositories.is_empty() {
        return;
    }

    let spinner = progress::create_spinner("Building knowledge graph...");
    match build_graph_from_repositories(session.repositories) {
        Ok(graph) => {
            progress::finish_and_clear(&spinner);
            session.graph = Some(graph);
        }
        Err(msg) => {
            progress::finish_with_error(&spinner, msg);
        }
    }
}

/// Builds the knowledge graph from repositories.
fn build_graph_from_repositories(repositories: &[PathBuf]) -> Result<KnowledgeGraph, &'static str> {
    let items = parse_repositories(repositories).map_err(|e| {
        tracing::warn!("Parse error: {}", e);
        "Failed to parse repositories"
    })?;

    sara_core::GraphBuilder::new()
        .add_items(items)
        .build()
        .map_err(|e| {
            tracing::warn!("Graph build error: {}", e);
            "Failed to build graph"
        })
}

/// Collects all item input through prompts.
fn collect_item_input(
    session: &InteractiveSession<'_>,
    item_type: ItemType,
) -> Result<InteractiveInput, PromptError> {
    let name = prompt_name(session.prefilled.name.as_ref(), None)?;
    let id = prompt_identifier(
        item_type,
        session.graph.as_ref(),
        session.prefilled.id.as_ref(),
    )?;
    let description = prompt_description(session.prefilled.description.as_ref(), None)?;
    let traceability =
        prompt_traceability(item_type, session.graph.as_ref(), &session.prefilled, None)?;
    let type_specific = collect_type_specific_input(session, item_type)?;
    let file = prompt_file(session.prefilled.file.as_ref())?;

    Ok(InteractiveInput {
        file,
        item_type,
        id,
        name,
        description,
        traceability,
        type_specific,
    })
}

/// Collects type-specific fields (specification, platform).
fn collect_type_specific_input(
    session: &InteractiveSession<'_>,
    item_type: ItemType,
) -> Result<TypeSpecificInput, PromptError> {
    let specification =
        prompt_specification(item_type, session.prefilled.specification.as_ref(), None)?;
    let platform = prompt_platform(item_type, session.prefilled.platform.as_ref(), None)?;

    Ok(TypeSpecificInput {
        specification,
        platform,
    })
}

/// Confirms the creation with the user (FR-048).
fn confirm_creation(input: &InteractiveInput) -> Result<(), PromptError> {
    if prompt_confirmation(input)? {
        Ok(())
    } else {
        Err(PromptError::Cancelled)
    }
}

/// Handles the result of an interactive session, including Ctrl+C (FR-049).
pub fn handle_interactive_result(
    result: Result<InteractiveInput, PromptError>,
    config: &OutputConfig,
) -> Result<Option<InteractiveInput>, PromptError> {
    match result {
        Ok(input) => Ok(Some(input)),
        Err(PromptError::Cancelled) => {
            println!();
            println!("Cancelled. No file was created.");
            Ok(None)
        }
        Err(PromptError::InquireError(InquireError::OperationInterrupted)) => {
            println!();
            println!("Cancelled. No file was created.");
            Ok(None)
        }
        Err(PromptError::NonInteractiveTerminal) => {
            print_error(
                config,
                "Interactive mode requires a terminal. Use --type <TYPE> to specify the item type.",
            );
            Err(PromptError::NonInteractiveTerminal)
        }
        Err(PromptError::MissingParent(ref err)) => {
            print_error(config, &err.to_string());
            Err(PromptError::MissingParent(err.clone()))
        }
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggest_next_id_no_graph() {
        let id = suggest_next_id(ItemType::Solution, None);
        assert!(id.starts_with("SOL-"));
    }

    #[test]
    fn test_id_validator_valid() {
        let validator = IdValidator;
        assert!(matches!(
            validator.validate("SOL-001"),
            Ok(Validation::Valid)
        ));
        assert!(matches!(
            validator.validate("UC_002"),
            Ok(Validation::Valid)
        ));
    }

    #[test]
    fn test_id_validator_invalid() {
        let validator = IdValidator;
        assert!(matches!(validator.validate(""), Ok(Validation::Invalid(_))));
        assert!(matches!(
            validator.validate("SOL 001"),
            Ok(Validation::Invalid(_))
        ));
    }

    #[test]
    fn test_name_validator_valid() {
        let validator = NameValidator;
        assert!(matches!(
            validator.validate("Test Name"),
            Ok(Validation::Valid)
        ));
    }

    #[test]
    fn test_name_validator_empty() {
        let validator = NameValidator;
        assert!(matches!(validator.validate(""), Ok(Validation::Invalid(_))));
        assert!(matches!(
            validator.validate("   "),
            Ok(Validation::Invalid(_))
        ));
    }

    #[test]
    fn test_required_parent_type() {
        assert_eq!(ItemType::Solution.required_parent_type(), None);
        assert_eq!(
            ItemType::UseCase.required_parent_type(),
            Some(ItemType::Solution)
        );
        assert_eq!(
            ItemType::Scenario.required_parent_type(),
            Some(ItemType::UseCase)
        );
    }

    #[test]
    fn test_traceability_field() {
        assert_eq!(ItemType::Solution.traceability_field(), None);
        assert_eq!(
            ItemType::UseCase.traceability_field(),
            Some(FieldName::Refines)
        );
        assert_eq!(
            ItemType::SystemRequirement.traceability_field(),
            Some(FieldName::DerivesFrom)
        );
        assert_eq!(
            ItemType::SystemArchitecture.traceability_field(),
            Some(FieldName::Satisfies)
        );
    }
}

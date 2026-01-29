//! Init types for item initialization (domain layer).
//!
//! Provides domain-level types for item creation that are independent
//! of file I/O and format-specific concerns.

use crate::error::ValidationError;

use super::{ItemType, TypeFields};

/// Domain options for creating a new item.
///
/// This struct contains all the information needed to create an item,
/// using the flat `TypeFields` container instead of type-specific variants.
#[derive(Debug, Clone)]
pub struct InitOptions {
    /// The type of item to create.
    pub item_type: ItemType,
    /// Optional ID (will be auto-generated if not provided).
    pub id: Option<String>,
    /// Optional name (will be derived from context if not provided).
    pub name: Option<String>,
    /// Optional description.
    pub description: Option<String>,
    /// Type-specific fields.
    pub fields: TypeFields,
}

impl InitOptions {
    /// Creates new init options for the given item type.
    pub fn new(item_type: ItemType) -> Self {
        Self {
            item_type,
            id: None,
            name: None,
            description: None,
            fields: TypeFields::default(),
        }
    }

    /// Creates init options with specific fields.
    pub fn with_fields(item_type: ItemType, fields: TypeFields) -> Self {
        Self {
            item_type,
            id: None,
            name: None,
            description: None,
            fields,
        }
    }

    /// Sets the ID.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the ID if provided.
    pub fn maybe_id(mut self, id: Option<String>) -> Self {
        self.id = id;
        self
    }

    /// Sets the name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the name if provided.
    pub fn maybe_name(mut self, name: Option<String>) -> Self {
        self.name = name;
        self
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the description if provided.
    pub fn maybe_description(mut self, description: Option<String>) -> Self {
        self.description = description;
        self
    }

    /// Sets the type-specific fields.
    pub fn with_type_fields(mut self, fields: TypeFields) -> Self {
        self.fields = fields;
        self
    }

    /// Validates the options.
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.fields.validate_for_type(self.item_type)
    }

    /// Returns true if specification is needed but not provided.
    pub fn needs_specification(&self) -> bool {
        self.fields.needs_specification(self.item_type)
    }
}

/// Prepared item data ready for generation.
///
/// This is the output of `prepare_init()` - validated and complete
/// item data that can be passed to a generator.
#[derive(Debug, Clone)]
pub struct InitData {
    /// The resolved ID (generated if not provided).
    pub id: String,
    /// The resolved name.
    pub name: String,
    /// The item type.
    pub item_type: ItemType,
    /// Optional description.
    pub description: Option<String>,
    /// Type-specific fields.
    pub fields: TypeFields,
}

impl InitData {
    /// Returns true if specification is needed but not provided.
    pub fn needs_specification(&self) -> bool {
        self.fields.needs_specification(self.item_type)
    }
}

/// Prepares item data for creation (domain logic only).
///
/// This function validates the options and resolves defaults,
/// producing `InitData` ready for a generator.
///
/// # Arguments
/// * `opts` - The initialization options
/// * `id_generator` - Function to generate an ID if not provided
/// * `name_resolver` - Function to resolve a name if not provided
pub fn prepare_init<F, G>(
    opts: &InitOptions,
    id_generator: F,
    name_resolver: G,
) -> Result<InitData, ValidationError>
where
    F: FnOnce(ItemType) -> String,
    G: FnOnce(&str) -> String,
{
    // Validate fields for this item type
    opts.fields.validate_for_type(opts.item_type)?;

    // Resolve ID
    let id = opts
        .id
        .clone()
        .unwrap_or_else(|| id_generator(opts.item_type));

    // Resolve name
    let name = opts.name.clone().unwrap_or_else(|| name_resolver(&id));

    Ok(InitData {
        id,
        name,
        item_type: opts.item_type,
        description: opts.description.clone(),
        fields: opts.fields.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::AdrStatus;

    #[test]
    fn test_init_options_new() {
        let opts = InitOptions::new(ItemType::Solution);
        assert_eq!(opts.item_type, ItemType::Solution);
        assert!(opts.id.is_none());
        assert!(opts.name.is_none());
    }

    #[test]
    fn test_init_options_builder() {
        let opts = InitOptions::new(ItemType::Solution)
            .with_id("SOL-001")
            .with_name("Test Solution")
            .with_description("A test");

        assert_eq!(opts.id, Some("SOL-001".to_string()));
        assert_eq!(opts.name, Some("Test Solution".to_string()));
        assert_eq!(opts.description, Some("A test".to_string()));
    }

    #[test]
    fn test_init_options_validate_valid() {
        let fields = TypeFields::new().with_specification("The system SHALL do X");
        let opts = InitOptions::with_fields(ItemType::SystemRequirement, fields);
        assert!(opts.validate().is_ok());
    }

    #[test]
    fn test_init_options_validate_invalid() {
        let fields = TypeFields::new().with_specification("Invalid for Solution");
        let opts = InitOptions::with_fields(ItemType::Solution, fields);
        assert!(opts.validate().is_err());
    }

    #[test]
    fn test_init_options_needs_specification() {
        let opts = InitOptions::new(ItemType::SystemRequirement);
        assert!(opts.needs_specification());

        let fields = TypeFields::new().with_specification("Has spec");
        let opts_with_spec = InitOptions::with_fields(ItemType::SystemRequirement, fields);
        assert!(!opts_with_spec.needs_specification());

        let opts_solution = InitOptions::new(ItemType::Solution);
        assert!(!opts_solution.needs_specification());
    }

    #[test]
    fn test_prepare_init_basic() {
        let opts = InitOptions::new(ItemType::Solution)
            .with_id("SOL-001")
            .with_name("Test Solution");

        let data = prepare_init(&opts, |_| "SOL-999".to_string(), |id| id.to_string()).unwrap();

        assert_eq!(data.id, "SOL-001");
        assert_eq!(data.name, "Test Solution");
        assert_eq!(data.item_type, ItemType::Solution);
    }

    #[test]
    fn test_prepare_init_generates_id() {
        let opts = InitOptions::new(ItemType::UseCase).with_name("Test Use Case");

        let data = prepare_init(
            &opts,
            |t| format!("{}-001", t.prefix()),
            |id| id.to_string(),
        )
        .unwrap();

        assert_eq!(data.id, "UC-001");
        assert_eq!(data.name, "Test Use Case");
    }

    #[test]
    fn test_prepare_init_resolves_name() {
        let opts = InitOptions::new(ItemType::Scenario).with_id("SCEN-001");

        let data = prepare_init(
            &opts,
            |_| "SCEN-999".to_string(),
            |id| format!("Name for {}", id),
        )
        .unwrap();

        assert_eq!(data.id, "SCEN-001");
        assert_eq!(data.name, "Name for SCEN-001");
    }

    #[test]
    fn test_prepare_init_with_fields() {
        let fields = TypeFields::new()
            .with_status(AdrStatus::Proposed)
            .with_deciders(vec!["Alice".to_string()]);

        let opts = InitOptions::with_fields(ItemType::ArchitectureDecisionRecord, fields)
            .with_id("ADR-001")
            .with_name("Test ADR");

        let data = prepare_init(&opts, |_| String::new(), |id| id.to_string()).unwrap();

        assert_eq!(data.fields.status, Some(AdrStatus::Proposed));
        assert_eq!(data.fields.deciders, vec!["Alice".to_string()]);
    }

    #[test]
    fn test_prepare_init_validation_error() {
        let fields = TypeFields::new().with_platform("Linux"); // Invalid for Solution
        let opts = InitOptions::with_fields(ItemType::Solution, fields)
            .with_id("SOL-001")
            .with_name("Test");

        let result = prepare_init(&opts, |_| String::new(), |id| id.to_string());
        assert!(result.is_err());
    }
}

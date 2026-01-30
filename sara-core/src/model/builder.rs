//! Builder for constructing Item instances.

use crate::error::ValidationError;
use crate::graph::KnowledgeGraph;

use super::{Item, ItemAttributes, ItemId, ItemType, Relationship, SourceLocation};

/// Builder for constructing Item instances.
///
/// Used for both parsing existing items and creating new items (init).
#[derive(Debug, Default)]
pub struct ItemBuilder {
    id: Option<ItemId>,
    raw_id: Option<String>,
    item_type: Option<ItemType>,
    name: Option<String>,
    description: Option<String>,
    source: Option<SourceLocation>,
    relationships: Vec<Relationship>,
    attributes: Option<ItemAttributes>,
}

impl ItemBuilder {
    /// Creates a new ItemBuilder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the item ID (validated).
    pub fn id(mut self, id: ItemId) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets an optional raw ID string (for init where ID may be auto-generated).
    pub fn maybe_id(mut self, id: Option<String>) -> Self {
        self.raw_id = id;
        self
    }

    /// Sets the item type.
    pub fn item_type(mut self, item_type: ItemType) -> Self {
        self.item_type = Some(item_type);
        self
    }

    /// Sets the item name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets an optional name (for init where name may be derived).
    pub fn maybe_name(mut self, name: Option<String>) -> Self {
        if name.is_some() {
            self.name = name;
        }
        self
    }

    /// Sets the item description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Sets an optional description.
    pub fn maybe_description(mut self, desc: Option<String>) -> Self {
        self.description = desc;
        self
    }

    /// Sets the source location.
    pub fn source(mut self, source: SourceLocation) -> Self {
        self.source = Some(source);
        self
    }

    /// Sets the relationships.
    pub fn relationships(mut self, relationships: Vec<Relationship>) -> Self {
        self.relationships = relationships;
        self
    }

    /// Adds a single relationship.
    pub fn add_relationship(mut self, relationship: Relationship) -> Self {
        self.relationships.push(relationship);
        self
    }

    /// Sets the type-specific attributes.
    pub fn attributes(mut self, attrs: ItemAttributes) -> Self {
        self.attributes = Some(attrs);
        self
    }

    /// Returns true if the specification field is needed but not provided.
    pub fn needs_specification(&self) -> bool {
        let Some(item_type) = self.item_type else {
            return false;
        };

        if !item_type.requires_specification() {
            return false;
        }

        match &self.attributes {
            Some(ItemAttributes::SystemRequirement { specification, .. })
            | Some(ItemAttributes::SoftwareRequirement { specification, .. })
            | Some(ItemAttributes::HardwareRequirement { specification, .. }) => {
                specification.is_empty()
            }
            None => true,
            _ => false,
        }
    }

    /// Builds the Item, returning an error if required fields are missing.
    pub fn build(self) -> Result<Item, ValidationError> {
        let file_path = self
            .source
            .as_ref()
            .map(|s| s.file_path.display().to_string())
            .unwrap_or_default();

        let id = self
            .id
            .clone()
            .ok_or_else(|| ValidationError::MissingField {
                field: "id".to_string(),
                file: file_path.clone(),
            })?;

        let item_type = self
            .item_type
            .ok_or_else(|| ValidationError::MissingField {
                field: "type".to_string(),
                file: file_path.clone(),
            })?;

        let name = self
            .name
            .clone()
            .ok_or_else(|| ValidationError::MissingField {
                field: "name".to_string(),
                file: file_path.clone(),
            })?;

        let source = self
            .source
            .clone()
            .ok_or_else(|| ValidationError::MissingField {
                field: "source".to_string(),
                file: String::new(),
            })?;

        // Use provided attributes or create default for the item type
        let attributes = self
            .attributes
            .unwrap_or_else(|| ItemAttributes::for_type(item_type));

        // Validate that attributes match the item type
        Self::validate_attributes(item_type, &attributes, &file_path)?;

        Ok(Item {
            id,
            item_type,
            name,
            description: self.description,
            source,
            relationships: self.relationships,
            attributes,
        })
    }

    /// Builds the Item with resolver callbacks for optional fields.
    ///
    /// Use this for init where ID and name may need to be auto-generated.
    pub fn build_with_resolvers<F, G>(
        self,
        id_generator: F,
        name_resolver: G,
    ) -> Result<Item, ValidationError>
    where
        F: FnOnce(ItemType) -> String,
        G: FnOnce(&str) -> String,
    {
        let item_type = self
            .item_type
            .ok_or_else(|| ValidationError::MissingField {
                field: "type".to_string(),
                file: String::new(),
            })?;

        // Resolve ID: use provided ItemId, or raw_id, or generate
        let id = if let Some(id) = self.id {
            id
        } else {
            let raw = self.raw_id.unwrap_or_else(|| id_generator(item_type));
            ItemId::new_unchecked(raw)
        };

        // Resolve name: use provided or generate from ID
        let name = self.name.unwrap_or_else(|| name_resolver(id.as_str()));

        let source = self
            .source
            .clone()
            .ok_or_else(|| ValidationError::MissingField {
                field: "source".to_string(),
                file: String::new(),
            })?;

        let file_path = source.file_path.display().to_string();

        // Use provided attributes or create default for the item type
        let attributes = self
            .attributes
            .unwrap_or_else(|| ItemAttributes::for_type(item_type));

        // For init, we allow empty specification (will be filled by user)
        // Only validate type matches
        Self::validate_attributes_for_init(item_type, &attributes, &file_path)?;

        Ok(Item {
            id,
            item_type,
            name,
            description: self.description,
            source,
            relationships: self.relationships,
            attributes,
        })
    }

    /// Builds the Item for init, using the graph for ID suggestion if needed.
    ///
    /// This is a simplified version of `build_with_resolvers` that handles
    /// ID generation and name resolution automatically:
    /// - If no ID is provided, generates the next sequential ID using the graph
    /// - If no name is provided, generates a default name from the ID
    /// - Validates with init rules (allows empty specification)
    pub fn build_for_init(self, graph: Option<&KnowledgeGraph>) -> Result<Item, ValidationError> {
        let item_type = self
            .item_type
            .ok_or_else(|| ValidationError::MissingField {
                field: "type".to_string(),
                file: String::new(),
            })?;

        // Resolve ID: use provided ItemId, or raw_id, or generate from graph
        let id = if let Some(id) = self.id {
            id
        } else if let Some(raw_id) = self.raw_id {
            ItemId::new_unchecked(raw_id)
        } else {
            let generated = graph
                .map(|g| g.suggest_next_id(item_type))
                .unwrap_or_else(|| item_type.generate_id(None));
            ItemId::new_unchecked(generated)
        };

        // Resolve name: use provided or generate from ID
        let name = self
            .name
            .unwrap_or_else(|| item_type.default_name(id.as_str()));

        let source = self
            .source
            .clone()
            .ok_or_else(|| ValidationError::MissingField {
                field: "source".to_string(),
                file: String::new(),
            })?;

        let file_path = source.file_path.display().to_string();

        // Use provided attributes or create default for the item type
        let attributes = self
            .attributes
            .unwrap_or_else(|| ItemAttributes::for_type(item_type));

        // For init, we allow empty specification (will be filled by user)
        Self::validate_attributes_for_init(item_type, &attributes, &file_path)?;

        Ok(Item {
            id,
            item_type,
            name,
            description: self.description,
            source,
            relationships: self.relationships,
            attributes,
        })
    }

    /// Validates that the attributes are appropriate for the item type.
    fn validate_attributes(
        item_type: ItemType,
        attributes: &ItemAttributes,
        file: &str,
    ) -> Result<(), ValidationError> {
        match (item_type, attributes) {
            // Simple types
            (ItemType::Solution, ItemAttributes::Solution)
            | (ItemType::UseCase, ItemAttributes::UseCase)
            | (ItemType::Scenario, ItemAttributes::Scenario)
            | (ItemType::SoftwareDetailedDesign, ItemAttributes::SoftwareDetailedDesign)
            | (ItemType::HardwareDetailedDesign, ItemAttributes::HardwareDetailedDesign)
            | (ItemType::SystemArchitecture, ItemAttributes::SystemArchitecture { .. }) => Ok(()),

            // Requirement types - validate specification is not empty
            (
                ItemType::SystemRequirement,
                ItemAttributes::SystemRequirement { specification, .. },
            )
            | (
                ItemType::SoftwareRequirement,
                ItemAttributes::SoftwareRequirement { specification, .. },
            )
            | (
                ItemType::HardwareRequirement,
                ItemAttributes::HardwareRequirement { specification, .. },
            ) => {
                if specification.is_empty() {
                    return Err(ValidationError::MissingField {
                        field: "specification".to_string(),
                        file: file.to_string(),
                    });
                }
                Ok(())
            }

            // ADR - validate required fields
            (ItemType::ArchitectureDecisionRecord, ItemAttributes::Adr { deciders, .. }) => {
                if deciders.is_empty() {
                    return Err(ValidationError::MissingField {
                        field: "deciders".to_string(),
                        file: file.to_string(),
                    });
                }
                Ok(())
            }

            // Mismatched type and attributes
            _ => Err(ValidationError::MissingField {
                field: format!("attributes matching type {:?}", item_type),
                file: file.to_string(),
            }),
        }
    }

    /// Validates attributes for init (allows empty specification).
    fn validate_attributes_for_init(
        item_type: ItemType,
        attributes: &ItemAttributes,
        file: &str,
    ) -> Result<(), ValidationError> {
        match (item_type, attributes) {
            // Simple types
            (ItemType::Solution, ItemAttributes::Solution)
            | (ItemType::UseCase, ItemAttributes::UseCase)
            | (ItemType::Scenario, ItemAttributes::Scenario)
            | (ItemType::SoftwareDetailedDesign, ItemAttributes::SoftwareDetailedDesign)
            | (ItemType::HardwareDetailedDesign, ItemAttributes::HardwareDetailedDesign)
            | (ItemType::SystemArchitecture, ItemAttributes::SystemArchitecture { .. }) => Ok(()),

            // Requirement types - allow empty specification for init
            (ItemType::SystemRequirement, ItemAttributes::SystemRequirement { .. })
            | (ItemType::SoftwareRequirement, ItemAttributes::SoftwareRequirement { .. })
            | (ItemType::HardwareRequirement, ItemAttributes::HardwareRequirement { .. }) => Ok(()),

            // ADR - allow empty deciders for init (will be prompted)
            (ItemType::ArchitectureDecisionRecord, ItemAttributes::Adr { .. }) => Ok(()),

            // Mismatched type and attributes
            _ => Err(ValidationError::MissingField {
                field: format!("attributes matching type {:?}", item_type),
                file: file.to_string(),
            }),
        }
    }
}

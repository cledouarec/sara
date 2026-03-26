//! Builder for constructing `Item` instances.

use std::path::PathBuf;

use super::adr::AdrStatus;
use super::item::{Item, ItemAttributes, ItemId, ItemType};
use super::metadata::SourceLocation;
use super::relationship::Relationship;
use crate::error::SaraError;

/// Builder for constructing `Item` instances from parsed frontmatter.
#[derive(Debug, Default)]
pub struct ItemBuilder {
    id: Option<ItemId>,
    item_type: Option<ItemType>,
    name: Option<String>,
    description: Option<String>,
    source: Option<SourceLocation>,
    relationships: Vec<Relationship>,
    // Temporary storage for attributes before we know the type
    specification: Option<String>,
    platform: Option<String>,
    depends_on: Vec<ItemId>,
    status: Option<AdrStatus>,
    deciders: Vec<String>,
    supersedes: Vec<ItemId>,
}

impl ItemBuilder {
    /// Creates a new `ItemBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the item ID.
    pub fn id(mut self, id: ItemId) -> Self {
        self.id = Some(id);
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

    /// Sets the item description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Sets the source location.
    pub fn source(mut self, source: SourceLocation) -> Self {
        self.source = Some(source);
        self
    }

    /// Sets all relationships for this item.
    pub fn relationships(mut self, relationships: Vec<Relationship>) -> Self {
        self.relationships = relationships;
        self
    }

    /// Sets the specification text (for requirement types).
    pub fn specification(mut self, spec: impl Into<String>) -> Self {
        self.specification = Some(spec.into());
        self
    }

    /// Sets the platform (for `SystemArchitecture`).
    pub fn platform(mut self, platform: impl Into<String>) -> Self {
        self.platform = Some(platform.into());
        self
    }

    /// Adds a dependency (for requirement types).
    pub fn depends_on(mut self, id: ItemId) -> Self {
        self.depends_on.push(id);
        self
    }

    /// Sets the ADR status.
    pub fn status(mut self, status: AdrStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Adds a decider (for ADR).
    pub fn decider(mut self, decider: impl Into<String>) -> Self {
        self.deciders.push(decider.into());
        self
    }

    /// Sets the deciders (for ADR).
    pub fn deciders(mut self, deciders: Vec<String>) -> Self {
        self.deciders = deciders;
        self
    }

    /// Adds a superseded ADR ID.
    pub fn supersedes(mut self, id: ItemId) -> Self {
        self.supersedes.push(id);
        self
    }

    /// Sets the supersedes references (for ADR).
    pub fn supersedes_all(mut self, ids: Vec<ItemId>) -> Self {
        self.supersedes = ids;
        self
    }

    /// Sets the attributes directly.
    pub fn attributes(mut self, attrs: ItemAttributes) -> Self {
        match attrs {
            ItemAttributes::Solution
            | ItemAttributes::UseCase
            | ItemAttributes::Scenario
            | ItemAttributes::SoftwareDetailedDesign
            | ItemAttributes::HardwareDetailedDesign => {}
            ItemAttributes::SystemRequirement {
                specification,
                depends_on,
            } => {
                self.specification = Some(specification);
                self.depends_on = depends_on;
            }
            ItemAttributes::SystemArchitecture { platform } => {
                self.platform = platform;
            }
            ItemAttributes::SoftwareRequirement {
                specification,
                depends_on,
            } => {
                self.specification = Some(specification);
                self.depends_on = depends_on;
            }
            ItemAttributes::HardwareRequirement {
                specification,
                depends_on,
            } => {
                self.specification = Some(specification);
                self.depends_on = depends_on;
            }
            ItemAttributes::Adr {
                status,
                deciders,
                supersedes,
            } => {
                self.status = Some(status);
                self.deciders = deciders;
                self.supersedes = supersedes;
            }
        }
        self
    }

    /// Validates and returns the specification, returning an error if missing.
    fn require_specification(&self, file: &str) -> Result<String, SaraError> {
        self.specification
            .clone()
            .ok_or_else(|| SaraError::MissingField {
                field: "specification".to_string(),
                file: PathBuf::from(file),
            })
    }

    /// Builds the attributes for the given item type.
    fn build_attributes(
        &self,
        item_type: ItemType,
        file: &str,
    ) -> Result<ItemAttributes, SaraError> {
        match item_type {
            ItemType::Solution => Ok(ItemAttributes::Solution),
            ItemType::UseCase => Ok(ItemAttributes::UseCase),
            ItemType::Scenario => Ok(ItemAttributes::Scenario),
            ItemType::SoftwareDetailedDesign => Ok(ItemAttributes::SoftwareDetailedDesign),
            ItemType::HardwareDetailedDesign => Ok(ItemAttributes::HardwareDetailedDesign),
            ItemType::SystemArchitecture => Ok(ItemAttributes::SystemArchitecture {
                platform: self.platform.clone(),
            }),
            ItemType::SystemRequirement => Ok(ItemAttributes::SystemRequirement {
                specification: self.require_specification(file)?,
                depends_on: self.depends_on.clone(),
            }),
            ItemType::SoftwareRequirement => Ok(ItemAttributes::SoftwareRequirement {
                specification: self.require_specification(file)?,
                depends_on: self.depends_on.clone(),
            }),
            ItemType::HardwareRequirement => Ok(ItemAttributes::HardwareRequirement {
                specification: self.require_specification(file)?,
                depends_on: self.depends_on.clone(),
            }),
            ItemType::ArchitectureDecisionRecord => {
                let status = self.status.ok_or_else(|| SaraError::MissingField {
                    field: "status".to_string(),
                    file: PathBuf::from(file),
                })?;
                if self.deciders.is_empty() {
                    return Err(SaraError::MissingField {
                        field: "deciders".to_string(),
                        file: PathBuf::from(file),
                    });
                }
                Ok(ItemAttributes::Adr {
                    status,
                    deciders: self.deciders.clone(),
                    supersedes: self.supersedes.clone(),
                })
            }
        }
    }

    /// Builds the `Item`, returning an error if required fields are missing.
    ///
    /// # Errors
    ///
    /// Returns `SaraError::MissingField` if a required field (id, type,
    /// name, source, or type-specific attributes) is not set.
    pub fn build(self) -> Result<Item, SaraError> {
        let id = self.id.clone().ok_or_else(|| SaraError::MissingField {
            field: "id".to_string(),
            file: self
                .source
                .as_ref()
                .map(|s| s.file_path.clone())
                .unwrap_or_default(),
        })?;

        let item_type = self.item_type.ok_or_else(|| SaraError::MissingField {
            field: "type".to_string(),
            file: self
                .source
                .as_ref()
                .map(|s| s.file_path.clone())
                .unwrap_or_default(),
        })?;

        let name = self.name.clone().ok_or_else(|| SaraError::MissingField {
            field: "name".to_string(),
            file: self
                .source
                .as_ref()
                .map(|s| s.file_path.clone())
                .unwrap_or_default(),
        })?;

        let source = self.source.clone().ok_or_else(|| SaraError::MissingField {
            field: "source".to_string(),
            file: PathBuf::new(),
        })?;

        let file_path = source.file_path.display().to_string();
        let attributes = self.build_attributes(item_type, &file_path)?;

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::relationship::RelationshipType;

    #[test]
    fn test_item_builder() {
        let source = SourceLocation {
            repository: PathBuf::from("/repo"),
            file_path: PathBuf::from("docs/SOL-001.md"),
            git_ref: None,
        };

        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("SOL-001"))
            .item_type(ItemType::Solution)
            .name("Test Solution")
            .source(source)
            .build();

        assert!(item.is_ok());
        let item = item.unwrap();
        assert_eq!(item.id.as_str(), "SOL-001");
        assert_eq!(item.name, "Test Solution");
    }

    #[test]
    fn test_item_builder_with_relationships() {
        let source = SourceLocation {
            repository: PathBuf::from("/repo"),
            file_path: PathBuf::from("docs/UC-001.md"),
            git_ref: None,
        };

        let item = ItemBuilder::new()
            .id(ItemId::new_unchecked("UC-001"))
            .item_type(ItemType::UseCase)
            .name("Test Use Case")
            .source(source)
            .relationships(vec![Relationship::new(
                ItemId::new_unchecked("SOL-001"),
                RelationshipType::Refines,
            )])
            .build()
            .unwrap();

        let refines: Vec<_> = item.relationship_ids(RelationshipType::Refines).collect();
        assert_eq!(refines.len(), 1);
        assert_eq!(refines[0].as_str(), "SOL-001");
    }
}

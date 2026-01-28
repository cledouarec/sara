//! Builder for constructing Item instances.

#![allow(clippy::result_large_err)]

use crate::error::ValidationError;

use super::{
    AdrStatus, DownstreamRefs, Item, ItemAttributes, ItemId, ItemType, SourceLocation, UpstreamRefs,
};

/// Builder for constructing Item instances from parsed frontmatter.
#[derive(Debug, Default)]
pub struct ItemBuilder {
    id: Option<ItemId>,
    item_type: Option<ItemType>,
    name: Option<String>,
    description: Option<String>,
    source: Option<SourceLocation>,
    upstream: UpstreamRefs,
    downstream: DownstreamRefs,
    // Temporary storage for attributes before we know the type
    specification: Option<String>,
    platform: Option<String>,
    depends_on: Vec<ItemId>,
    status: Option<AdrStatus>,
    deciders: Vec<String>,
    supersedes: Vec<ItemId>,
}

impl ItemBuilder {
    /// Creates a new ItemBuilder.
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

    /// Sets the upstream references.
    pub fn upstream(mut self, upstream: UpstreamRefs) -> Self {
        self.upstream = upstream;
        self
    }

    /// Sets the downstream references.
    pub fn downstream(mut self, downstream: DownstreamRefs) -> Self {
        self.downstream = downstream;
        self
    }

    /// Sets the specification text (for requirement types).
    pub fn specification(mut self, spec: impl Into<String>) -> Self {
        self.specification = Some(spec.into());
        self
    }

    /// Sets the platform (for SystemArchitecture).
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
        // Extract values from the attributes enum
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
    fn require_specification(&self, file: &str) -> Result<String, ValidationError> {
        self.specification
            .clone()
            .ok_or_else(|| ValidationError::MissingField {
                field: "specification".to_string(),
                file: file.to_string(),
            })
    }

    /// Builds the attributes for the given item type.
    fn build_attributes(
        &self,
        item_type: ItemType,
        file: &str,
    ) -> Result<ItemAttributes, ValidationError> {
        match item_type {
            // Simple types with no additional attributes
            ItemType::Solution => Ok(ItemAttributes::Solution),
            ItemType::UseCase => Ok(ItemAttributes::UseCase),
            ItemType::Scenario => Ok(ItemAttributes::Scenario),
            ItemType::SoftwareDetailedDesign => Ok(ItemAttributes::SoftwareDetailedDesign),
            ItemType::HardwareDetailedDesign => Ok(ItemAttributes::HardwareDetailedDesign),

            // Architecture with optional platform
            ItemType::SystemArchitecture => Ok(ItemAttributes::SystemArchitecture {
                platform: self.platform.clone(),
            }),

            // Requirement types with specification and dependencies
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

            // ADR with status, deciders, and supersedes
            ItemType::ArchitectureDecisionRecord => {
                let status = self.status.ok_or_else(|| ValidationError::MissingField {
                    field: "status".to_string(),
                    file: file.to_string(),
                })?;
                if self.deciders.is_empty() {
                    return Err(ValidationError::MissingField {
                        field: "deciders".to_string(),
                        file: file.to_string(),
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

    /// Builds the Item, returning an error if required fields are missing.
    pub fn build(self) -> Result<Item, ValidationError> {
        let id = self
            .id
            .clone()
            .ok_or_else(|| ValidationError::MissingField {
                field: "id".to_string(),
                file: self
                    .source
                    .as_ref()
                    .map(|s| s.file_path.display().to_string())
                    .unwrap_or_default(),
            })?;

        let item_type = self
            .item_type
            .ok_or_else(|| ValidationError::MissingField {
                field: "type".to_string(),
                file: self
                    .source
                    .as_ref()
                    .map(|s| s.file_path.display().to_string())
                    .unwrap_or_default(),
            })?;

        let name = self
            .name
            .clone()
            .ok_or_else(|| ValidationError::MissingField {
                field: "name".to_string(),
                file: self
                    .source
                    .as_ref()
                    .map(|s| s.file_path.display().to_string())
                    .unwrap_or_default(),
            })?;

        let source = self
            .source
            .clone()
            .ok_or_else(|| ValidationError::MissingField {
                field: "source".to_string(),
                file: String::new(),
            })?;

        let file_path = source.file_path.display().to_string();
        let attributes = self.build_attributes(item_type, &file_path)?;

        Ok(Item {
            id,
            item_type,
            name,
            description: self.description,
            source,
            upstream: self.upstream,
            downstream: self.downstream,
            attributes,
        })
    }
}

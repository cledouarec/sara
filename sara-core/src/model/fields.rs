//! Type-specific fields container for item initialization and editing.

use crate::error::ValidationError;

use super::{AdrStatus, ItemType};

/// Type-agnostic container for all item fields.
///
/// This struct holds all possible type-specific fields for any item type.
/// Validation ensures only applicable fields are used per ItemType.
#[derive(Debug, Clone, Default)]
pub struct TypeFields {
    // Requirement fields
    /// Specification statement (for requirement types).
    pub specification: Option<String>,
    /// Peer dependencies (for requirement types).
    pub depends_on: Vec<String>,
    /// Items this derives from (for requirement types).
    pub derives_from: Vec<String>,

    // Architecture fields
    /// Target platform (for SystemArchitecture).
    pub platform: Option<String>,
    /// Items this satisfies (for architecture/design types).
    pub satisfies: Vec<String>,

    // Use Case / Scenario fields
    /// Items this refines (for UseCase, Scenario).
    pub refines: Vec<String>,

    // ADR fields
    /// ADR lifecycle status.
    pub status: Option<AdrStatus>,
    /// Decision makers.
    pub deciders: Vec<String>,
    /// Design artifacts this ADR justifies.
    pub justifies: Vec<String>,
    /// Older ADRs this decision supersedes.
    pub supersedes: Vec<String>,
    /// Newer ADR that supersedes this one.
    pub superseded_by: Option<String>,
}

impl TypeFields {
    /// Creates an empty TypeFields.
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate fields are appropriate for the given ItemType.
    pub fn validate_for_type(&self, item_type: ItemType) -> Result<(), ValidationError> {
        // Check specification
        if self.specification.is_some() && !item_type.requires_specification() {
            return Err(ValidationError::InvalidFieldForType {
                field: "specification".to_string(),
                item_type: item_type.display_name().to_string(),
            });
        }

        // Check platform
        if self.platform.is_some() && !item_type.accepts_platform() {
            return Err(ValidationError::InvalidFieldForType {
                field: "platform".to_string(),
                item_type: item_type.display_name().to_string(),
            });
        }

        // Check refines
        if !self.refines.is_empty() && !item_type.requires_refines() {
            return Err(ValidationError::InvalidFieldForType {
                field: "refines".to_string(),
                item_type: item_type.display_name().to_string(),
            });
        }

        // Check derives_from
        if !self.derives_from.is_empty() && !item_type.requires_derives_from() {
            return Err(ValidationError::InvalidFieldForType {
                field: "derives_from".to_string(),
                item_type: item_type.display_name().to_string(),
            });
        }

        // Check satisfies
        if !self.satisfies.is_empty() && !item_type.requires_satisfies() {
            return Err(ValidationError::InvalidFieldForType {
                field: "satisfies".to_string(),
                item_type: item_type.display_name().to_string(),
            });
        }

        // Check depends_on
        if !self.depends_on.is_empty() && !item_type.supports_depends_on() {
            return Err(ValidationError::InvalidFieldForType {
                field: "depends_on".to_string(),
                item_type: item_type.display_name().to_string(),
            });
        }

        // Check ADR-specific fields
        if self.status.is_some() && !item_type.is_adr() {
            return Err(ValidationError::InvalidFieldForType {
                field: "status".to_string(),
                item_type: item_type.display_name().to_string(),
            });
        }

        if !self.deciders.is_empty() && !item_type.is_adr() {
            return Err(ValidationError::InvalidFieldForType {
                field: "deciders".to_string(),
                item_type: item_type.display_name().to_string(),
            });
        }

        if !self.justifies.is_empty() && !item_type.is_adr() {
            return Err(ValidationError::InvalidFieldForType {
                field: "justifies".to_string(),
                item_type: item_type.display_name().to_string(),
            });
        }

        if !self.supersedes.is_empty() && !item_type.is_adr() {
            return Err(ValidationError::InvalidFieldForType {
                field: "supersedes".to_string(),
                item_type: item_type.display_name().to_string(),
            });
        }

        if self.superseded_by.is_some() && !item_type.is_adr() {
            return Err(ValidationError::InvalidFieldForType {
                field: "superseded_by".to_string(),
                item_type: item_type.display_name().to_string(),
            });
        }

        Ok(())
    }

    /// Check if specification is needed but not provided.
    pub fn needs_specification(&self, item_type: ItemType) -> bool {
        item_type.requires_specification() && self.specification.is_none()
    }

    /// Returns true if all fields are empty/None.
    pub fn is_empty(&self) -> bool {
        self.specification.is_none()
            && self.depends_on.is_empty()
            && self.derives_from.is_empty()
            && self.platform.is_none()
            && self.satisfies.is_empty()
            && self.refines.is_empty()
            && self.status.is_none()
            && self.deciders.is_empty()
            && self.justifies.is_empty()
            && self.supersedes.is_empty()
            && self.superseded_by.is_none()
    }

    // Builder-style methods

    /// Sets the specification.
    pub fn with_specification(mut self, spec: impl Into<String>) -> Self {
        self.specification = Some(spec.into());
        self
    }

    /// Sets the platform.
    pub fn with_platform(mut self, platform: impl Into<String>) -> Self {
        self.platform = Some(platform.into());
        self
    }

    /// Sets the refines references.
    pub fn with_refines(mut self, refines: Vec<String>) -> Self {
        self.refines = refines;
        self
    }

    /// Sets the derives_from references.
    pub fn with_derives_from(mut self, derives_from: Vec<String>) -> Self {
        self.derives_from = derives_from;
        self
    }

    /// Sets the satisfies references.
    pub fn with_satisfies(mut self, satisfies: Vec<String>) -> Self {
        self.satisfies = satisfies;
        self
    }

    /// Sets the depends_on references.
    pub fn with_depends_on(mut self, depends_on: Vec<String>) -> Self {
        self.depends_on = depends_on;
        self
    }

    /// Sets the ADR status.
    pub fn with_status(mut self, status: AdrStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Sets the deciders.
    pub fn with_deciders(mut self, deciders: Vec<String>) -> Self {
        self.deciders = deciders;
        self
    }

    /// Sets the justifies references.
    pub fn with_justifies(mut self, justifies: Vec<String>) -> Self {
        self.justifies = justifies;
        self
    }

    /// Sets the supersedes references.
    pub fn with_supersedes(mut self, supersedes: Vec<String>) -> Self {
        self.supersedes = supersedes;
        self
    }

    /// Sets the superseded_by reference.
    pub fn with_superseded_by(mut self, superseded_by: impl Into<String>) -> Self {
        self.superseded_by = Some(superseded_by.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_fields() {
        let fields = TypeFields::new();
        assert!(fields.is_empty());
    }

    #[test]
    fn test_validate_specification_for_requirement() {
        let fields = TypeFields::new().with_specification("Must do X");
        assert!(
            fields
                .validate_for_type(ItemType::SystemRequirement)
                .is_ok()
        );
        assert!(
            fields
                .validate_for_type(ItemType::SoftwareRequirement)
                .is_ok()
        );
        assert!(
            fields
                .validate_for_type(ItemType::HardwareRequirement)
                .is_ok()
        );
    }

    #[test]
    fn test_validate_specification_for_non_requirement() {
        let fields = TypeFields::new().with_specification("Must do X");
        assert!(fields.validate_for_type(ItemType::Solution).is_err());
        assert!(fields.validate_for_type(ItemType::UseCase).is_err());
    }

    #[test]
    fn test_validate_platform_for_architecture() {
        let fields = TypeFields::new().with_platform("Linux x86_64");
        assert!(
            fields
                .validate_for_type(ItemType::SystemArchitecture)
                .is_ok()
        );
    }

    #[test]
    fn test_validate_platform_for_non_architecture() {
        let fields = TypeFields::new().with_platform("Linux x86_64");
        assert!(fields.validate_for_type(ItemType::Solution).is_err());
        assert!(
            fields
                .validate_for_type(ItemType::SystemRequirement)
                .is_err()
        );
    }

    #[test]
    fn test_validate_adr_fields() {
        let fields = TypeFields::new()
            .with_status(AdrStatus::Proposed)
            .with_deciders(vec!["Alice".to_string()]);
        assert!(
            fields
                .validate_for_type(ItemType::ArchitectureDecisionRecord)
                .is_ok()
        );
    }

    #[test]
    fn test_validate_adr_fields_for_non_adr() {
        let fields = TypeFields::new().with_status(AdrStatus::Proposed);
        assert!(fields.validate_for_type(ItemType::Solution).is_err());
    }

    #[test]
    fn test_needs_specification() {
        let empty = TypeFields::new();
        assert!(empty.needs_specification(ItemType::SystemRequirement));
        assert!(!empty.needs_specification(ItemType::Solution));

        let with_spec = TypeFields::new().with_specification("spec");
        assert!(!with_spec.needs_specification(ItemType::SystemRequirement));
    }
}

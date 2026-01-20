//! Field name definitions for YAML frontmatter.
//!
//! This module provides a single source of truth for all field names
//! used in YAML frontmatter serialization and deserialization.

/// All field names used in YAML frontmatter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldName {
    // Required fields
    Id,
    Type,
    Name,

    // Optional metadata
    Description,

    // Upstream traceability (relationships)
    Refines,
    DerivesFrom,
    Satisfies,

    // Downstream traceability (relationships)
    IsRefinedBy,
    Derives,
    IsSatisfiedBy,

    // Peer relationships
    DependsOn,
    IsRequiredBy,

    // Type-specific attributes
    Specification,
    Platform,
    JustifiedBy,
}

impl FieldName {
    /// Returns all known field names.
    pub const fn all() -> &'static [FieldName] {
        &[
            Self::Id,
            Self::Type,
            Self::Name,
            Self::Description,
            Self::Refines,
            Self::DerivesFrom,
            Self::Satisfies,
            Self::IsRefinedBy,
            Self::Derives,
            Self::IsSatisfiedBy,
            Self::DependsOn,
            Self::IsRequiredBy,
            Self::Specification,
            Self::Platform,
            Self::JustifiedBy,
        ]
    }

    /// Returns the YAML field name (snake_case).
    ///
    /// Used for serialization, deserialization, and error messages.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::Type => "type",
            Self::Name => "name",
            Self::Description => "description",
            Self::Refines => "refines",
            Self::DerivesFrom => "derives_from",
            Self::Satisfies => "satisfies",
            Self::IsRefinedBy => "is_refined_by",
            Self::Derives => "derives",
            Self::IsSatisfiedBy => "is_satisfied_by",
            Self::DependsOn => "depends_on",
            Self::IsRequiredBy => "is_required_by",
            Self::Specification => "specification",
            Self::Platform => "platform",
            Self::JustifiedBy => "justified_by",
        }
    }

    /// Returns the human-readable display name.
    ///
    /// Used for user-facing output like change summaries.
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Id => "ID",
            Self::Type => "Type",
            Self::Name => "Name",
            Self::Description => "Description",
            Self::Refines => "Refines",
            Self::DerivesFrom => "Derives from",
            Self::Satisfies => "Satisfies",
            Self::IsRefinedBy => "Is refined by",
            Self::Derives => "Derives",
            Self::IsSatisfiedBy => "Is satisfied by",
            Self::DependsOn => "Depends on",
            Self::IsRequiredBy => "Is required by",
            Self::Specification => "Specification",
            Self::Platform => "Platform",
            Self::JustifiedBy => "Justified by",
        }
    }

    /// Returns true if this is an upstream traceability field.
    pub const fn is_upstream(&self) -> bool {
        matches!(self, Self::Refines | Self::DerivesFrom | Self::Satisfies)
    }

    /// Returns true if this is a downstream traceability field.
    pub const fn is_downstream(&self) -> bool {
        matches!(
            self,
            Self::IsRefinedBy | Self::Derives | Self::IsSatisfiedBy
        )
    }

    /// Returns true if this is a peer relationship field.
    pub const fn is_peer(&self) -> bool {
        matches!(self, Self::DependsOn | Self::IsRequiredBy)
    }

    /// Returns true if this is a traceability field (upstream, downstream, or peer).
    pub const fn is_traceability(&self) -> bool {
        self.is_upstream() || self.is_downstream() || self.is_peer()
    }
}

impl std::fmt::Display for FieldName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_name_as_str() {
        assert_eq!(FieldName::Id.as_str(), "id");
        assert_eq!(FieldName::DerivesFrom.as_str(), "derives_from");
        assert_eq!(FieldName::Specification.as_str(), "specification");
    }

    #[test]
    fn test_field_name_is_upstream() {
        assert!(FieldName::Refines.is_upstream());
        assert!(FieldName::DerivesFrom.is_upstream());
        assert!(FieldName::Satisfies.is_upstream());
        assert!(!FieldName::IsRefinedBy.is_upstream());
        assert!(!FieldName::Specification.is_upstream());
    }

    #[test]
    fn test_field_name_is_downstream() {
        assert!(FieldName::IsRefinedBy.is_downstream());
        assert!(FieldName::Derives.is_downstream());
        assert!(FieldName::IsSatisfiedBy.is_downstream());
        assert!(!FieldName::Refines.is_downstream());
    }

    #[test]
    fn test_field_name_all() {
        let all = FieldName::all();
        assert!(all.contains(&FieldName::Id));
        assert!(all.contains(&FieldName::Refines));
        assert!(all.contains(&FieldName::Specification));
        assert_eq!(all.len(), 15);
    }

    #[test]
    fn test_field_name_display() {
        assert_eq!(format!("{}", FieldName::DerivesFrom), "derives_from");
    }
}

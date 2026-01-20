//! Error types for the sara-core library.

use std::path::PathBuf;
use thiserror::Error;

use crate::model::{ItemId, ItemType, RelationshipType, SourceLocation};

/// Errors that can occur during parsing.
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Failed to read file {path}: {reason}")]
    FileRead { path: PathBuf, reason: String },

    #[error("Invalid frontmatter in {file}: {reason}")]
    InvalidFrontmatter { file: PathBuf, reason: String },

    #[error("Missing frontmatter in {file}")]
    MissingFrontmatter { file: PathBuf },

    #[error("Invalid YAML in {file}: {reason}")]
    InvalidYaml { file: PathBuf, reason: String },

    #[error("Missing required field '{field}' in {file}")]
    MissingField { file: PathBuf, field: String },

    #[error("Invalid item type '{value}' in {file}")]
    InvalidItemType { file: PathBuf, value: String },
}

/// Errors that can occur during validation.
#[derive(Debug, Error, Clone, serde::Serialize)]
pub enum ValidationError {
    #[error("Invalid item ID '{id}': {reason}")]
    InvalidId { id: String, reason: String },

    #[error("Missing required field '{field}' in {file}")]
    MissingField { field: String, file: String },

    #[error("Broken reference: {from} references non-existent item {to}")]
    BrokenReference {
        from: ItemId,
        to: ItemId,
        location: Option<SourceLocation>,
    },

    #[error("Orphan item: {id} has no upstream parent")]
    OrphanItem {
        id: ItemId,
        item_type: ItemType,
        location: Option<SourceLocation>,
    },

    #[error("Duplicate identifier: {id} defined in multiple files")]
    DuplicateIdentifier {
        id: ItemId,
        locations: Vec<SourceLocation>,
    },

    #[error("Circular reference detected: {cycle}")]
    CircularReference {
        cycle: String,
        location: Option<SourceLocation>,
    },

    #[error("Invalid relationship: {from_type} cannot {rel_type} {to_type}")]
    InvalidRelationship {
        from_id: ItemId,
        to_id: ItemId,
        from_type: ItemType,
        to_type: ItemType,
        rel_type: RelationshipType,
        location: Option<SourceLocation>,
    },

    #[error("Invalid metadata in {file}: {reason}")]
    InvalidMetadata { file: String, reason: String },

    #[error("Unrecognized field '{field}' in {file}")]
    UnrecognizedField {
        field: String,
        file: String,
        location: Option<SourceLocation>,
    },

    #[error(
        "Redundant relationship: {from_id} and {to_id} both declare the relationship (only one is needed)"
    )]
    RedundantRelationship {
        from_id: ItemId,
        to_id: ItemId,
        from_rel: RelationshipType,
        to_rel: RelationshipType,
        from_location: Option<SourceLocation>,
        to_location: Option<SourceLocation>,
    },
}

impl ValidationError {
    /// Returns the source location if available.
    pub fn location(&self) -> Option<&SourceLocation> {
        match self {
            Self::BrokenReference { location, .. } => location.as_ref(),
            Self::OrphanItem { location, .. } => location.as_ref(),
            Self::DuplicateIdentifier { locations, .. } => locations.first(),
            Self::CircularReference { location, .. } => location.as_ref(),
            Self::InvalidRelationship { location, .. } => location.as_ref(),
            Self::UnrecognizedField { location, .. } => location.as_ref(),
            Self::RedundantRelationship { from_location, .. } => from_location.as_ref(),
            _ => None,
        }
    }

    /// Returns true if this is an error (blocks validation).
    pub fn is_error(&self) -> bool {
        !matches!(
            self,
            Self::UnrecognizedField { .. } | Self::RedundantRelationship { .. }
        )
    }

    /// Returns the error code for this validation error.
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidId { .. } => "invalid_id",
            Self::MissingField { .. } => "missing_field",
            Self::BrokenReference { .. } => "broken_reference",
            Self::OrphanItem { .. } => "orphan_item",
            Self::DuplicateIdentifier { .. } => "duplicate_identifier",
            Self::CircularReference { .. } => "circular_reference",
            Self::InvalidRelationship { .. } => "invalid_relationship",
            Self::InvalidMetadata { .. } => "invalid_metadata",
            Self::UnrecognizedField { .. } => "unrecognized_field",
            Self::RedundantRelationship { .. } => "redundant_relationship",
        }
    }
}

/// Errors that can occur with configuration.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config file {path}: {reason}")]
    FileRead { path: PathBuf, reason: String },

    #[error("Invalid config file {path}: {reason}")]
    InvalidConfig { path: PathBuf, reason: String },

    #[error("Repository not found: {path}")]
    RepositoryNotFound { path: PathBuf },

    #[error("Invalid glob pattern '{pattern}': {reason}")]
    InvalidGlobPattern { pattern: String, reason: String },
}

/// Errors that can occur during queries.
#[derive(Debug, Error)]
pub enum QueryError {
    #[error("Item not found: {id}")]
    ItemNotFound {
        id: String,
        suggestions: Vec<String>,
    },

    #[error("Invalid query: {reason}")]
    InvalidQuery { reason: String },
}

/// Errors that can occur with Git operations.
#[derive(Debug, Error)]
pub enum GitError {
    #[error("Failed to open repository {path}: {reason}")]
    OpenRepository { path: PathBuf, reason: String },

    #[error("Invalid Git reference: {reference}")]
    InvalidReference { reference: String },

    #[error("Failed to read file {path} at {reference}: {reason}")]
    ReadFile {
        path: PathBuf,
        reference: String,
        reason: String,
    },
}

/// Main error type for sara-core.
#[derive(Debug, Error)]
pub enum SaraError {
    #[error(transparent)]
    Parse(#[from] ParseError),

    #[error(transparent)]
    Validation(Box<ValidationError>),

    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error(transparent)]
    Query(#[from] QueryError),

    #[error(transparent)]
    Git(#[from] GitError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Git operation failed: {0}")]
    GitError(String),
}

impl From<ValidationError> for SaraError {
    fn from(err: ValidationError) -> Self {
        SaraError::Validation(Box::new(err))
    }
}

/// Errors that can occur during edit operations (FR-054 through FR-066).
#[derive(Debug, Error)]
pub enum EditError {
    #[error("Item not found: {id}")]
    ItemNotFound {
        id: String,
        suggestions: Vec<String>,
    },

    #[error(
        "Interactive mode requires a terminal. Use modification flags (--name, --description, etc.) to edit non-interactively."
    )]
    NonInteractiveTerminal,

    #[error("User cancelled")]
    Cancelled,

    #[error("Invalid traceability link: {id} does not exist")]
    InvalidLink { id: String },

    #[error("Failed to read file: {0}")]
    IoError(String),

    #[error("Failed to parse graph: {0}")]
    GraphError(String),
}

impl EditError {
    /// Format suggestions for "not found" error (FR-061).
    pub fn format_suggestions(&self) -> Option<String> {
        if let EditError::ItemNotFound { suggestions, .. } = self
            && !suggestions.is_empty()
        {
            return Some(format!("Did you mean: {}?", suggestions.join(", ")));
        }
        None
    }

    /// Returns true if this error has suggestions.
    pub fn has_suggestions(&self) -> bool {
        matches!(self, EditError::ItemNotFound { suggestions, .. } if !suggestions.is_empty())
    }
}

/// Result type for sara-core operations.
pub type Result<T> = std::result::Result<T, SaraError>;

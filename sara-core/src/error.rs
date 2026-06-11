//! Error types for the sara-core library.
//!
//! This module defines a unified error type for all SARA operations.
//! All errors are consolidated into the single [`SaraError`] enum with
//! clear variants for each error category.
//!
//! # Error Categories
//!
//! - **Parsing**: Markdown and YAML frontmatter parsing
//! - **Validation**: Graph structure and item validation
//! - **Configuration**: Loading and validating configuration
//! - **Queries**: Item lookup and graph traversal
//! - **Git Operations**: Repository access and version control
//! - **Editing**: Item modification operations
//!
//! # Examples
//!
//! ```
//! use sara_core::error::SaraError;
//! use std::path::PathBuf;
//!
//! # fn example() -> Result<(), SaraError> {
//! // Validation errors use explicit variants
//! let err = SaraError::BrokenReference {
//!     from: sara_core::model::ItemId::new_unchecked("UC-001"),
//!     to: sara_core::model::ItemId::new_unchecked("SOL-999"),
//! };
//!
//! // File operations with context
//! let err = SaraError::InvalidFrontmatter {
//!     file: PathBuf::from("doc.md"),
//!     reason: "Missing required 'id' field".to_string(),
//! };
//! # Ok(())
//! # }
//! ```

use std::path::PathBuf;

use thiserror::Error;

use crate::model::{ItemId, ItemType, RelationshipType};

/// Main error type for sara-core operations.
///
/// Consolidates all error categories into a single type with clear variants.
/// Uses `thiserror` for automatic `Display` and `Error` trait implementations.
///
/// # Errors
///
/// This enum categorizes all possible errors that can occur during SARA operations.
/// Each variant includes contextual information to help diagnose the issue.
///
/// # Examples
///
/// ```
/// use sara_core::error::SaraError;
/// use std::path::PathBuf;
///
/// # fn example() -> Result<(), SaraError> {
/// // Missing item with fuzzy-match suggestions
/// let err = SaraError::ItemNotFound {
///     id: "SOL-999".to_string(),
///     suggestions: vec!["SOL-001".to_string()],
/// };
///
/// // Parse error with context
/// let err = SaraError::InvalidFrontmatter {
///     file: PathBuf::from("doc.md"),
///     reason: "Missing 'id' field".to_string(),
/// };
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Error, serde::Serialize)]
#[serde(tag = "error_type", content = "details")]
pub enum SaraError {
    // ==================== Parsing ====================
    /// Invalid frontmatter in a Markdown file.
    #[error("Invalid frontmatter in {file}: {reason}")]
    InvalidFrontmatter {
        /// Path to the file with invalid frontmatter.
        file: PathBuf,
        /// Description of what's wrong with the frontmatter.
        reason: String,
    },

    /// File is missing required frontmatter section.
    #[error("Missing frontmatter in {file}")]
    MissingFrontmatter {
        /// Path to the file without frontmatter.
        file: PathBuf,
    },

    /// Invalid YAML syntax in frontmatter.
    #[error("Invalid YAML in {file}: {reason}")]
    InvalidYaml {
        /// Path to the file with invalid YAML.
        file: PathBuf,
        /// YAML parsing error details.
        reason: String,
    },

    /// Missing required field in frontmatter.
    #[error("Missing required field '{field}' in {file}")]
    MissingField {
        /// The field that was missing.
        field: String,
        /// Path to the file.
        file: PathBuf,
    },

    // ==================== Validation ====================
    /// Invalid item ID format.
    #[error("Invalid item ID '{id}': {reason}")]
    InvalidId {
        /// The invalid ID.
        id: String,
        /// Why the ID is invalid.
        reason: String,
    },

    /// Broken reference to non-existent item.
    #[error("Broken reference: {from} references non-existent item {to}")]
    BrokenReference {
        /// The item with the broken reference.
        from: ItemId,
        /// The non-existent item being referenced.
        to: ItemId,
    },

    /// Orphan item with no upstream parent.
    #[error("Orphan item: {id} ({item_type}) has no upstream parent")]
    OrphanItem {
        /// The orphaned item ID.
        id: ItemId,
        /// The item type.
        item_type: ItemType,
    },

    /// Duplicate identifier found in multiple files.
    #[error("Duplicate identifier: {id} defined in multiple files")]
    DuplicateIdentifier {
        /// The duplicated ID.
        id: ItemId,
    },

    /// Circular reference detected in the graph.
    #[error("Circular reference detected: {cycle}")]
    CircularReference {
        /// Description of the cycle.
        cycle: String,
    },

    /// Invalid relationship between item types.
    #[error("Invalid relationship: {from_id} ({from_type}) cannot {rel_type} {to_id} ({to_type})")]
    InvalidRelationship {
        /// Source item ID.
        from_id: ItemId,
        /// Target item ID.
        to_id: ItemId,
        /// Source item type.
        from_type: ItemType,
        /// Target item type.
        to_type: ItemType,
        /// Relationship type attempted.
        rel_type: RelationshipType,
    },

    /// Invalid metadata in item.
    #[error("Invalid metadata in {file}: {reason}")]
    InvalidMetadata {
        /// File containing the invalid metadata.
        file: String,
        /// Description of the metadata issue.
        reason: String,
    },

    /// Redundant relationship declared on both sides.
    #[error(
        "Redundant relationship: {from_id} and {to_id} both declare the relationship (only one is needed)"
    )]
    RedundantRelationship {
        /// First item ID.
        from_id: ItemId,
        /// Second item ID.
        to_id: ItemId,
    },

    // ==================== Configuration ====================
    /// Configuration file could not be read.
    #[error("Failed to read config file {path}: {reason}")]
    ConfigRead {
        /// Path to the config file.
        path: PathBuf,
        /// Reason for the failure.
        reason: String,
    },

    /// Configuration file has invalid content.
    #[error("Invalid config file {path}: {reason}")]
    InvalidConfig {
        /// Path to the config file.
        path: PathBuf,
        /// Description of the configuration error.
        reason: String,
    },

    // ==================== Queries ====================
    /// No parent items exist for the given item type.
    #[error(
        "Cannot create {item_type}: no {parent_type} items exist. Create a {parent_type} first."
    )]
    MissingParent {
        /// The item type that requires a parent.
        item_type: String,
        /// The parent type that is missing.
        parent_type: String,
    },

    /// Item was not found in the knowledge graph.
    #[error("Item not found: {id}")]
    ItemNotFound {
        /// The item ID that wasn't found.
        id: String,
        /// Suggested similar item IDs (fuzzy matches).
        suggestions: Vec<String>,
    },

    // ==================== Git Operations ====================
    /// Generic Git operation error.
    #[error("Git operation failed: {0}")]
    Git(String),

    // ==================== Edit Operations ====================
    /// Interactive terminal required but not available.
    #[error(
        "Interactive mode requires a terminal. Use modification flags (--name, --description, etc.) to edit non-interactively."
    )]
    NonInteractiveTerminal,

    /// Edit operation failed with custom error message.
    #[error("Edit failed: {0}")]
    EditFailed(String),

    // ==================== Wrapped Errors ====================
    /// Standard I/O error.
    #[error("I/O error: {0}")]
    Io(
        #[serde(skip)]
        #[from]
        std::io::Error,
    ),

    /// Gitoxide (gix) library error.
    ///
    /// Boxed because gix exposes a distinct error type per operation; a single
    /// boxed variant keeps the enum stable across gix minor bumps while still
    /// preserving `Display` and `source()` for tracing and user reports.
    #[error("Git error: {0}")]
    Gix(#[serde(skip)] Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl SaraError {
    /// Formats suggestions as a user-friendly message.
    ///
    /// Returns `None` if this is not an `ItemNotFound` error or if there are no suggestions.
    pub fn format_suggestions(&self) -> Option<String> {
        match self {
            Self::ItemNotFound { suggestions, .. } if !suggestions.is_empty() => {
                Some(format!("Did you mean: {}?", suggestions.join(", ")))
            }
            _ => None,
        }
    }
}

/// Result type for sara-core operations.
///
/// This is a convenience alias for `Result<T, SaraError>`.
///
/// # Examples
///
/// ```
/// use sara_core::error::Result;
///
/// fn parse_file() -> Result<String> {
///     Ok("parsed content".to_string())
/// }
/// ```
pub type Result<T> = std::result::Result<T, SaraError>;

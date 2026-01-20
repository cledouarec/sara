//! Sara Core - Requirements Knowledge Graph Library
//!
//! This library provides the core functionality for managing architecture documents
//! and requirements as a unified interconnected knowledge graph.

pub mod config;
pub mod diff;
pub mod edit;
pub mod error;
pub mod graph;
pub mod init;
pub mod model;
pub mod parser;
pub mod query;
pub mod report;
pub mod repository;
pub mod template;
pub mod validation;

// Re-export commonly used types
pub use config::Config;
pub use diff::{DiffError, DiffOptions, DiffResult, DiffService};
pub use edit::{
    EditOptions as CoreEditOptions, EditResult as CoreEditResult, EditService, EditedValues,
    ItemContext,
};
pub use error::{EditError, ParseError, Result, SaraError, ValidationError};
pub use graph::{
    GraphBuilder, GraphDiff, GraphStats, KnowledgeGraph, TraversalOptions, TraversalResult,
    traverse_downstream, traverse_upstream,
};
pub use init::{InitError, InitOptions, InitResult, InitService, parse_item_type};
pub use model::{
    EditSummary, EditUpdates, FieldChange, FieldName, Item, ItemId, ItemType, RelationshipType,
    SourceLocation, TraceabilityConfig, TraceabilityLinks, UpstreamRefs,
};
pub use parser::{extract_body, parse_document, parse_markdown_file, update_frontmatter};
pub use query::{
    LookupResult, MissingParentError, QueryEngine, check_parent_exists, find_similar_ids,
    lookup_item_or_suggest,
};
pub use report::{CoverageReport, TraceabilityMatrix};
pub use repository::{parse_directory, parse_repositories};
pub use template::{
    GeneratorOptions, extract_name_from_content, generate_document, generate_frontmatter,
    generate_id, suggest_next_id,
};
pub use validation::{ValidationReport, Validator, validate};

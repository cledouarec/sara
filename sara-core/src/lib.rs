//! Sara Core - Requirements Knowledge Graph Library
//!
//! This library provides the core functionality for managing architecture documents
//! and requirements as a unified interconnected knowledge graph.
//!
//! # Architecture
//!
//! - `model/` - Domain types (Item, ItemType, ItemAttributes, etc.)
//! - `graph/` - Knowledge graph operations
//! - `validation/` - Validation rules
//! - `query/` - Query operations
//! - `parser/` - Input adapters (YAML, Markdown parsing)
//! - `generator/` - Output adapters (YAML, Markdown generation)
//! - `service/` - File I/O services (init, edit, diff)
//! - `report/` - Report generation

pub mod config;
pub mod error;
pub mod generator;
pub mod graph;
pub mod model;
pub mod parser;
pub mod query;
pub mod report;
pub mod repository;
pub mod service;
pub mod validation;

#[cfg(test)]
pub mod test_utils;

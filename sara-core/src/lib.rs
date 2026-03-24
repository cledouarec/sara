//! Sara Core - Requirements Knowledge Graph Library
//!
//! This library provides the core functionality for managing architecture documents
//! and requirements as a unified interconnected knowledge graph.

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

//! Report generation modules.

mod coverage;
mod matrix;

pub use coverage::{CoverageReport, IncompleteItem, TypeCoverage};
pub use matrix::{MatrixRow, MatrixTarget, TraceabilityMatrix};

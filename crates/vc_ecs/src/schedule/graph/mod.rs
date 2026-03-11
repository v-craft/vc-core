mod dag;
mod graphs;
mod reduction;
mod scc;
mod toposort;

// -----------------------------------------------------------------------------
// Exports

pub use dag::{Dag, DagAnalysis, DagGroups};
pub use graphs::{DiGraph, Direction, Graph, GraphNode, UnGraph};
pub use scc::{SccIterator, SccNodes};
pub use toposort::ToposortError;

// -----------------------------------------------------------------------------
// Helper

/// Converts 2D row-major pair of indices into a 1D array index.
fn flatten_index(row: usize, col: usize, num_cols: usize) -> usize {
    debug_assert!(col < num_cols);
    (row * num_cols) + col
}

/// Converts a 1D array index into a 2D row-major pair of indices.
fn unflatten_index(index: usize, num_cols: usize) -> (usize, usize) {
    (index / num_cols, index % num_cols)
}

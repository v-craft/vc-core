//! Re-exports [`fastvec`]'s containers.
//!
//! It's a high-performance vector crate tuned for small data sizes.

// -----------------------------------------------------------------------------
// Stack Only

pub use fastvec::{StackVec, stack_vec};

// -----------------------------------------------------------------------------
// Data Process

pub use fastvec::{FastVec, fast_vec};

// -----------------------------------------------------------------------------
// Data Storage

pub use fastvec::{AutoVec, auto_vec};

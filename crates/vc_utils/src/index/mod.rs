//! Re-exports *[indexmap]*, provide newtypes based on fixed hash state.

pub use indexmap::{Equivalent, GetDisjointMutError, TryReserveError};

pub mod map;
pub mod set;

pub use map::IndexMap;
pub use set::IndexSet;

// pub use map::SparseIndexMap;
// pub use set::SparseIndexSet;

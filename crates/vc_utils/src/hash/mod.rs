//! Provide hash containers, re-exports *[hashbrown]* and *[foldhash]*.

// -----------------------------------------------------------------------------
// Modules

mod hasher;

pub mod hash_map;
pub mod hash_set;
pub mod hash_table;

// -----------------------------------------------------------------------------
// Exports

pub use hasher::{FixedHashState, FixedHasher};
pub use hasher::{NoOpHashState, NoOpHasher};
pub use hasher::{SparseHashState, SparseHasher};

pub use hash_map::{HashMap, NoOpHashMap, SparseHashMap};
pub use hash_set::{HashSet, NoOpHashSet, SparseHashSet};
pub use hash_table::HashTable;

pub use hashbrown::Equivalent;

// -----------------------------------------------------------------------------
// Re-export crates

pub use foldhash;
pub use hashbrown;

//! Provide hash containers, re-exports *hashbrown* and *foldhash*.

// -----------------------------------------------------------------------------
// Modules

mod hasher;
mod pre_hashed;

pub mod hash_map;
pub mod hash_set;
pub mod hash_table;

// -----------------------------------------------------------------------------
// Exports

pub use hasher::{FixedHashState, FixedHasher};
pub use hasher::{NoOpHashState, NoOpHasher};

pub use pre_hashed::{Hashed, PreHashMap};

pub use hash_map::HashMap;
pub use hash_set::HashSet;
pub use hash_table::HashTable;

// -----------------------------------------------------------------------------
// Re-export crates

pub use foldhash;
pub use hashbrown;

//! Re-export [`HashTable`] from [hashbrown] crate.

use hashbrown::hash_table as hb;

pub use hb::HashTable;

pub use hb::{AbsentEntry, Entry, OccupiedEntry, VacantEntry};
pub use hb::{Drain, ExtractIf, IntoIter};
pub use hb::{Iter, IterHash, IterHashMut, IterMut};

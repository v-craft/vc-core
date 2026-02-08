#![allow(clippy::new_without_default, reason = "internal function")]

use core::fmt::Debug;

use super::{NoSends, Resources, SparseSets, Tables};

// -----------------------------------------------------------------------------
// Storages

pub struct Storages {
    pub(crate) tables: Tables,
    pub(crate) sparse_sets: SparseSets,
    pub(crate) resources: Resources,
    pub(crate) no_sends: NoSends,
}

impl Storages {
    pub(crate) fn new() -> Storages {
        Storages {
            tables: Tables::new(),
            sparse_sets: SparseSets::new(),
            resources: Resources::new(),
            no_sends: NoSends::new(),
        }
    }
}

impl Debug for Storages {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Storages")
            .field("tables", &self.tables)
            .field("sparse_sets", &self.sparse_sets)
            .field("resources", &self.resources)
            .field("no_sends", &self.no_sends)
            .finish()
    }
}

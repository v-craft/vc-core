#![expect(unsafe_code, reason = "original implementation requires unsafe code.")]

// -----------------------------------------------------------------------------
// Modules

mod index;
mod resource;
mod sparse;
mod table;
mod utils;

// -----------------------------------------------------------------------------
// Internal API

use utils::{AbortOnPanic, BlobArray, Column, VecCopyRemove, VecSwapRemove};

// -----------------------------------------------------------------------------
// Exports

pub use index::{StorageIndex, StorageType};
pub use resource::{NoSendResourceData, NoSendResources, ResourceData, Resources};
pub use sparse::{SparseComponent, SparseSet, SparseSets};
pub use table::{Table, TableBuilder, TableId, TableMoveResult, TableRow, Tables};

// -----------------------------------------------------------------------------
// Inline-Exports

pub struct Storages {
    pub sparse_sets: SparseSets,
    pub tables: Tables,
    pub resources: Resources,
    pub non_send_resources: NoSendResources,
}

impl Storages {
    #[inline]
    pub fn prepare_component(&mut self, info: &crate::component::ComponentInfo) {
        match info.storage_type() {
            StorageType::SparseSet => {
                self.sparse_sets.prepare_component(info);
            }
            StorageType::Table => {
                // table needs no preparation
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Modules

mod blob_array;
mod blob_box;
mod column;
mod index;
mod tick_array;
mod utils;

mod indices;
mod resource;
mod sparse;
mod storages;
mod table;

// -----------------------------------------------------------------------------
// Internal

// use utils::{VecCopyRemove, VecSwapRemove};
use blob_array::BlobArray;
use blob_box::BlobBox;
use column::Column;
use indices::ComponentIndices;
use tick_array::TickArray;
use utils::{AbortOnDropFail, AbortOnPanic, VecRemoveExt};

// -----------------------------------------------------------------------------
// Exports

pub use index::{StorageIndex, StorageType};
pub use resource::{NonSendData, NonSends, ResourceData, Resources};
pub use sparse::{SparseComponent, SparseSets};
pub use storages::Storages;
pub use table::{Table, TableId, TableRow, Tables};
pub use table::{TableMoveResult, TableRemoveResult};

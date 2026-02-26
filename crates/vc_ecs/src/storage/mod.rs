// -----------------------------------------------------------------------------
// Modules

mod column;
mod dense;
mod global;
mod impls;
mod sparse;
mod utils;

// -----------------------------------------------------------------------------
// Internal

use utils::{AbortOnPanic, VecRemoveExt};

// -----------------------------------------------------------------------------
// Exports

pub use column::Column;
pub use dense::{Table, TableMoveResult, Tables};
pub use dense::{TableCol, TableId, TableRow};
pub use global::{ResData, ResSet};
pub use impls::Storages;
pub use sparse::{Map, Maps};
pub use sparse::{MapId, MapRow};

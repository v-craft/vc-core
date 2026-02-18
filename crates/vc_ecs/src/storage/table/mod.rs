// -----------------------------------------------------------------------------
// Module

mod data;
mod id;
mod tables;

use data::TableBuilder;

// -----------------------------------------------------------------------------
// Exports

pub use data::{Table, TableMoveResult, TableRemoveResult};
pub use id::{TableId, TableRow};
pub use tables::Tables;

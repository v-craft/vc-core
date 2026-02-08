// -----------------------------------------------------------------------------
// Module

mod ident;
mod table;
mod tables;

// -----------------------------------------------------------------------------
// Internal

use table::TableBuilder;

// -----------------------------------------------------------------------------
// Exports

pub use ident::{TableCol, TableId, TableRow};
pub use table::Table;
pub use table::TableMoveResult;
pub use tables::Tables;

// -----------------------------------------------------------------------------
// Modules

mod access;
mod ident;
mod impls;

// -----------------------------------------------------------------------------
// Exports

pub use ident::{WorldId, WorldIdAllocator};
pub use impls::World;

pub use access::AccessTable;
pub use access::{EntityMut, EntityRef};
pub use access::{UnsafeWorld, WorldMode};

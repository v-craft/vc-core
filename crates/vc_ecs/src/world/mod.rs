// -----------------------------------------------------------------------------
// Modules

mod access;
mod ident;
mod impls;
mod methods;

// -----------------------------------------------------------------------------
// Exports

pub use access::*;
pub use ident::{WorldId, WorldIdAllocator};
pub use impls::World;

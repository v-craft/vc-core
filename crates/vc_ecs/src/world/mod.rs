// -----------------------------------------------------------------------------
// Modules

mod access;
mod ident;
mod methods;
mod world;

// -----------------------------------------------------------------------------
// Exports

pub use access::*;
pub use ident::{WorldId, WorldIdAllocator};
pub use world::World;

// -----------------------------------------------------------------------------
// Modules

mod allocator;
mod ident;
mod info;

// -----------------------------------------------------------------------------
// Internal

// -----------------------------------------------------------------------------
// Exports

pub use allocator::{EntityAllocator, RemoteAllocator};
pub use ident::{Entity, EntityGeneration, EntityId};
pub use info::{Entities, EntityInfo, EntityLocation};

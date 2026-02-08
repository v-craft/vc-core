// -----------------------------------------------------------------------------
// Modules

mod allocator;
mod error;
mod ident;
mod info;

// -----------------------------------------------------------------------------
// Exports

pub use allocator::{AllocEntitiesIter, EntityAllocator, RemoteAllocator};
pub use error::*;
pub use ident::{Entity, EntityGeneration, EntityId};
pub use info::{Entities, EntityLocation, MovedEntity};

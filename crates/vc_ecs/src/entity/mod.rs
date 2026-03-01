// -----------------------------------------------------------------------------
// Modules

mod allocator;
mod error;
mod ident;
mod info;
mod mapper;
mod storage;

// -----------------------------------------------------------------------------
// Exports

pub use allocator::{AllocEntitiesIter, EntityAllocator, RemoteAllocator};
pub use error::*;
pub use ident::{Entity, EntityGeneration, EntityId};
pub use info::{Entities, EntityLocation, MovedEntity};
pub use mapper::{EntityHashMap, EntityMapper};
pub use storage::StorageId;

//! Entity

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
pub use info::{Entities, EntityLocation, MovedEntityRow};
pub use mapper::{EntityMap, EntityMapper};
pub use storage::StorageId;

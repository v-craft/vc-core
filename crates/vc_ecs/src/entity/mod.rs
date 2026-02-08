// -----------------------------------------------------------------------------
// Modules

mod id;

mod allocator;
mod clone;
mod entities;
mod entity;
mod location;
mod remote_allocator;
mod utils;

pub mod error;

// -----------------------------------------------------------------------------
// Exports

pub use utils::*;

pub use allocator::EntityAllocator;
pub use clone::ComponentCloneCtx;
pub use entities::Entities;
pub use entity::Entity;
pub use id::{EntityGeneration, EntityId};
pub use location::EntityLocation;

// -----------------------------------------------------------------------------
// Modules

mod id;
mod world;

mod deferred;
mod error;
mod world_cell;

mod entity_access;

// -----------------------------------------------------------------------------
// Exports

pub use id::WorldId;
pub use world::World;

pub use deferred::DeferredWorld;
pub use world_cell::{UnsafeEntityCell, UnsafeWorldCell};

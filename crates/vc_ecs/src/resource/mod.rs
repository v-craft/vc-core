//! Global resources unrelated to entities.

// -----------------------------------------------------------------------------
// Modules

mod ident;
mod impls;
mod info;
mod resources;

// -----------------------------------------------------------------------------
// Exports

pub use vc_ecs_derive::Resource;

pub use ident::ResourceId;
pub use impls::Resource;
pub use info::{ResourceDescriptor, ResourceInfo};
pub use resources::Resources;

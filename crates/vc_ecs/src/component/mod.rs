//! Component - Entity specific data.

// -----------------------------------------------------------------------------
// Modules

mod components;
mod ident;
mod impls;
mod info;
mod required;
mod storage;
mod tools;

// -----------------------------------------------------------------------------
// Exports

pub use vc_ecs_derive::Component;

pub use components::Components;
pub use ident::ComponentId;
pub use impls::Component;
pub use info::{ComponentDescriptor, ComponentInfo};
pub use required::{Required, RequiredComponents};
pub use storage::ComponentStorage;
pub use tools::*;

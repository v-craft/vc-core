// -----------------------------------------------------------------------------
// Modules

mod components;
mod ident;
mod impls;
mod info;
mod storage;

// -----------------------------------------------------------------------------
// Exports

pub use components::Components;
pub use ident::ComponentId;
pub use impls::{Component, ComponentCollector, ComponentRegistrar, ComponentWriter};
pub use info::{ComponentDescriptor, ComponentInfo};
pub use storage::ComponentStorage;

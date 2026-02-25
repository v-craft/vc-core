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
pub use impls::{ComponentCollector, ComponentRegistrar, CollectResult};
pub use impls::{Component, ComponentWriter};
pub use info::{ComponentDescriptor, ComponentInfo};
pub use storage::ComponentStorage;

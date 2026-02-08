// -----------------------------------------------------------------------------
// Modules

mod id;

mod borrow;
mod tick;

mod clone;
mod components;
mod info;
mod mutable;
mod register;
mod required;

// -----------------------------------------------------------------------------
// Internal API

use crate::relationship::RelationshipAccessor;
use crate::storage::StorageType;

pub(crate) use tick::{ComponentTicksMut, ComponentTicksRef};

// -----------------------------------------------------------------------------
// Exports

pub use id::{ComponentId, ComponentIdAllocator, ComponentIndices};

pub use borrow::{Mut, MutUntyped, Ref};
pub use borrow::{NonSend, NonSendMut, Res, ResMut};
pub use clone::{ComponentCloneBehavior, ComponentCloneFn, SourceComponent};
pub use components::Components;
pub use info::{ComponentDescriptor, ComponentInfo};
pub use mutable::{ComponentMutability, Immutable, Mutable};
pub use register::{ComponentsRegistrator, QueuedComponents, QueuedRegistration};
pub use required::{RequiredComponent, RequiredComponents};
pub use required::{RequiredComponentsError, RequiredComponentsRegistrator};
pub use tick::{ComponentTickCells, ComponentTicks};

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum InsertMode {
    /// Any existing components of a matching type will be overwritten.
    Replace,
    /// Any existing components of a matching type will be left unchanged.
    Keep,
}

// -----------------------------------------------------------------------------
// TODO

use crate::lifecycle::ComponentHook;

pub trait Component: Send + Sync + 'static {
    const STORAGE_TYPE: StorageType;
    type Mutability: ComponentMutability;

    /// Gets the `on_add` [`ComponentHook`] for this [`Component`] if one is defined.
    fn on_add() -> Option<ComponentHook> {
        None
    }

    /// Gets the `on_insert` [`ComponentHook`] for this [`Component`] if one is defined.
    fn on_insert() -> Option<ComponentHook> {
        None
    }

    /// Gets the `on_replace` [`ComponentHook`] for this [`Component`] if one is defined.
    fn on_replace() -> Option<ComponentHook> {
        None
    }

    /// Gets the `on_remove` [`ComponentHook`] for this [`Component`] if one is defined.
    fn on_remove() -> Option<ComponentHook> {
        None
    }

    /// Gets the `on_despawn` [`ComponentHook`] for this [`Component`] if one is defined.
    fn on_despawn() -> Option<ComponentHook> {
        None
    }

    #[inline]
    fn register_required_components(
        _id: ComponentId,
        _registrator: &mut RequiredComponentsRegistrator,
    ) {
    }

    #[inline]
    fn clone_behavior() -> ComponentCloneBehavior {
        ComponentCloneBehavior::Default
    }

    #[inline]
    fn relationship_accessor() -> Option<RelationshipAccessor> {
        None
    }

    #[inline]
    fn map_entities<E: crate::entity::EntityMapper>(_this: &mut Self, _mapper: &mut E) {}
}

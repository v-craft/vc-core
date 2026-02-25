#![allow(clippy::len_without_is_empty, reason = "internal type")]

use alloc::vec::Vec;
use core::any::TypeId;
use core::fmt::Debug;

use vc_utils::extra::TypeIdMap;

use crate::component::ComponentRegistrar;

use super::{Component, ComponentDescriptor};
use super::{ComponentId, ComponentInfo};

// -----------------------------------------------------------------------------
// Components

/// A registry that manages all component types in the ECS world.
pub struct Components {
    infos: Vec<ComponentInfo>,
    mapper: TypeIdMap<ComponentId>,
}

impl Debug for Components {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.infos, f)
    }
}

impl Components {
    /// Creates a new empty component registry.
    #[inline]
    pub(crate) const fn new() -> Self {
        Self {
            infos: Vec::new(),
            mapper: TypeIdMap::new(),
        }
    }

    /// Returns the number of registered components.
    #[inline]
    pub const fn len(&self) -> usize {
        self.infos.len()
    }

    /// Looks up a component ID by its [`TypeId`].
    #[inline]
    pub fn get_id(&self, type_id: TypeId) -> Option<ComponentId> {
        self.mapper.get(&type_id).copied()
    }

    /// Returns the component info for the given ID.
    #[inline]
    pub fn get(&self, id: ComponentId) -> Option<&ComponentInfo> {
        self.infos.get(id.index())
    }

    /// Returns a mutable reference to the component info for the given ID.
    #[inline]
    pub fn get_mut(&mut self, id: ComponentId) -> Option<&mut ComponentInfo> {
        self.infos.get_mut(id.index())
    }

    /// Returns the component info for the given ID without bounds checking.
    ///
    /// # Safety
    /// The caller must ensure `id` is a valid ID (i.e., `id.index() < self.len()`).
    #[inline]
    pub unsafe fn get_unchecked(&self, id: ComponentId) -> &ComponentInfo {
        debug_assert!(id.index() < self.infos.len());
        unsafe { self.infos.get_unchecked(id.index()) }
    }

    /// Returns a mutable reference to the component info for the given ID
    /// without bounds checking.
    ///
    /// # Safety
    /// The caller must ensure `id` is a valid ID (i.e., `id.index() < self.len()`).
    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, id: ComponentId) -> &mut ComponentInfo {
        debug_assert!(id.index() < self.infos.len());
        unsafe { self.infos.get_unchecked_mut(id.index()) }
    }

    /// Registers a component type `T` and returns its unique ID.
    ///
    /// If the component type is already registered, this returns the existing ID.
    /// Otherwise, it creates a new descriptor, assigns a new ID, and stores the metadata.
    #[inline]
    pub fn register<T: Component>(&mut self) -> ComponentId {
        #[cold]
        #[inline(never)]
        fn register_internal(
            this: &mut Components,
            required: unsafe fn(&mut ComponentRegistrar),
            descriptor: fn() -> ComponentDescriptor,
        ) -> ComponentId {
            let id = ComponentId::new(this.infos.len() as u32);
            let descriptor = descriptor();
            let type_id = descriptor.type_id;

            this.infos.push(ComponentInfo::new(id, descriptor));
            this.mapper.insert(type_id, id);

            unsafe {
                required(&mut ComponentRegistrar { components: this });
            }

            id
        }

        if let Some(id) = self.get_id(TypeId::of::<T>()) {
            id
        } else {
            register_internal(self, T::register_required, ComponentDescriptor::new::<T>)
        }
    }
}

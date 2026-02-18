#![allow(clippy::new_without_default, reason = "internal function")]

use alloc::vec::Vec;
use core::alloc::Layout;
use core::any::TypeId;
use core::fmt::Debug;

use vc_ptr::OwningPtr;
use vc_utils::extra::TypeIdMap;

use crate::clone::CloneBehavior;
use crate::component::{Component, ComponentId, NonSendResource, Resource};
use crate::storage::StorageType;
use crate::utils::{DebugCheckedUnwrap, DebugName};

// -----------------------------------------------------------------------------
// ComponentKind

#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    Component,
    Resource,
    NonSendResource,
}

impl PartialEq for ComponentKind {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        *self as u8 == *other as u8
    }
}

impl Eq for ComponentKind {}

// -----------------------------------------------------------------------------
// ComponentDescriptor

#[derive(Debug, Clone)]
pub struct ComponentDescriptor {
    pub(crate) debug_name: DebugName,
    pub(crate) type_id: TypeId,
    pub(crate) layout: Layout,
    pub(crate) kind: ComponentKind,
    pub(crate) mutable: bool,
    pub(crate) storage_type: StorageType,
    pub(crate) drop_fn: Option<unsafe fn(OwningPtr<'_>)>,
    pub(crate) clone_behavior: CloneBehavior,
    // TODO: relationship
}

impl ComponentDescriptor {
    /// # Safety
    /// type correct
    unsafe fn debug_checked_drop_as<T>(ptr: OwningPtr<'_>) {
        ptr.debug_assert_aligned::<T>();
        unsafe {
            ptr.drop_as::<T>();
        }
    }

    const fn drop_fn_for<T>() -> Option<unsafe fn(OwningPtr<'_>)> {
        if core::mem::needs_drop::<T>() {
            Some(Self::debug_checked_drop_as::<T>)
        } else {
            None
        }
    }

    pub const fn new_component<T: Component>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            layout: Layout::new::<T>(),
            kind: ComponentKind::Component,
            mutable: T::MUTABLE,
            storage_type: T::STORAGE_TYPE,
            clone_behavior: T::CLONE_BEHAVIOR,
            drop_fn: Self::drop_fn_for::<T>(),
            debug_name: DebugName::type_name::<T>(),
        }
    }

    pub const fn new_resource<T: Resource>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            layout: Layout::new::<T>(),
            kind: ComponentKind::Resource,
            mutable: T::MUTABLE,
            storage_type: StorageType::SparseSet, // unused
            clone_behavior: T::CLONE_BEHAVIOR,
            drop_fn: Self::drop_fn_for::<T>(),
            debug_name: DebugName::type_name::<T>(),
        }
    }

    pub const fn new_non_send<T: NonSendResource>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            layout: Layout::new::<T>(),
            kind: ComponentKind::NonSendResource,
            mutable: T::MUTABLE,
            storage_type: StorageType::SparseSet, // unused
            clone_behavior: T::CLONE_BEHAVIOR,
            drop_fn: Self::drop_fn_for::<T>(),
            debug_name: DebugName::type_name::<T>(),
        }
    }
}

// -----------------------------------------------------------------------------
// ComponentInfo

pub struct ComponentInfo {
    id: ComponentId,
    descriptor: ComponentDescriptor,
    // TODO: required
}

impl Debug for ComponentInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ComponentInfo")
            .field("id", &self.id)
            .field("kind", &self.descriptor.kind)
            .field("name", &self.descriptor.debug_name)
            .field("storage", &self.descriptor.storage_type)
            .field("mutable", &self.descriptor.mutable)
            .finish()
    }
}

impl ComponentInfo {
    #[inline]
    pub(crate) fn new(id: ComponentId, descriptor: ComponentDescriptor) -> Self {
        Self { id, descriptor }
    }

    #[inline(always)]
    pub fn id(&self) -> ComponentId {
        self.id
    }

    #[inline(always)]
    pub fn debug_name(&self) -> DebugName {
        self.descriptor.debug_name
    }

    #[inline(always)]
    pub fn type_id(&self) -> TypeId {
        self.descriptor.type_id
    }

    #[inline(always)]
    pub fn layout(&self) -> Layout {
        self.descriptor.layout
    }

    #[inline(always)]
    pub fn kind(&self) -> ComponentKind {
        self.descriptor.kind
    }

    #[inline(always)]
    pub fn mutable(&self) -> bool {
        self.descriptor.mutable
    }

    #[inline(always)]
    pub fn storage_type(&self) -> StorageType {
        self.descriptor.storage_type
    }

    #[inline(always)]
    pub fn drop_fn(&self) -> Option<unsafe fn(OwningPtr<'_>)> {
        self.descriptor.drop_fn
    }

    #[inline(always)]
    pub fn clone_behavior(&self) -> CloneBehavior {
        self.descriptor.clone_behavior
    }
}

// -----------------------------------------------------------------------------

pub struct Components {
    pub(crate) infos: Vec<Option<ComponentInfo>>,
    pub(crate) components: TypeIdMap<ComponentId>,
    pub(crate) resources: TypeIdMap<ComponentId>,
    pub(crate) non_sends: TypeIdMap<ComponentId>,
}

impl Debug for Components {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.infos, f)
    }
}

impl Components {
    pub(crate) const fn new() -> Self {
        Self {
            infos: Vec::new(),
            components: TypeIdMap::new(),
            resources: TypeIdMap::new(),
            non_sends: TypeIdMap::new(),
        }
    }

    /// # Safety
    /// The target must already exist.
    #[inline]
    pub unsafe fn get(&self, id: ComponentId) -> &ComponentInfo {
        // SAFETY: The caller ensures `id` is valid.
        unsafe {
            self.infos
                .get_unchecked(id.index())
                .as_ref()
                .debug_checked_unwrap()
        }
    }

    #[inline]
    pub fn try_get(&self, id: ComponentId) -> Option<&ComponentInfo> {
        self.infos.get(id.index()).and_then(|v| v.as_ref())
    }

    #[inline]
    pub fn get_component_id(&self, type_id: TypeId) -> Option<ComponentId> {
        self.components.get(&type_id).copied()
    }

    #[inline]
    pub fn get_resource_id(&self, type_id: TypeId) -> Option<ComponentId> {
        self.resources.get(&type_id).copied()
    }

    #[inline]
    pub fn get_non_send_id(&self, type_id: TypeId) -> Option<ComponentId> {
        self.non_sends.get(&type_id).copied()
    }

    pub fn contains_component(&self, type_id: TypeId) -> bool {
        self.components.contains(&type_id)
    }

    pub fn contains_resource(&self, type_id: TypeId) -> bool {
        self.resources.contains(&type_id)
    }

    pub fn contains_non_send(&self, type_id: TypeId) -> bool {
        self.non_sends.contains(&type_id)
    }
}

// -----------------------------------------------------------------------------
// Tests

#[cfg(test)]
mod tests {
    use super::ComponentKind;

    #[test]
    fn kind_eq() {
        assert_eq!(ComponentKind::Component, ComponentKind::Component);
        assert_eq!(ComponentKind::Resource, ComponentKind::Resource);
        assert_eq!(
            ComponentKind::NonSendResource,
            ComponentKind::NonSendResource
        );

        assert_ne!(ComponentKind::Component, ComponentKind::Resource);
        assert_ne!(ComponentKind::Resource, ComponentKind::NonSendResource);
        assert_ne!(ComponentKind::NonSendResource, ComponentKind::Component);
    }
}

use core::alloc::Layout;
use core::any::TypeId;
use core::fmt::Debug;

use vc_ptr::OwningPtr;

use super::{Component, ComponentId, ComponentStorage};
use crate::clone::CloneBehavior;
use crate::utils::DebugName;

// -----------------------------------------------------------------------------
// ComponentDescriptor

/// Metadata describing a resource type.
///
/// This descriptor contains all static information about a resource type,
/// including its name, type ID, memory layout, and behavior flags.
#[derive(Debug, Clone)]
pub struct ComponentDescriptor {
    pub name: DebugName,
    pub type_id: TypeId,
    pub layout: Layout,
    pub mutable: bool,
    pub storage: ComponentStorage,
    pub drop_fn: Option<unsafe fn(OwningPtr<'_>)>,
    pub clone_behavior: CloneBehavior,
}

impl ComponentDescriptor {
    /// Creates a new descriptor for resource type `T`.
    pub const fn new<T: Component>() -> Self {
        const {
            Self {
                name: DebugName::type_name::<T>(),
                type_id: TypeId::of::<T>(),
                layout: Layout::new::<T>(),
                mutable: T::MUTABLE,
                storage: T::STORAGE,
                clone_behavior: T::CLONE_BEHAVIOR,
                drop_fn: OwningPtr::drop_fn_of::<T>(),
            }
        }
    }
}

// -----------------------------------------------------------------------------
// ComponentInfo

/// Runtime information for a registered resource.
///
/// Combines a unique [`ComponentId`] with its static [`ComponentDescriptor`].
pub struct ComponentInfo {
    id: ComponentId,
    descriptor: ComponentDescriptor,
}

impl Debug for ComponentInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Component")
            .field("id", &self.id)
            .field("name", &self.descriptor.name)
            .field("storage", &self.descriptor.storage)
            .field("mutable", &self.descriptor.mutable)
            .finish()
    }
}

impl ComponentInfo {
    /// Creates a new resource info with given ID and descriptor.
    #[inline]
    pub(crate) fn new(id: ComponentId, descriptor: ComponentDescriptor) -> Self {
        Self { id, descriptor }
    }

    /// Returns the resource's unique ID.
    #[inline(always)]
    pub fn id(&self) -> ComponentId {
        self.id
    }

    /// Returns the resource's debug name.
    #[inline(always)]
    pub fn debug_name(&self) -> DebugName {
        self.descriptor.name
    }

    /// Returns the resource's [`TypeId`].
    #[inline(always)]
    pub fn type_id(&self) -> TypeId {
        self.descriptor.type_id
    }

    /// Returns the resource's memory layout.
    #[inline(always)]
    pub fn layout(&self) -> Layout {
        self.descriptor.layout
    }

    /// Returns whether the resource can be mutated.
    #[inline(always)]
    pub fn mutable(&self) -> bool {
        self.descriptor.mutable
    }

    /// Returns the resource's storage strategy.
    #[inline(always)]
    pub fn storage(&self) -> ComponentStorage {
        self.descriptor.storage
    }

    /// Returns the function that drops this resource, if any.
    #[inline(always)]
    pub fn drop_fn(&self) -> Option<unsafe fn(OwningPtr<'_>)> {
        self.descriptor.drop_fn
    }

    /// Returns the resource's cloning behavior.
    #[inline(always)]
    pub fn clone_behavior(&self) -> CloneBehavior {
        self.descriptor.clone_behavior
    }
}

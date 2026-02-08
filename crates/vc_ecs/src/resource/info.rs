use core::alloc::Layout;
use core::any::TypeId;
use core::fmt::Debug;

use vc_ptr::OwningPtr;

use super::{Resource, ResourceId};
use crate::clone::CloneBehavior;
use crate::utils::DebugName;

// -----------------------------------------------------------------------------
// ResourceDescriptor

/// Metadata describing a resource type.
///
/// This descriptor contains all static information about a resource type,
/// including its name, type ID, memory layout, and behavior flags.
#[derive(Debug, Clone)]
pub struct ResourceDescriptor {
    pub name: DebugName,
    pub type_id: TypeId,
    pub layout: Layout,
    pub mutable: bool,
    pub is_send: bool,
    pub drop_fn: Option<unsafe fn(OwningPtr<'_>)>,
    pub clone_behavior: CloneBehavior,
}

impl ResourceDescriptor {
    /// Creates a new descriptor for resource type `T`.
    pub const fn new<T: Resource>() -> Self {
        const {
            Self {
                name: DebugName::type_name::<T>(),
                type_id: TypeId::of::<T>(),
                layout: Layout::new::<T>(),
                mutable: T::MUTABLE,
                is_send: T::IS_SEND,
                clone_behavior: T::CLONE_BEHAVIOR,
                drop_fn: OwningPtr::drop_fn_of::<T>(),
            }
        }
    }
}

// -----------------------------------------------------------------------------
// ResourceInfo

/// Runtime information for a registered resource.
///
/// Combines a unique [`ResourceId`] with its static [`ResourceDescriptor`].
pub struct ResourceInfo {
    id: ResourceId,
    descriptor: ResourceDescriptor,
}

impl Debug for ResourceInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Resource")
            .field("id", &self.id)
            .field("name", &self.descriptor.name)
            .field("is_send", &self.descriptor.is_send)
            .field("mutable", &self.descriptor.mutable)
            .finish()
    }
}

impl ResourceInfo {
    /// Creates a new resource info with given ID and descriptor.
    #[inline(always)]
    pub(crate) fn new(id: ResourceId, descriptor: ResourceDescriptor) -> Self {
        Self { id, descriptor }
    }

    /// Returns the resource's unique ID.
    #[inline(always)]
    pub fn id(&self) -> ResourceId {
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
    pub fn is_send(&self) -> bool {
        self.descriptor.is_send
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

// -----------------------------------------------------------------------------
// Modules

mod ident;
mod info;
mod register;
mod tick;

// -----------------------------------------------------------------------------
// Internal API

pub(crate) use ident::CompIdAllocator;
pub(crate) use tick::{ComponentTicksMut, ComponentTicksRef};
pub(crate) use tick::{ComponentTicksSliceMut, ComponentTicksSliceRef};

// -----------------------------------------------------------------------------
// Exports

pub use ident::ComponentId;
pub use info::{ComponentDescriptor, ComponentInfo, ComponentKind, Components};
pub use tick::{ComponentTickCells, ComponentTicks};

// -----------------------------------------------------------------------------
// Component

use crate::clone::CloneBehavior;
use crate::storage::StorageType;

pub trait Resource: Sized + Send + Sync + 'static {
    const MUTABLE: bool = true;
    const CLONE_BEHAVIOR: CloneBehavior = CloneBehavior::Refuse;
}

pub trait NoSendResource: Sized + 'static {
    const MUTABLE: bool = true;
    const CLONE_BEHAVIOR: CloneBehavior = CloneBehavior::Refuse;
}

pub trait Component: Sized + Send + Sync + 'static {
    const STORAGE_TYPE: StorageType;
    const MUTABLE: bool = true;
    const CLONE_BEHAVIOR: CloneBehavior = CloneBehavior::Refuse;
}

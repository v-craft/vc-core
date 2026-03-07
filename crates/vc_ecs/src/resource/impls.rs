use crate::utils::{Cloner, Dropper};

/// Marker trait for values stored in a world's resource storage.
///
/// A resource is a singleton value identified by its concrete Rust type. At most
/// one value of a given resource type can exist in a [`crate::world::World`].
/// Thread-safety determines which access APIs are available:
///
/// - `Send + Sync` resources can be accessed through [`crate::borrow::Res`],
///   [`crate::borrow::ResRef`], and [`crate::borrow::ResMut`].
/// - `!Sync` resources must stay on the main thread and are accessed through
///   [`crate::borrow::NonSend`], [`crate::borrow::NonSendRef`], and
///   [`crate::borrow::NonSendMut`].
///
/// # Safety
///
/// Implementing this trait promises that the type can be stored behind the ECS'
/// type-erased resource storage. If you override [`Self::CLONER`] or
/// [`Self::DROPPER`], they must match the implementor's actual layout and drop
/// behavior.
pub unsafe trait Resource: Sized + 'static {
    const MUTABLE: bool = true;
    const CLONER: Option<Cloner> = None;
    const DROPPER: Option<Dropper> = Dropper::of::<Self>();
}

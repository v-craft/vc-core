//! Core component trait definition for the entity-component system.
//!
//! This module defines the fundamental `Component` trait that all components
//! must implement, along with associated configuration constants that control
//! component behavior within the system.

use super::{ComponentStorage, Required};
use crate::entity::EntityMapper;
use crate::utils::{Cloner, Dropper};

// -----------------------------------------------------------------------------
// Component

/// The core trait for all components in the entity-component system.
///
/// This trait must be implemented for any type that can be used as a component.
/// It provides essential metadata about the component's behavior, including
/// mutability, storage strategy, cloning behavior, and required dependencies.
///
/// It provides essential metadata about the component's behavior, including
/// mutability, storage strategy, cloning behavior, and required dependencies.
///
/// # Safety
///
/// This trait is unsafe because incorrect implementations can lead to:
/// - Memory unsafety in component storage and access
/// - Violation of thread safety guarantees
/// - Incorrect component versioning and tick tracking
/// - Undefined behavior in component cloning and mutation
pub unsafe trait Component: Sized + Send + Sync + 'static {
    const STORAGE: ComponentStorage = ComponentStorage::Dense;
    const MUTABLE: bool = true;
    const DROPPER: Option<Dropper> = Dropper::of::<Self>();
    const CLONER: Option<Cloner> = None;
    const REQUIRED: Option<Required> = None;

    /// Maps the entities on this component using the given [`EntityMapper`].
    ///
    /// This is used to remap entities in contexts like scenes and entity cloning.
    #[inline(always)]
    #[allow(unused_variables, reason = "default implementation")]
    fn map_entities<E: EntityMapper>(this: &mut Self, mapper: &mut E) {}
}

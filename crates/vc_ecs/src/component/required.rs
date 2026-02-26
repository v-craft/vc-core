//! Required components handling for the component system.
//!
//! This module provides a trait-based system for managing component dependencies
//! and requirements. It allows components to specify their dependencies through
//! tuple composition, enabling automatic registration, collection, and writing
//! of required components.

use vc_utils::range_invoke;

use crate::component::{Component, ComponentWriter};
use crate::component::{ComponentCollector, ComponentRegistrar};

// -----------------------------------------------------------------------------
// Required

/// A v-table that stores the function pointers for RequiredComponents.
#[derive(Debug, Clone, Copy)]
pub struct Required {
    pub(crate) register: fn(&mut ComponentRegistrar),
    pub(crate) collect: fn(&mut ComponentCollector),
    pub(crate) write: unsafe fn(&mut ComponentWriter),
}

impl Required {
    /// Create A `Required` from specific params.
    pub const fn from<T: RequiredComponents>() -> Self {
        Self {
            register: T::required_register,
            collect: T::required_collect,
            write: T::required_write,
        }
    }
}

/// A trait for types that have required components.
///
/// This trait defines the operations needed to manage component dependencies:
/// registration, collection, and writing. It is implemented for tuples of
/// components, allowing complex dependency trees to be expressed through
/// composition.
///
/// # Safety
///
/// This trait is unsafe because incorrect implementations could lead to:
/// - Missing component registrations
/// - Invalid component writes
/// - Memory unsafety in the component system
///
/// Implementations must ensure that all required components are properly
/// registered, collected, and written.
pub unsafe trait RequiredComponents {
    /// Registers all required components with the given registrar.
    ///
    /// The order is not required, and duplicate registrations are allowed.
    fn required_register(registrar: &mut ComponentRegistrar);

    /// Collects all required components using the given collector.
    ///
    /// The order is not required, and duplicate collection are allowed.
    fn required_collect(collector: &mut ComponentCollector);

    /// Writes all required components using the given writer.
    ///
    /// # Safety
    /// This function is unsafe because:
    /// - It may write to memory locations that must be valid
    /// - The writer's internal state must be properly initialized
    unsafe fn required_write(writer: &mut ComponentWriter);
}

unsafe impl RequiredComponents for () {
    fn required_register(_registrar: &mut ComponentRegistrar) {}
    fn required_collect(_collector: &mut ComponentCollector) {}
    unsafe fn required_write(_writer: &mut ComponentWriter) {}
}

unsafe impl<T: Component + Default> RequiredComponents for T {
    fn required_register(registrar: &mut ComponentRegistrar) {
        registrar.register::<T>();
    }

    fn required_collect(collector: &mut ComponentCollector) {
        collector.collect::<T>();
    }

    unsafe fn required_write(writer: &mut ComponentWriter) {
        unsafe {
            writer.write_required::<T>(T::default);
        }
    }
}

macro_rules! impl_required_for_tuple {
    (0: []) => {};
    (1 : [ $index:tt : $name:ident ]) => {
        #[cfg_attr(docsrs, doc(fake_variadic))]
        #[cfg_attr(docsrs, doc = "This trait is implemented for tuples up to 15 items long.")]
        unsafe impl<$name: RequiredComponents> RequiredComponents for ($name,) {
            fn required_register(registrar: &mut ComponentRegistrar) {
                <$name>::required_register(registrar);
            }

            fn required_collect(collector: &mut ComponentCollector) {
                <$name>::required_collect(collector);
            }

            unsafe fn required_write(writer: &mut ComponentWriter) {
                unsafe { <$name>::required_write(writer); }
            }
        }
    };
    ($num:literal : [$($index:tt : $name:ident),*]) => {
        #[cfg_attr(docsrs, doc(hidden))]
        unsafe impl<$($name: RequiredComponents),*> RequiredComponents for ( $($name),* ) {
            fn required_register(registrar: &mut ComponentRegistrar) {
                $( <$name>::required_register(registrar); )*
            }

            fn required_collect(collector: &mut ComponentCollector) {
                $( <$name>::required_collect(collector); )*
            }

            unsafe fn required_write(writer: &mut ComponentWriter) {
                $( unsafe { <$name>::required_write(writer); } )*
            }
        }
    };
}

range_invoke!(impl_required_for_tuple,  15: P);

#![allow(clippy::missing_safety_doc, reason = "todo")]

use vc_utils::range_invoke;

use crate::component::{Component, ComponentCollector, ComponentWriter};

/// A trait for types that can be used as bundles of components.
///
/// Bundles allow grouping multiple components together for efficient
/// insertion and spawning. They can be implemented manually for custom
/// bundle types or derived automatically for tuples of components.
///
/// # Safety
///
/// Implementing this trait requires careful handling of memory layout and
/// component registration. The implementor must ensure:
///
/// - Components are registered in the correct order during `register_components`.
/// - Component data is written at correct offsets during `write_components`.
pub unsafe trait Bundle: Sized + Sync + Send + 'static {
    unsafe fn collect_components(registrar: &mut ComponentCollector);
    unsafe fn write_fields(writer: &mut ComponentWriter, base: usize);
    unsafe fn write_required(writer: &mut ComponentWriter);
}

/// Automatic implementation of [`Bundle`] for any single component.
///
/// This allows using individual component types directly as bundles for
/// convenience when spawning entities with only one component.
unsafe impl<T: Component> Bundle for T {
    unsafe fn collect_components(registrar: &mut ComponentCollector) {
        registrar.collect::<T>();
    }

    unsafe fn write_fields(writer: &mut ComponentWriter, base: usize) {
        unsafe {
            writer.write_field::<T>(base);
        }
    }

    unsafe fn write_required(writer: &mut ComponentWriter) {
        unsafe {
            T::write_required(writer);
        }
    }
}

macro_rules! impl_bundle_for_tuple {
    (0: []) => {
        unsafe impl Bundle for () {
            unsafe fn collect_components(_registrar: &mut ComponentCollector) {}
            unsafe fn write_fields( _writer: &mut ComponentWriter, _base: usize,) {}
            unsafe fn write_required(_writer: &mut ComponentWriter) {}
        }
    };
    (1 : [ $index:tt : $name:ident ]) => {
        #[cfg_attr(docsrs, doc(fake_variadic))]
        unsafe impl<$name: Bundle> Bundle for ($name,) {
            unsafe fn collect_components(registrar: &mut ComponentCollector) {
                unsafe { <$name>::collect_components(registrar) }
            }

            unsafe fn write_fields(writer: &mut ComponentWriter, base: usize) {
                const { assert!(::core::mem::offset_of!(Self, 0) == 0); }
                unsafe { <$name>::write_fields(writer, base); }
            }

            unsafe fn write_required(writer: &mut ComponentWriter) {
                const { assert!(::core::mem::offset_of!(Self, 0) == 0); }
                unsafe { <$name>::write_required(writer); }
            }
        }
    };
    ($num:literal : [$($index:tt : $name:ident),*]) => {
        #[cfg_attr(docsrs, doc(hidden))]
        unsafe impl<$($name: Bundle),*> Bundle for ($($name,)*) {
            unsafe fn collect_components(registrar: &mut ComponentCollector) {
                $( unsafe { <$name>::collect_components(registrar); } )*
            }

            unsafe fn write_fields(writer: &mut ComponentWriter, base: usize) {
                $(unsafe {
                    let offset = ::core::mem::offset_of!(Self, $index) + base;
                    <$name>::write_fields(writer, offset);
                })*
            }

            unsafe fn write_required(writer: &mut ComponentWriter) {
                $(unsafe { <$name>::write_required(writer); })*
            }
        }
    };
}

range_invoke!(impl_bundle_for_tuple,  15: P);

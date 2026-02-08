// -----------------------------------------------------------------------------
// Modules

mod ident;
mod info;

// -----------------------------------------------------------------------------
// Exports

use core::any::TypeId;

pub use ident::BundleId;
pub use info::{BundleInfo, Bundles};

// -----------------------------------------------------------------------------
// Inline

use alloc::vec::Vec;
use vc_ptr::OwningPtr;
use vc_utils::range_invoke;

use crate::archetype::Archetype;
use crate::component::{CompIdAllocator, Component, ComponentId, Components};
use crate::entity::EntityId;
use crate::storage::{SparseSets, StorageType, Table, TableRow};
use crate::tick::Tick;
use crate::utils::DebugLocation;

pub struct BundleComponentRegistrar<'a> {
    pub(crate) components: &'a mut Components,
    pub(crate) allocator: &'a mut CompIdAllocator,
    pub(crate) out: &'a mut Vec<ComponentId>,
}

impl BundleComponentRegistrar<'_> {
    #[inline]
    pub fn register<T: Component>(&mut self) {
        self.out
            .push(self.components.register_component::<T>(self.allocator));
    }
}

pub struct BundleComponentWriter<'a> {
    pub(crate) data: OwningPtr<'a>,
    pub(crate) components: &'a Components,
    pub(crate) archetype: &'a Archetype,
    pub(crate) sparse_sets: &'a mut SparseSets,
    pub(crate) table: &'a mut Table,
    pub(crate) table_row: TableRow,
    pub(crate) entity_id: EntityId,
    pub(crate) tick: Tick,
    pub(crate) caller: DebugLocation,
}

impl BundleComponentWriter<'_> {
    #[inline(always)]
    pub fn write<T: Component>(&mut self, offset: usize) {
        self.write_internal(TypeId::of::<T>(), offset);
    }

    #[inline(never)]
    fn write_internal(&mut self, type_id: TypeId, offset: usize) {
        let data = unsafe { self.data.borrow_mut().byte_add(offset).promote() };
        let component_id = unsafe { self.components.get_component_id_unchecked(type_id) };
        let (storage_type, storage_index) =
            unsafe { self.archetype.get_storage_info_unchecked(component_id) };
        match storage_type {
            StorageType::Table => unsafe {
                self.table.init_component(
                    storage_index,
                    self.table_row,
                    data,
                    self.tick,
                    self.caller,
                );
            },
            StorageType::SparseSet => unsafe {
                self.sparse_sets.init_component(
                    storage_index,
                    self.entity_id,
                    data,
                    self.tick,
                    self.caller,
                );
            },
        }
    }
}

pub trait Bundle: Sized + Sync + Send + 'static {
    const COMPONENT_COUNT: usize;

    fn register_components(registrar: &mut BundleComponentRegistrar);
    fn write_components(base: usize, writer: &mut BundleComponentWriter);
}

impl<T: Component> Bundle for T {
    const COMPONENT_COUNT: usize = 1;

    fn register_components(registrar: &mut BundleComponentRegistrar) {
        registrar.register::<T>();
    }

    fn write_components(base: usize, writer: &mut BundleComponentWriter) {
        writer.write::<T>(base);
    }
}

macro_rules! impl_bundle_for_tuple {
    (0: []) => {
        impl Bundle for () {
            const COMPONENT_COUNT: usize = 0;

            fn register_components(_registrar: &mut BundleComponentRegistrar) {}
            fn write_components(_base: usize, _writer: &mut BundleComponentWriter) {}
        }
    };
    (1 : [ $index:tt : $name:ident ]) => {
        #[cfg_attr(docsrs, doc(fake_variadic))]
        impl<$name: Bundle> Bundle for ($name,) {
            const COMPONENT_COUNT: usize = <$name>::COMPONENT_COUNT;

            fn register_components(registrar: &mut BundleComponentRegistrar) {
                <$name>::register_components(registrar)
            }

            fn write_components(base: usize, writer: &mut BundleComponentWriter) {
                #[cfg(debug_assertions)]
                const {
                    assert!(::core::mem::offset_of!(Self, 0) == 0);
                }

                <$name>::write_components(base, writer);
            }
        }
    };
    ($num:literal : [$($index:tt : $name:ident),*]) => {
        #[cfg_attr(docsrs, doc(hidden))]
        impl<$($name: Bundle),*> Bundle for ($($name,)*) {
            const COMPONENT_COUNT: usize = { 0 $( + <$name>::COMPONENT_COUNT )* };

            fn register_components(registrar: &mut BundleComponentRegistrar) {
                $( <$name>::register_components(registrar); )*
            }

            fn write_components(base: usize, writer: &mut BundleComponentWriter) {
                $({
                    let offset = ::core::mem::offset_of!(Self, $index) + base;
                    <$name>::write_components(offset, writer);
                })*
            }
        }
    };
}

range_invoke!(impl_bundle_for_tuple,  12: P);

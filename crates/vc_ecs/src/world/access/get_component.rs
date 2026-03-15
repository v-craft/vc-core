#![allow(clippy::missing_safety_doc, reason = "todo")]

use core::any::TypeId;

use crate::archetype::ArcheId;
use crate::borrow::{Mut, Ref};
use crate::component::{Component, ComponentStorage};
use crate::entity::Entity;
use crate::storage::{TableId, TableRow};
use crate::tick::Tick;
use crate::world::UnsafeWorld;

pub unsafe trait GetComponents {
    type Raw<'a>;
    type Ref<'a>;
    type Mut<'a>;

    unsafe fn contains(world: UnsafeWorld, arche_id: ArcheId) -> bool;

    unsafe fn get<'a>(
        world: UnsafeWorld<'a>,
        entity: Entity,
        table_id: TableId,
        table_row: TableRow,
    ) -> Option<Self::Raw<'a>>;

    unsafe fn get_ref<'a>(
        world: UnsafeWorld<'a>,
        entity: Entity,
        table_id: TableId,
        table_row: TableRow,
        last_run: Tick,
        this_run: Tick,
    ) -> Option<Self::Ref<'a>>;

    unsafe fn get_mut<'a>(
        world: UnsafeWorld<'a>,
        entity: Entity,
        table_id: TableId,
        table_row: TableRow,
        last_run: Tick,
        this_run: Tick,
    ) -> Option<Self::Mut<'a>>;
}

unsafe impl<T: Component> GetComponents for T {
    type Raw<'a> = &'a T;
    type Ref<'a> = Ref<'a, T>;
    type Mut<'a> = Mut<'a, T>;

    unsafe fn contains(world: UnsafeWorld, arche_id: ArcheId) -> bool {
        let world = unsafe { world.read_only() };
        let Some(id) = world.components.get_id(TypeId::of::<T>()) else {
            return false;
        };
        let arche = unsafe { world.archetypes.get_unchecked(arche_id) };
        match T::STORAGE {
            ComponentStorage::Dense => arche.contains_dense_component(id),
            ComponentStorage::Sparse => arche.contains_sparse_component(id),
        }
    }

    unsafe fn get<'a>(
        world: UnsafeWorld<'a>,
        entity: Entity,
        table_id: TableId,
        table_row: TableRow,
    ) -> Option<Self::Raw<'a>> {
        let world = unsafe { world.read_only() };
        let id = world.components.get_id(TypeId::of::<T>())?;
        match T::STORAGE {
            ComponentStorage::Dense => {
                let tables = &world.storages.tables;
                let table = unsafe { tables.get_unchecked(table_id) };
                let table_col = table.get_table_col(id)?;
                let ptr = unsafe { table.get_data(table_row, table_col) };
                ptr.debug_assert_aligned::<T>();
                Some(unsafe { ptr.as_ref() })
            }
            ComponentStorage::Sparse => {
                let maps = &world.storages.maps;
                let map_id = maps.get_id(id)?;
                let map = unsafe { maps.get_unchecked(map_id) };
                let map_row = map.get_map_row(entity)?;
                let ptr = unsafe { map.get_data(map_row) };
                ptr.debug_assert_aligned::<T>();
                Some(unsafe { ptr.as_ref() })
            }
        }
    }

    unsafe fn get_ref<'a>(
        world: UnsafeWorld<'a>,
        entity: Entity,
        table_id: TableId,
        table_row: TableRow,
        last_run: Tick,
        this_run: Tick,
    ) -> Option<Self::Ref<'a>> {
        let world = unsafe { world.read_only() };
        let id = world.components.get_id(TypeId::of::<T>())?;
        match T::STORAGE {
            ComponentStorage::Dense => {
                let tables = &world.storages.tables;
                let table = unsafe { tables.get_unchecked(table_id) };
                let table_col = table.get_table_col(id)?;
                let untyped = unsafe { table.get_ref(table_row, table_col, last_run, this_run) };
                Some(unsafe { untyped.with_type::<T>() })
            }
            ComponentStorage::Sparse => {
                let maps = &world.storages.maps;
                let map_id = maps.get_id(id)?;
                let map = unsafe { maps.get_unchecked(map_id) };
                let map_row = map.get_map_row(entity)?;
                let untyped = unsafe { map.get_ref(map_row, last_run, this_run) };
                Some(unsafe { untyped.with_type::<T>() })
            }
        }
    }

    unsafe fn get_mut<'a>(
        world: UnsafeWorld<'a>,
        entity: Entity,
        table_id: TableId,
        table_row: TableRow,
        last_run: Tick,
        this_run: Tick,
    ) -> Option<Self::Mut<'a>> {
        if !T::MUTABLE {
            return None;
        }

        let world = unsafe { world.data_mut() };
        let id = world.components.get_id(TypeId::of::<T>())?;
        match T::STORAGE {
            ComponentStorage::Dense => {
                let tables = &mut world.storages.tables;
                let table = unsafe { tables.get_unchecked_mut(table_id) };
                let table_col = table.get_table_col(id)?;
                let untyped = unsafe { table.get_mut(table_row, table_col, last_run, this_run) };
                Some(unsafe { untyped.with_type::<T>() })
            }
            ComponentStorage::Sparse => {
                let maps = &mut world.storages.maps;
                let map_id = maps.get_id(id)?;
                let map = unsafe { maps.get_unchecked_mut(map_id) };
                let map_row = map.get_map_row(entity)?;
                let untyped = unsafe { map.get_mut(map_row, last_run, this_run) };
                Some(unsafe { untyped.with_type::<T>() })
            }
        }
    }
}

macro_rules! impl_tuple {
    (0: []) => {};
    (1 : [ $index:tt : $name:ident ]) => {
        #[cfg_attr(docsrs, doc(fake_variadic))]
        unsafe impl<$name: Component> GetComponents for ($name,) {
            type Raw<'a> = ( &'a $name, );
            type Ref<'a> = ( Ref<'a, $name>, );
            type Mut<'a> = ( Mut<'a, $name>, );

            unsafe fn contains(world: UnsafeWorld, arche_id: ArcheId) -> bool {
                unsafe {
                    <$name as GetComponents>::contains(world, arche_id)
                }
            }

            unsafe fn get<'a>(
                world: UnsafeWorld<'a>,
                entity: Entity,
                table_id: TableId,
                table_row: TableRow,
            ) -> Option<Self::Raw<'a>> {
                unsafe {
                    Some((
                        <$name as GetComponents>::get(world, entity, table_id, table_row)?,
                    ))
                }
            }

            unsafe fn get_ref<'a>(
                world: UnsafeWorld<'a>,
                entity: Entity,
                table_id: TableId,
                table_row: TableRow,
                last_run: Tick,
                this_run: Tick,
            ) -> Option<Self::Ref<'a>> {
                unsafe {
                    Some((
                        <$name as GetComponents>::get_ref(world, entity, table_id, table_row, last_run, this_run)?,
                    ))
                }
            }

            unsafe fn get_mut<'a>(
                world: UnsafeWorld<'a>,
                entity: Entity,
                table_id: TableId,
                table_row: TableRow,
                last_run: Tick,
                this_run: Tick,
            ) -> Option<Self::Mut<'a>> {
                unsafe {
                    Some((
                        <$name as GetComponents>::get_mut(world, entity, table_id, table_row, last_run, this_run)?,
                    ))
                }
            }
        }
    };
    ($num:literal : [$($index:tt : $name:ident),*]) => {
        #[cfg_attr(docsrs, doc(hidden))]
        unsafe impl<$($name: Component),*> GetComponents for ($($name,)*) {
            type Raw<'a> = ( $( &'a $name, )* );
            type Ref<'a> = ( $( Ref<'a, $name>, )* );
            type Mut<'a> = ( $( Mut<'a, $name>, )* );

            unsafe fn contains(world: UnsafeWorld, arche_id: ArcheId) -> bool {
                unsafe {
                    true $( && <$name as GetComponents>::contains(world, arche_id) )*
                }
            }

            unsafe fn get<'a>(
                world: UnsafeWorld<'a>,
                entity: Entity,
                table_id: TableId,
                table_row: TableRow,
            ) -> Option<Self::Raw<'a>> {
                unsafe {
                    Some((
                        $( <$name as GetComponents>::get(world, entity, table_id, table_row)?, )*
                    ))
                }
            }

            unsafe fn get_ref<'a>(
                world: UnsafeWorld<'a>,
                entity: Entity,
                table_id: TableId,
                table_row: TableRow,
                last_run: Tick,
                this_run: Tick,
            ) -> Option<Self::Ref<'a>> {
                unsafe {
                    Some((
                        $( <$name as GetComponents>::get_ref(world, entity, table_id, table_row, last_run, this_run)?, )*
                    ))
                }
            }

            unsafe fn get_mut<'a>(
                world: UnsafeWorld<'a>,
                entity: Entity,
                table_id: TableId,
                table_row: TableRow,
                last_run: Tick,
                this_run: Tick,
            ) -> Option<Self::Mut<'a>> {
                unsafe {
                    Some((
                        $( <$name as GetComponents>::get_mut(world, entity, table_id, table_row, last_run, this_run)?, )*
                    ))
                }
            }
        }
    };
}

vc_utils::range_invoke!(impl_tuple, 12);

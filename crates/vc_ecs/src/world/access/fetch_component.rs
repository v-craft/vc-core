#![allow(clippy::missing_safety_doc, reason = "todo")]

use core::any::TypeId;

use crate::borrow::{Mut, Ref};
use crate::component::{Component, ComponentStorage};
use crate::entity::Entity;
use crate::storage::{TableId, TableRow};
use crate::tick::Tick;
use crate::world::UnsafeWorld;

pub unsafe trait FetchComponents {
    type Item<'a>;

    unsafe fn fetch<'a>(
        mutable: bool,
        world: UnsafeWorld<'a>,
        entity: Entity,
        table_id: TableId,
        table_row: TableRow,
        last_run: Tick,
        this_run: Tick,
    ) -> Option<Self::Item<'a>>;
}

unsafe impl<T: Component> FetchComponents for &T {
    type Item<'a> = &'a T;

    unsafe fn fetch<'a>(
        _mutable: bool,
        world: UnsafeWorld<'a>,
        entity: Entity,
        table_id: TableId,
        table_row: TableRow,
        _last_run: Tick,
        _this_run: Tick,
    ) -> Option<Self::Item<'a>> {
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
}

unsafe impl<T: Component> FetchComponents for &mut T {
    type Item<'a> = &'a mut T;

    unsafe fn fetch<'a>(
        mutable: bool,
        world: UnsafeWorld<'a>,
        entity: Entity,
        table_id: TableId,
        table_row: TableRow,
        last_run: Tick,
        this_run: Tick,
    ) -> Option<Self::Item<'a>> {
        if !T::MUTABLE || !mutable {
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
                Some(unsafe { untyped.with_type::<T>().into_inner() })
            }
            ComponentStorage::Sparse => {
                let maps = &mut world.storages.maps;
                let map_id = maps.get_id(id)?;
                let map = unsafe { maps.get_unchecked_mut(map_id) };
                let map_row = map.get_map_row(entity)?;
                let untyped = unsafe { map.get_mut(map_row, last_run, this_run) };
                Some(unsafe { untyped.with_type::<T>().into_inner() })
            }
        }
    }
}

unsafe impl<T: Component> FetchComponents for Ref<'_, T> {
    type Item<'a> = Ref<'a, T>;

    unsafe fn fetch<'a>(
        _mutable: bool,
        world: UnsafeWorld<'a>,
        entity: Entity,
        table_id: TableId,
        table_row: TableRow,
        last_run: Tick,
        this_run: Tick,
    ) -> Option<Self::Item<'a>> {
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
}

unsafe impl<T: Component> FetchComponents for Mut<'_, T> {
    type Item<'a> = Mut<'a, T>;

    unsafe fn fetch<'a>(
        mutable: bool,
        world: UnsafeWorld<'a>,
        entity: Entity,
        table_id: TableId,
        table_row: TableRow,
        last_run: Tick,
        this_run: Tick,
    ) -> Option<Self::Item<'a>> {
        if !T::MUTABLE || !mutable {
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

unsafe impl<T: FetchComponents> FetchComponents for Option<T> {
    type Item<'a> = Option<T::Item<'a>>;

    unsafe fn fetch<'a>(
        mutable: bool,
        world: UnsafeWorld<'a>,
        entity: Entity,
        table_id: TableId,
        table_row: TableRow,
        last_run: Tick,
        this_run: Tick,
    ) -> Option<Self::Item<'a>> {
        unsafe {
            Some(T::fetch(
                mutable, world, entity, table_id, table_row, last_run, this_run,
            ))
        }
    }
}

macro_rules! impl_tuple {
    (0: []) => {};
    (1 : [ $index:tt : $name:ident ]) => {
        #[cfg_attr(docsrs, doc(fake_variadic))]
        unsafe impl<$name: FetchComponents> FetchComponents for ($name,) {
            type Item<'a> = (<$name>::Item<'a>,);

            unsafe fn fetch<'a>(
                mutable: bool,
                world: UnsafeWorld<'a>,
                entity: Entity,
                table_id: TableId,
                table_row: TableRow,
                last_run: Tick,
                this_run: Tick,
            ) -> Option<Self::Item<'a>> {
                unsafe {
                    Some((
                        <$name>::fetch(mutable, world, entity, table_id, table_row, last_run, this_run)?,
                    ))
                }
            }
        }
    };
    ($num:literal : [$($index:tt : $name:ident),*]) => {
        #[cfg_attr(docsrs, doc(hidden))]
        unsafe impl<$($name: FetchComponents),*> FetchComponents for ($($name,)*) {
            type Item<'a> = ( $( <$name>::Item<'a>, )* );

            unsafe fn fetch<'a>(
                mutable: bool,
                world: UnsafeWorld<'a>,
                entity: Entity,
                table_id: TableId,
                table_row: TableRow,
                last_run: Tick,
                this_run: Tick,
            ) -> Option<Self::Item<'a>> {
                unsafe {
                    Some((
                        $( <$name>::fetch(mutable, world, entity, table_id, table_row, last_run, this_run)?, )*
                    ))
                }
            }
        }
    };
}

vc_utils::range_invoke!(impl_tuple,  8: P);

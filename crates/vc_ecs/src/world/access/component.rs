use core::any::TypeId;
use core::ptr::NonNull;

use vc_utils::range_invoke;

use crate::borrow::{Mut, Ref};
use crate::component::{Component, ComponentStorage};
use crate::entity::Entity;
use crate::storage::{TableId, TableRow};
use crate::tick::Tick;
use crate::world::World;

pub trait FetchComponent {
    type Raw<'a>;
    type Ref<'a>;
    type Mut<'a>;

    /// # Safety
    /// The caller guarantees the correctness of the lifecycle
    unsafe fn get<'a>(
        world: NonNull<World>,
        entity: Entity,
        table_id: TableId,
        table_row: TableRow,
    ) -> Option<Self::Raw<'a>>;

    /// # Safety
    /// The caller guarantees the correctness of the lifecycle
    unsafe fn get_ref<'a>(
        world: NonNull<World>,
        entity: Entity,
        table_id: TableId,
        table_row: TableRow,
        last_run: Tick,
        this_run: Tick,
    ) -> Option<Self::Ref<'a>>;

    /// # Safety
    /// The caller guarantees the correctness of the lifecycle
    unsafe fn get_mut<'a>(
        world: NonNull<World>,
        entity: Entity,
        table_id: TableId,
        table_row: TableRow,
        last_run: Tick,
        this_run: Tick,
    ) -> Option<Self::Mut<'a>>;
}

impl<T: Component> FetchComponent for T {
    type Raw<'a> = &'a T;
    type Ref<'a> = Ref<'a, T>;
    type Mut<'a> = Mut<'a, T>;

    unsafe fn get<'a>(
        world: NonNull<World>,
        entity: Entity,
        table_id: TableId,
        table_row: TableRow,
    ) -> Option<Self::Raw<'a>> {
        let world = unsafe { &*world.as_ptr() };
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
        world: NonNull<World>,
        entity: Entity,
        table_id: TableId,
        table_row: TableRow,
        last_run: Tick,
        this_run: Tick,
    ) -> Option<Self::Ref<'a>> {
        let world = unsafe { &*world.as_ptr() };
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
        world: NonNull<World>,
        entity: Entity,
        table_id: TableId,
        table_row: TableRow,
        last_run: Tick,
        this_run: Tick,
    ) -> Option<Self::Mut<'a>> {
        if !T::MUTABLE {
            return None;
        }

        let world = unsafe { &mut *world.as_ptr() };
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

impl<T: Component> FetchComponent for Option<T> {
    type Raw<'a> = Option<&'a T>;
    type Ref<'a> = Option<Ref<'a, T>>;
    type Mut<'a> = Option<Mut<'a, T>>;

    unsafe fn get<'a>(
        world: NonNull<World>,
        entity: Entity,
        table_id: TableId,
        table_row: TableRow,
    ) -> Option<Self::Raw<'a>> {
        let world = unsafe { &*world.as_ptr() };
        let Some(id) = world.components.get_id(TypeId::of::<T>()) else {
            return Some(None);
        };
        match T::STORAGE {
            ComponentStorage::Dense => {
                let tables = &world.storages.tables;
                let table = unsafe { tables.get_unchecked(table_id) };
                let Some(table_col) = table.get_table_col(id) else {
                    return Some(None);
                };
                let ptr = unsafe { table.get_data(table_row, table_col) };
                ptr.debug_assert_aligned::<T>();
                Some(Some(unsafe { ptr.as_ref() }))
            }
            ComponentStorage::Sparse => {
                let maps = &world.storages.maps;
                let map_id = maps.get_id(id)?;
                let map = unsafe { maps.get_unchecked(map_id) };
                let Some(map_row) = map.get_map_row(entity) else {
                    return Some(None);
                };
                let ptr = unsafe { map.get_data(map_row) };
                ptr.debug_assert_aligned::<T>();
                Some(Some(unsafe { ptr.as_ref() }))
            }
        }
    }

    unsafe fn get_ref<'a>(
        world: NonNull<World>,
        entity: Entity,
        table_id: TableId,
        table_row: TableRow,
        last_run: Tick,
        this_run: Tick,
    ) -> Option<Self::Ref<'a>> {
        let world = unsafe { &*world.as_ptr() };
        let Some(id) = world.components.get_id(TypeId::of::<T>()) else {
            return Some(None);
        };
        match T::STORAGE {
            ComponentStorage::Dense => {
                let tables = &world.storages.tables;
                let table = unsafe { tables.get_unchecked(table_id) };
                let Some(table_col) = table.get_table_col(id) else {
                    return Some(None);
                };
                let untyped = unsafe { table.get_ref(table_row, table_col, last_run, this_run) };
                Some(Some(unsafe { untyped.with_type::<T>() }))
            }
            ComponentStorage::Sparse => {
                let maps = &world.storages.maps;
                let map_id = maps.get_id(id)?;
                let map = unsafe { maps.get_unchecked(map_id) };
                let Some(map_row) = map.get_map_row(entity) else {
                    return Some(None);
                };
                let untyped = unsafe { map.get_ref(map_row, last_run, this_run) };
                Some(Some(unsafe { untyped.with_type::<T>() }))
            }
        }
    }

    unsafe fn get_mut<'a>(
        world: NonNull<World>,
        entity: Entity,
        table_id: TableId,
        table_row: TableRow,
        last_run: Tick,
        this_run: Tick,
    ) -> Option<Self::Mut<'a>> {
        if !T::MUTABLE {
            return None;
        }

        let world = unsafe { &mut *world.as_ptr() };
        let Some(id) = world.components.get_id(TypeId::of::<T>()) else {
            return Some(None);
        };
        match T::STORAGE {
            ComponentStorage::Dense => {
                let tables = &mut world.storages.tables;
                let table = unsafe { tables.get_unchecked_mut(table_id) };
                let Some(table_col) = table.get_table_col(id) else {
                    return Some(None);
                };
                let untyped = unsafe { table.get_mut(table_row, table_col, last_run, this_run) };
                Some(Some(unsafe { untyped.with_type::<T>() }))
            }
            ComponentStorage::Sparse => {
                let maps = &mut world.storages.maps;
                let map_id = maps.get_id(id)?;
                let map = unsafe { maps.get_unchecked_mut(map_id) };
                let Some(map_row) = map.get_map_row(entity) else {
                    return Some(None);
                };
                let untyped = unsafe { map.get_mut(map_row, last_run, this_run) };
                Some(Some(unsafe { untyped.with_type::<T>() }))
            }
        }
    }
}

macro_rules! impl_bundle_for_tuple {
    (0: []) => {};
    (1 : [ $index:tt : $name:ident ]) => {
        #[cfg_attr(docsrs, doc(fake_variadic))]
        impl<$name: FetchComponent> FetchComponent for ($name,) {
            type Raw<'a> = (<$name>::Raw<'a>,);
            type Ref<'a> = (<$name>::Ref<'a>,);
            type Mut<'a> = (<$name>::Mut<'a>,);

            unsafe fn get<'a>(
                world: NonNull<World>,
                entity: Entity,
                table_id: TableId,
                table_row: TableRow,
            ) -> Option<Self::Raw<'a>> {
                unsafe {
                    Some((
                        <$name>::get(world, entity, table_id, table_row)?,
                    ))
                }

            }

            unsafe fn get_ref<'a>(
                world: NonNull<World>,
                entity: Entity,
                table_id: TableId,
                table_row: TableRow,
                last_run: Tick,
                this_run: Tick,
            ) -> Option<Self::Ref<'a>> {
                unsafe {
                    Some((
                        <$name>::get_ref(world, entity, table_id, table_row, last_run, this_run)?,
                    ))
                }
            }

            unsafe fn get_mut<'a>(
                world: NonNull<World>,
                entity: Entity,
                table_id: TableId,
                table_row: TableRow,
                last_run: Tick,
                this_run: Tick,
            ) -> Option<Self::Mut<'a>> {
                unsafe {
                    Some((
                        <$name>::get_mut(world, entity, table_id, table_row, last_run, this_run)?,
                    ))
                }
            }
        }
    };
    ($num:literal : [$($index:tt : $name:ident),*]) => {
        #[cfg_attr(docsrs, doc(hidden))]
        impl<$($name: FetchComponent),*> FetchComponent for ($($name,)*) {
            type Raw<'a> = ( $( <$name>::Raw<'a>, )* );
            type Ref<'a> = ( $( <$name>::Ref<'a>, )* );
            type Mut<'a> = ( $( <$name>::Mut<'a>, )* );

            unsafe fn get<'a>(
                world: NonNull<World>,
                entity: Entity,
                table_id: TableId,
                table_row: TableRow,
            ) -> Option<Self::Raw<'a>> {
                unsafe {
                    Some((
                        $( <$name>::get(world, entity, table_id, table_row)?, )*
                    ))
                }
            }

            unsafe fn get_ref<'a>(
                world: NonNull<World>,
                entity: Entity,
                table_id: TableId,
                table_row: TableRow,
                last_run: Tick,
                this_run: Tick,
            ) -> Option<Self::Ref<'a>> {
                unsafe {
                    Some((
                        $( <$name>::get_ref(world, entity, table_id, table_row, last_run, this_run)?, )*
                    ))
                }
            }

            unsafe fn get_mut<'a>(
                world: NonNull<World>,
                entity: Entity,
                table_id: TableId,
                table_row: TableRow,
                last_run: Tick,
                this_run: Tick,
            ) -> Option<Self::Mut<'a>> {
                unsafe {
                    Some((
                        $( <$name>::get_mut(world, entity, table_id, table_row, last_run, this_run)?, )*
                    ))
                }
            }
        }
    };
}

range_invoke!(impl_bundle_for_tuple,  12: P);

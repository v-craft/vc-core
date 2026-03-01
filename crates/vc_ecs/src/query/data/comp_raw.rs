use alloc::vec::Vec;
use vc_ptr::Ptr;

use super::{QueryData, ReadOnlyQuery};
use crate::archetype::Archetype;
use crate::component::{Component, ComponentId, ComponentStorage};
use crate::entity::Entity;
use crate::storage::{Map, Table, TableRow};
use crate::system::{FilterData, FilterParamBuilder};
use crate::tick::Tick;
use crate::world::{UnsafeWorld, World, WorldMode};

// -----------------------------------------------------------------------------
// &T

// We do not use A and hope to slightly reduce compilation time.
pub union ComponentView<'w> {
    dense: Option<Ptr<'w>>,
    sparse: Option<&'w Map>,
}

unsafe impl<T: Component> ReadOnlyQuery for &T {}

unsafe impl<T: Component> QueryData for &T {
    type State = ComponentId;
    type Cache<'world> = ComponentView<'world>;
    type Item<'world> = &'world T;

    const COMPONENTS_ARE_DENSE: bool = T::STORAGE.is_dense();
    const WORLD_MODE: WorldMode = WorldMode::ReadOnly;

    unsafe fn build_state(world: &mut World) -> Self::State {
        world.register_component::<T>()
    }

    unsafe fn build_cache<'w>(
        state: &Self::State,
        world: UnsafeWorld<'w>,
        _last_run: Tick,
        _this_run: Tick,
    ) -> Self::Cache<'w> {
        match T::STORAGE {
            ComponentStorage::Dense => ComponentView { dense: None },
            ComponentStorage::Sparse => {
                let world_ref = unsafe { world.read_only() };
                let maps = &world_ref.storages.maps;
                let Some(map_id) = maps.get_id(*state) else {
                    return ComponentView { sparse: None };
                };
                let map = unsafe { maps.get_unchecked(map_id) };
                ComponentView { sparse: Some(map) }
            }
        }
    }

    unsafe fn build_filter(state: &Self::State, out: &mut Vec<FilterParamBuilder>) {
        out.iter_mut().for_each(|param| {
            param.with(*state);
        });
    }

    unsafe fn build_target(state: &Self::State, out: &mut FilterData) -> bool {
        if out.can_reading(*state) {
            out.set_reading(*state);
            true
        } else {
            false
        }
    }

    unsafe fn set_for_arche<'w>(
        state: &Self::State,
        cache: &mut Self::Cache<'w>,
        _arche: &'w Archetype,
        table: &'w Table,
    ) {
        unsafe {
            Self::set_for_table(state, cache, table);
        }
    }

    unsafe fn set_for_table<'w>(
        state: &Self::State,
        cache: &mut Self::Cache<'w>,
        table: &'w Table,
    ) {
        if T::STORAGE.is_dense() {
            let Some(table_col) = table.get_table_col(*state) else {
                cache.dense = None;
                return;
            };
            cache.dense = Some(unsafe { table.get_data_ptr(table_col) });
        }
    }

    unsafe fn fetch<'w>(
        _state: &Self::State,
        cache: &mut Self::Cache<'w>,
        entity: Entity,
        table_row: TableRow,
    ) -> Option<Self::Item<'w>> {
        match T::STORAGE {
            ComponentStorage::Dense => {
                let ptr = unsafe { cache.dense }?;
                let size = size_of::<T>();
                let data = unsafe { ptr.byte_add(size * table_row.0 as usize) };
                data.debug_assert_aligned::<T>();
                Some(unsafe { data.as_ref::<T>() })
            }
            ComponentStorage::Sparse => {
                let map = unsafe { cache.sparse }?;
                let map_row = map.get_map_row(entity)?;
                let ptr = unsafe { map.get_data(map_row) };
                ptr.debug_assert_aligned::<T>();
                Some(unsafe { ptr.as_ref::<T>() })
            }
        }
    }
}

unsafe impl<T: Component> QueryData for Option<&T> {
    type State = ComponentId;
    type Cache<'world> = ComponentView<'world>;
    type Item<'world> = Option<&'world T>;

    const COMPONENTS_ARE_DENSE: bool = T::STORAGE.is_dense();
    const WORLD_MODE: WorldMode = WorldMode::ReadOnly;

    unsafe fn build_state(world: &mut World) -> Self::State {
        world.register_component::<T>()
    }

    unsafe fn build_cache<'w>(
        state: &Self::State,
        world: UnsafeWorld<'w>,
        _last_run: Tick,
        _this_run: Tick,
    ) -> Self::Cache<'w> {
        match T::STORAGE {
            ComponentStorage::Dense => ComponentView { dense: None },
            ComponentStorage::Sparse => {
                let world_ref = unsafe { world.read_only() };
                let maps = &world_ref.storages.maps;
                let Some(map_id) = maps.get_id(*state) else {
                    return ComponentView { sparse: None };
                };
                let map = unsafe { maps.get_unchecked(map_id) };
                ComponentView { sparse: Some(map) }
            }
        }
    }

    unsafe fn build_filter(_state: &Self::State, _out: &mut Vec<FilterParamBuilder>) {}

    unsafe fn build_target(state: &Self::State, out: &mut FilterData) -> bool {
        if out.can_reading(*state) {
            out.set_reading(*state);
            true
        } else {
            false
        }
    }

    unsafe fn set_for_arche<'w>(
        state: &Self::State,
        cache: &mut Self::Cache<'w>,
        _arche: &'w Archetype,
        table: &'w Table,
    ) {
        unsafe {
            Self::set_for_table(state, cache, table);
        }
    }

    unsafe fn set_for_table<'w>(
        state: &Self::State,
        cache: &mut Self::Cache<'w>,
        table: &'w Table,
    ) {
        if T::STORAGE.is_dense() {
            let Some(table_col) = table.get_table_col(*state) else {
                cache.dense = None;
                return;
            };
            cache.dense = Some(unsafe { table.get_data_ptr(table_col) });
        }
    }

    unsafe fn fetch<'w>(
        _state: &Self::State,
        cache: &mut Self::Cache<'w>,
        entity: Entity,
        table_row: TableRow,
    ) -> Option<Self::Item<'w>> {
        match T::STORAGE {
            ComponentStorage::Dense => {
                let ptr = unsafe { cache.dense };
                let Some(ptr) = ptr else { return Some(None) };
                let size = size_of::<T>();
                let data = unsafe { ptr.byte_add(size * table_row.0 as usize) };
                data.debug_assert_aligned::<T>();
                Some(Some(unsafe { data.as_ref::<T>() }))
            }
            ComponentStorage::Sparse => {
                let map = unsafe { cache.sparse };
                let Some(map) = map else { return Some(None) };
                let Some(map_row) = map.get_map_row(entity) else {
                    return Some(None);
                };
                let ptr = unsafe { map.get_data(map_row) };
                ptr.debug_assert_aligned::<T>();
                Some(Some(unsafe { ptr.as_ref::<T>() }))
            }
        }
    }
}

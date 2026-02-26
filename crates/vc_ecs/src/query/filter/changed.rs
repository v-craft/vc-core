use alloc::vec::Vec;
use vc_ptr::ThinSlice;

use super::QueryFilter;
use crate::archetype::Archetype;
use crate::component::{Component, ComponentId, ComponentStorage};
use crate::entity::Entity;
use crate::storage::{Map, Storages, Table, TableRow};
use crate::system::FilterParamBuilder;
use crate::tick::Tick;
use crate::world::{UnsafeWorld, World};

// -----------------------------------------------------------------------------
// Changed

#[derive(Clone, Copy)]
union StorageSwitch<'w> {
    dense: Option<ThinSlice<'w, Tick>>,
    sparse: Option<&'w Map>,
}

#[derive(Clone)]
pub struct ChangedView<'w> {
    storages: &'w Storages,
    ticks: StorageSwitch<'w>,
    last_run: Tick,
    this_run: Tick,
}

pub struct Changed<T: Component>(T);

unsafe impl<T: Component> QueryFilter for Changed<T> {
    type State = ComponentId;
    type Cache<'world> = ChangedView<'world>;

    const COMPONENTS_ARE_DENSE: bool = T::STORAGE.is_dense();
    const ENABLE_ENTITY_FILTER: bool = true;

    unsafe fn build_state(world: &mut World) -> Self::State {
        world.register_component::<T>()
    }

    unsafe fn build_cache<'w>(
        state: &Self::State,
        world: UnsafeWorld<'w>,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Cache<'w> {
        let storages = unsafe { &world.read_only().storages };
        let ticks = match T::STORAGE {
            ComponentStorage::Dense => StorageSwitch { dense: None },
            ComponentStorage::Sparse => {
                if let Some(map_id) = storages.maps.get_id(*state) {
                    StorageSwitch {
                        sparse: storages.maps.get(map_id),
                    }
                } else {
                    StorageSwitch { sparse: None }
                }
            }
        };

        ChangedView {
            storages,
            ticks,
            last_run,
            this_run,
        }
    }

    unsafe fn build_filter(state: &Self::State, outer: &mut Vec<FilterParamBuilder>) {
        let mut builder = FilterParamBuilder::new();
        builder.with(*state);
        outer.push(builder);
    }

    unsafe fn set_for_arche<'w>(
        state: &Self::State,
        cache: &mut Self::Cache<'w>,
        arche: &'w Archetype,
    ) {
        if T::STORAGE.is_dense() {
            let table_id = arche.table_id();
            let table = unsafe { cache.storages.tables.get_unchecked(table_id) };
            let Some(table_col) = table.get_table_col(*state) else {
                cache.ticks = StorageSwitch { dense: None };
                return;
            };
            let slice = unsafe { ThinSlice::from_ref(table.get_changed_slice(table_col)) };
            cache.ticks = StorageSwitch { dense: Some(slice) };
        }
    }

    unsafe fn set_for_table<'w>(
        state: &Self::State,
        cache: &mut Self::Cache<'w>,
        table: &'w Table,
    ) {
        if T::STORAGE.is_dense() {
            let Some(table_col) = table.get_table_col(*state) else {
                cache.ticks = StorageSwitch { dense: None };
                return;
            };
            let slice = unsafe { ThinSlice::from_ref(table.get_changed_slice(table_col)) };
            cache.ticks = StorageSwitch { dense: Some(slice) };
        }
    }

    unsafe fn filter<'w>(
        _state: &Self::State,
        cache: &mut Self::Cache<'w>,
        entity: Entity,
        table_row: TableRow,
    ) -> bool {
        match T::STORAGE {
            ComponentStorage::Dense => {
                let dense = unsafe { cache.ticks.dense };
                let Some(slice) = dense else {
                    return false;
                };
                let changed = unsafe { *slice.get(table_row.0 as usize) };
                changed.is_newer_than(cache.last_run, cache.this_run)
            }
            ComponentStorage::Sparse => {
                let sparse = unsafe { cache.ticks.sparse };
                let Some(map) = sparse else {
                    return false;
                };
                let Some(map_row) = map.get_map_row(entity) else {
                    return false;
                };
                let changed = unsafe { map.get_changed(map_row) };
                changed.is_newer_than(cache.last_run, cache.this_run)
            }
        }
    }
}

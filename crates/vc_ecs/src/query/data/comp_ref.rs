use alloc::vec::Vec;

use super::{QueryData, ReadOnlyQuery};
use crate::archetype::Archetype;
use crate::borrow::Ref;
use crate::component::{Component, ComponentId, ComponentStorage};
use crate::entity::Entity;
use crate::storage::{Map, Table, TableCol, TableRow};
use crate::system::{FilterData, FilterParamBuilder};
use crate::tick::Tick;
use crate::world::{UnsafeWorld, World, WorldMode};

// -----------------------------------------------------------------------------
// &T

union DataView<'w> {
    dense: Option<(&'w Table, TableCol)>,
    sparse: Option<&'w Map>,
}

pub struct ComponentRefView<'w> {
    data: DataView<'w>,
    last_run: Tick,
    this_run: Tick,
}

unsafe impl<T: Component> ReadOnlyQuery for Ref<'_, T> {}

unsafe impl<T: Component> QueryData for Ref<'_, T> {
    type State = ComponentId;
    type Cache<'world> = ComponentRefView<'world>;
    type Item<'world> = Ref<'world, T>;

    const COMPONENTS_ARE_DENSE: bool = T::STORAGE.is_dense();
    const WORLD_MODE: WorldMode = WorldMode::ReadOnly;

    unsafe fn build_state(world: &mut World) -> Self::State {
        world.register_component::<T>()
    }

    unsafe fn build_cache<'w>(
        state: &Self::State,
        world: UnsafeWorld<'w>,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Cache<'w> {
        match T::STORAGE {
            ComponentStorage::Dense => ComponentRefView {
                last_run,
                this_run,
                data: DataView { dense: None },
            },
            ComponentStorage::Sparse => {
                let world_ref = unsafe { world.read_only() };
                let maps = &world_ref.storages.maps;
                let Some(map_id) = maps.get_id(*state) else {
                    return ComponentRefView {
                        last_run,
                        this_run,
                        data: DataView { sparse: None },
                    };
                };
                let map = unsafe { maps.get_unchecked(map_id) };
                ComponentRefView {
                    last_run,
                    this_run,
                    data: DataView { sparse: Some(map) },
                }
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
                cache.data.dense = None;
                return;
            };
            cache.data.dense = Some((table, table_col));
        }
    }

    unsafe fn fetch<'w>(
        _state: &Self::State,
        cache: &mut Self::Cache<'w>,
        entity: Entity,
        table_row: TableRow,
    ) -> Option<Self::Item<'w>> {
        let last_run = cache.last_run;
        let this_run = cache.this_run;
        match T::STORAGE {
            ComponentStorage::Dense => {
                let (table, table_col) = unsafe { cache.data.dense }?;
                let untyped = unsafe { table.get_ref(table_row, table_col, last_run, this_run) };
                unsafe { Some(untyped.with_type::<T>()) }
            }
            ComponentStorage::Sparse => {
                let map = unsafe { cache.data.sparse }?;
                let map_row = map.get_map_row(entity)?;
                let untyped = unsafe { map.get_ref(map_row, last_run, this_run) };
                unsafe { Some(untyped.with_type::<T>()) }
            }
        }
    }
}

unsafe impl<T: Component> QueryData for Option<Ref<'_, T>> {
    type State = ComponentId;
    type Cache<'world> = ComponentRefView<'world>;
    type Item<'world> = Option<Ref<'world, T>>;

    const COMPONENTS_ARE_DENSE: bool = T::STORAGE.is_dense();
    const WORLD_MODE: WorldMode = WorldMode::ReadOnly;

    unsafe fn build_state(world: &mut World) -> Self::State {
        world.register_component::<T>()
    }

    unsafe fn build_cache<'w>(
        state: &Self::State,
        world: UnsafeWorld<'w>,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Cache<'w> {
        match T::STORAGE {
            ComponentStorage::Dense => ComponentRefView {
                last_run,
                this_run,
                data: DataView { dense: None },
            },
            ComponentStorage::Sparse => {
                let world_ref = unsafe { world.read_only() };
                let maps = &world_ref.storages.maps;
                let Some(map_id) = maps.get_id(*state) else {
                    return ComponentRefView {
                        last_run,
                        this_run,
                        data: DataView { sparse: None },
                    };
                };
                let map = unsafe { maps.get_unchecked(map_id) };
                ComponentRefView {
                    last_run,
                    this_run,
                    data: DataView { sparse: Some(map) },
                }
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
                cache.data.dense = None;
                return;
            };
            cache.data.dense = Some((table, table_col));
        }
    }

    unsafe fn fetch<'w>(
        _state: &Self::State,
        cache: &mut Self::Cache<'w>,
        entity: Entity,
        table_row: TableRow,
    ) -> Option<Self::Item<'w>> {
        let last_run = cache.last_run;
        let this_run = cache.this_run;
        match T::STORAGE {
            ComponentStorage::Dense => {
                let dense = unsafe { cache.data.dense };
                let Some((table, table_col)) = dense else {
                    return Some(None);
                };
                let untyped = unsafe { table.get_ref(table_row, table_col, last_run, this_run) };
                unsafe { Some(Some(untyped.with_type::<T>())) }
            }
            ComponentStorage::Sparse => {
                let sparse = unsafe { cache.data.sparse };
                let Some(map) = sparse else {
                    return Some(None);
                };
                let Some(map_row) = map.get_map_row(entity) else {
                    return Some(None);
                };
                let untyped = unsafe { map.get_ref(map_row, last_run, this_run) };
                unsafe { Some(Some(untyped.with_type::<T>())) }
            }
        }
    }
}

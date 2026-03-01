use core::marker::PhantomData;
use core::ptr::NonNull;

use alloc::vec::Vec;

use super::QueryData;
use crate::archetype::Archetype;
use crate::borrow::Mut;
use crate::component::{Component, ComponentId, ComponentStorage};
use crate::entity::Entity;
use crate::storage::{Map, Table, TableCol, TableRow};
use crate::system::{FilterData, FilterParamBuilder};
use crate::tick::Tick;
use crate::world::{UnsafeWorld, World, WorldMode};

// -----------------------------------------------------------------------------
// &mut T

union DataView {
    dense: Option<(NonNull<Table>, TableCol)>,
    sparse: Option<NonNull<Map>>,
}

pub struct ComponentMutView<'w> {
    _marker: PhantomData<&'w World>,
    data: DataView,
    last_run: Tick,
    this_run: Tick,
}

unsafe impl<T: Component> QueryData for &mut T {
    type State = ComponentId;
    type Cache<'world> = ComponentMutView<'world>;
    type Item<'world> = Mut<'world, T>;

    const COMPONENTS_ARE_DENSE: bool = T::STORAGE.is_dense();
    const WORLD_MODE: WorldMode = WorldMode::DataMut;

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
            ComponentStorage::Dense => ComponentMutView {
                _marker: PhantomData,
                last_run,
                this_run,
                data: DataView { dense: None },
            },
            ComponentStorage::Sparse => {
                let world_ref = unsafe { world.data_mut() };
                let maps = &mut world_ref.storages.maps;
                let Some(map_id) = maps.get_id(*state) else {
                    return ComponentMutView {
                        _marker: PhantomData,
                        last_run,
                        this_run,
                        data: DataView { sparse: None },
                    };
                };
                let map = unsafe { maps.get_unchecked_mut(map_id) };
                ComponentMutView {
                    _marker: PhantomData,
                    last_run,
                    this_run,
                    data: DataView {
                        sparse: Some(NonNull::from_mut(map)),
                    },
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
        if out.can_writing(*state) {
            out.set_writing(*state);
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
            let ptr = NonNull::from_ref(table);
            cache.data.dense = Some((ptr, table_col));
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
                let table = unsafe { &mut *table.as_ptr() };
                let untyped = unsafe { table.get_mut(table_row, table_col, last_run, this_run) };
                unsafe { Some(untyped.with_type::<T>()) }
            }
            ComponentStorage::Sparse => {
                let map = unsafe { cache.data.sparse }?;
                let map = unsafe { &mut *map.as_ptr() };
                let map_row = map.get_map_row(entity)?;
                let untyped = unsafe { map.get_mut(map_row, last_run, this_run) };
                unsafe { Some(untyped.with_type::<T>()) }
            }
        }
    }
}

unsafe impl<T: Component> QueryData for Mut<'_, T> {
    type State = ComponentId;
    type Cache<'world> = ComponentMutView<'world>;
    type Item<'world> = Mut<'world, T>;

    const COMPONENTS_ARE_DENSE: bool = T::STORAGE.is_dense();
    const WORLD_MODE: WorldMode = WorldMode::DataMut;

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
            ComponentStorage::Dense => ComponentMutView {
                _marker: PhantomData,
                last_run,
                this_run,
                data: DataView { dense: None },
            },
            ComponentStorage::Sparse => {
                let world_ref = unsafe { world.data_mut() };
                let maps = &mut world_ref.storages.maps;
                let Some(map_id) = maps.get_id(*state) else {
                    return ComponentMutView {
                        _marker: PhantomData,
                        last_run,
                        this_run,
                        data: DataView { sparse: None },
                    };
                };
                let map = unsafe { maps.get_unchecked_mut(map_id) };
                ComponentMutView {
                    _marker: PhantomData,
                    last_run,
                    this_run,
                    data: DataView {
                        sparse: Some(NonNull::from_mut(map)),
                    },
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
        if out.can_writing(*state) {
            out.set_writing(*state);
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
            let ptr = NonNull::from_ref(table);
            cache.data.dense = Some((ptr, table_col));
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
                let table = unsafe { &mut *table.as_ptr() };
                let untyped = unsafe { table.get_mut(table_row, table_col, last_run, this_run) };
                unsafe { Some(untyped.with_type::<T>()) }
            }
            ComponentStorage::Sparse => {
                let map = unsafe { cache.data.sparse }?;
                let map = unsafe { &mut *map.as_ptr() };
                let map_row = map.get_map_row(entity)?;
                let untyped = unsafe { map.get_mut(map_row, last_run, this_run) };
                unsafe { Some(untyped.with_type::<T>()) }
            }
        }
    }
}

unsafe impl<T: Component> QueryData for Option<&mut T> {
    type State = ComponentId;
    type Cache<'world> = ComponentMutView<'world>;
    type Item<'world> = Option<Mut<'world, T>>;

    const COMPONENTS_ARE_DENSE: bool = T::STORAGE.is_dense();
    const WORLD_MODE: WorldMode = WorldMode::DataMut;

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
            ComponentStorage::Dense => ComponentMutView {
                _marker: PhantomData,
                last_run,
                this_run,
                data: DataView { dense: None },
            },
            ComponentStorage::Sparse => {
                let world_ref = unsafe { world.data_mut() };
                let maps = &mut world_ref.storages.maps;
                let Some(map_id) = maps.get_id(*state) else {
                    return ComponentMutView {
                        _marker: PhantomData,
                        last_run,
                        this_run,
                        data: DataView { sparse: None },
                    };
                };
                let map = unsafe { maps.get_unchecked_mut(map_id) };
                ComponentMutView {
                    _marker: PhantomData,
                    last_run,
                    this_run,
                    data: DataView {
                        sparse: Some(NonNull::from_mut(map)),
                    },
                }
            }
        }
    }

    unsafe fn build_filter(_state: &Self::State, _out: &mut Vec<FilterParamBuilder>) {}

    unsafe fn build_target(state: &Self::State, out: &mut FilterData) -> bool {
        if out.can_writing(*state) {
            out.set_writing(*state);
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
            let ptr = NonNull::from_ref(table);
            cache.data.dense = Some((ptr, table_col));
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
                let dense = unsafe { cache.data.dense };
                let Some((table, table_col)) = dense else {
                    return Some(None);
                };
                let table = unsafe { &mut *table.as_ptr() };
                let untyped =
                    unsafe { table.get_mut(table_row, table_col, cache.last_run, cache.this_run) };
                unsafe { Some(Some(untyped.with_type::<T>())) }
            }
            ComponentStorage::Sparse => {
                let sparse = unsafe { cache.data.sparse };
                let Some(map) = sparse else {
                    return Some(None);
                };
                let map = unsafe { &mut *map.as_ptr() };
                let Some(map_row) = map.get_map_row(entity) else {
                    return Some(None);
                };
                let untyped = unsafe { map.get_mut(map_row, cache.last_run, cache.this_run) };
                unsafe { Some(Some(untyped.with_type::<T>())) }
            }
        }
    }
}

unsafe impl<T: Component> QueryData for Option<Mut<'_, T>> {
    type State = ComponentId;
    type Cache<'world> = ComponentMutView<'world>;
    type Item<'world> = Option<Mut<'world, T>>;

    const COMPONENTS_ARE_DENSE: bool = T::STORAGE.is_dense();
    const WORLD_MODE: WorldMode = WorldMode::DataMut;

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
            ComponentStorage::Dense => ComponentMutView {
                _marker: PhantomData,
                last_run,
                this_run,
                data: DataView { dense: None },
            },
            ComponentStorage::Sparse => {
                let world_ref = unsafe { world.data_mut() };
                let maps = &mut world_ref.storages.maps;
                let Some(map_id) = maps.get_id(*state) else {
                    return ComponentMutView {
                        _marker: PhantomData,
                        last_run,
                        this_run,
                        data: DataView { sparse: None },
                    };
                };
                let map = unsafe { maps.get_unchecked_mut(map_id) };
                ComponentMutView {
                    _marker: PhantomData,
                    last_run,
                    this_run,
                    data: DataView {
                        sparse: Some(NonNull::from_mut(map)),
                    },
                }
            }
        }
    }

    unsafe fn build_filter(_state: &Self::State, _out: &mut Vec<FilterParamBuilder>) {}

    unsafe fn build_target(state: &Self::State, out: &mut FilterData) -> bool {
        if out.can_writing(*state) {
            out.set_writing(*state);
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
            let ptr = NonNull::from_ref(table);
            cache.data.dense = Some((ptr, table_col));
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
                let dense = unsafe { cache.data.dense };
                let Some((table, table_col)) = dense else {
                    return Some(None);
                };
                let table = unsafe { &mut *table.as_ptr() };
                let untyped =
                    unsafe { table.get_mut(table_row, table_col, cache.last_run, cache.this_run) };
                unsafe { Some(Some(untyped.with_type::<T>())) }
            }
            ComponentStorage::Sparse => {
                let sparse = unsafe { cache.data.sparse };
                let Some(map) = sparse else {
                    return Some(None);
                };
                let map = unsafe { &mut *map.as_ptr() };
                let Some(map_row) = map.get_map_row(entity) else {
                    return Some(None);
                };
                let untyped = unsafe { map.get_mut(map_row, cache.last_run, cache.this_run) };
                unsafe { Some(Some(untyped.with_type::<T>())) }
            }
        }
    }
}

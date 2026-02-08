use alloc::vec::Vec;

use crate::archetype::Archetype;
use crate::entity::Entity;
use crate::query::QueryData;
use crate::storage::{Table, TableRow};
use crate::system::{FilterData, FilterParamBuilder};
use crate::tick::Tick;
use crate::world::{EntityMut, EntityRef, UnsafeWorld, World, WorldMode};

// -----------------------------------------------------------------------------
// Entity

unsafe impl QueryData for Entity {
    type State = ();
    type Cache<'world> = ();
    type Item<'world> = Entity;

    const COMPONENTS_ARE_DENSE: bool = true;
    const MODE: WorldMode = WorldMode::ReadOnly;

    unsafe fn build_state(_world: &mut World) -> Self::State {}

    unsafe fn build_cache<'w>(
        _state: &Self::State,
        _world: UnsafeWorld<'w>,
        _last_run: Tick,
        _this_run: Tick,
    ) -> Self::Cache<'w> {
    }

    unsafe fn filter_param(_state: &Self::State, _out: &mut Vec<FilterParamBuilder>) {}

    unsafe fn filter_data(_state: &Self::State, _out: &mut FilterData) -> bool {
        true
    }

    unsafe fn set_for_arche<'w, 's>(
        _state: &'s Self::State,
        _cache: &mut Self::Cache<'w>,
        _arche: &'w Archetype,
    ) {
    }

    unsafe fn set_for_table<'w, 's>(
        _state: &'s Self::State,
        _cache: &mut Self::Cache<'w>,
        _table: &'w Table,
    ) {
    }

    unsafe fn fetch<'w, 's>(
        _state: &'s Self::State,
        _cache: &mut Self::Cache<'w>,
        entity: Entity,
        _table_row: TableRow,
    ) -> Option<Self::Item<'w>> {
        Some(entity)
    }
}

// -----------------------------------------------------------------------------
// EntityRef & EntityMut

#[derive(Clone, Copy)]
pub struct EntityView<'w> {
    world: UnsafeWorld<'w>,
    last_run: Tick,
    this_run: Tick,
}

unsafe impl QueryData for EntityRef<'_> {
    type State = ();
    type Cache<'world> = EntityView<'world>;
    type Item<'world> = EntityRef<'world>;

    const COMPONENTS_ARE_DENSE: bool = true;
    const MODE: WorldMode = WorldMode::ReadOnly;

    unsafe fn build_state(_world: &mut World) -> Self::State {}

    unsafe fn build_cache<'w>(
        _state: &Self::State,
        world: UnsafeWorld<'w>,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Cache<'w> {
        EntityView {
            world,
            last_run,
            this_run,
        }
    }

    unsafe fn filter_param(_state: &Self::State, _out: &mut Vec<FilterParamBuilder>) {}

    unsafe fn filter_data(_state: &Self::State, out: &mut FilterData) -> bool {
        if out.can_entity_ref() {
            out.set_entity_ref();
            true
        } else {
            false
        }
    }

    unsafe fn set_for_arche<'w, 's>(
        _state: &'s Self::State,
        _cache: &mut Self::Cache<'w>,
        _arche: &'w Archetype,
    ) {
    }

    unsafe fn set_for_table<'w, 's>(
        _state: &'s Self::State,
        _cache: &mut Self::Cache<'w>,
        _table: &'w Table,
    ) {
    }

    unsafe fn fetch<'w, 's>(
        _state: &'s Self::State,
        cache: &mut Self::Cache<'w>,
        entity: Entity,
        _table_row: TableRow,
    ) -> Option<Self::Item<'w>> {
        let world = unsafe { cache.world.read_only() };
        let location = world.entities.get_spawned(entity).ok()?;
        Some(EntityRef {
            world,
            entity,
            location,
            last_run: cache.last_run,
            this_run: cache.this_run,
        })
    }
}

unsafe impl QueryData for EntityMut<'_> {
    type State = ();
    type Cache<'world> = EntityView<'world>;
    type Item<'world> = EntityMut<'world>;

    const COMPONENTS_ARE_DENSE: bool = true;
    const MODE: WorldMode = WorldMode::DataMut;

    unsafe fn build_state(_world: &mut World) -> Self::State {}

    unsafe fn build_cache<'w>(
        _state: &Self::State,
        world: UnsafeWorld<'w>,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Cache<'w> {
        EntityView {
            world,
            last_run,
            this_run,
        }
    }

    unsafe fn filter_param(_state: &Self::State, _out: &mut Vec<FilterParamBuilder>) {}

    unsafe fn filter_data(_state: &Self::State, out: &mut FilterData) -> bool {
        if out.can_entity_mut() {
            out.set_entity_mut();
            true
        } else {
            false
        }
    }

    unsafe fn set_for_arche<'w, 's>(
        _state: &'s Self::State,
        _cache: &mut Self::Cache<'w>,
        _arche: &'w Archetype,
    ) {
    }

    unsafe fn set_for_table<'w, 's>(
        _state: &'s Self::State,
        _cache: &mut Self::Cache<'w>,
        _table: &'w Table,
    ) {
    }

    unsafe fn fetch<'w, 's>(
        _state: &'s Self::State,
        cache: &mut Self::Cache<'w>,
        entity: Entity,
        _table_row: TableRow,
    ) -> Option<Self::Item<'w>> {
        let world = unsafe { cache.world.data_mut() };
        let location = world.entities.get_spawned(entity).ok()?;
        Some(EntityMut {
            world,
            entity,
            location,
            last_run: cache.last_run,
            this_run: cache.this_run,
        })
    }
}

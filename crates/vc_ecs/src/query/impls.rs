use core::iter::FusedIterator;

use super::{QueryData, QueryFilter, QueryState};
use crate::archetype::Archetypes;
use crate::entity::{Entities, Entity};
use crate::query::state::StorageId;
use crate::storage::{TableRow, Tables};
use crate::system::{AccessTable, SystemParam};
use crate::tick::Tick;
use crate::world::{UnsafeWorld, WorldMode};

pub struct Query<'world, 'state, D: QueryData, F: QueryFilter = ()> {
    world: UnsafeWorld<'world>,
    state: &'state QueryState<D, F>,
    last_run: Tick,
    this_run: Tick,
}

unsafe impl<D: QueryData + 'static, F: QueryFilter + 'static> SystemParam for Query<'_, '_, D, F> {
    type State = QueryState<D, F>;
    type Item<'world, 'state> = Query<'world, 'state, D, F>;

    const WORLD_MODE: WorldMode = D::WORLD_MODE;
    const MAIN_THREAD: bool = false;

    unsafe fn init_state(world: &mut crate::world::World) -> Self::State {
        unsafe { QueryState::init(world) }
    }

    unsafe fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
        state.mark_assess(table)
    }

    unsafe fn get_param<'w, 's>(
        world: UnsafeWorld<'w>,
        state: &'s mut Self::State,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Item<'w, 's> {
        unsafe {
            state.updata(world.read_only());
        }
        Query {
            world,
            state,
            last_run,
            this_run,
        }
    }
}

const EMPTY_ENTITIES: &[Entity] = &[];

impl<'w, 's, D: QueryData, F: QueryFilter> IntoIterator for Query<'w, 's, D, F> {
    type Item = D::Item<'w>;
    type IntoIter = QueryIter<'w, 's, D, F>;

    fn into_iter(self) -> Self::IntoIter {
        let last_run = self.last_run;
        let this_run = self.this_run;
        let world = self.world;
        let state = self.state;
        let storages = state.storages.iter();
        unsafe {
            let d_cache = D::build_cache(&state.d_state, world, last_run, this_run);
            let f_cache = F::build_cache(&state.f_state, world, last_run, this_run);
            let tables = &world.read_only().storages.tables;
            let arches = &world.read_only().archetypes;
            let entities = &world.read_only().entities;
            QueryIter {
                state,
                d_cache,
                f_cache,
                entities,
                arches,
                tables,
                storages,
                slice: EMPTY_ENTITIES,
                cursor: 0,
            }
        }
    }
}

pub struct QueryIter<'w, 's, D: QueryData, F: QueryFilter> {
    state: &'s QueryState<D, F>,
    d_cache: D::Cache<'w>,
    f_cache: F::Cache<'w>,
    entities: &'w Entities,
    arches: &'w Archetypes,
    tables: &'w Tables,
    storages: core::slice::Iter<'s, StorageId>,
    slice: &'w [Entity],
    cursor: u32,
}

impl<'w, 's, D: QueryData, F: QueryFilter> QueryIter<'w, 's, D, F> {
    #[cold]
    #[inline(never)]
    pub fn update_slice(&mut self) -> Option<()> {
        self.cursor = 0;
        loop {
            let id = *self.storages.next()?;
            if QueryState::<D, F>::IS_DENSE {
                let table_id = unsafe { id.table_id };
                let table = unsafe { self.tables.get_unchecked(table_id) };
                self.slice = table.entities();
                if !self.slice.is_empty() {
                    return Some(());
                }
            } else {
                let arche_id = unsafe { id.arche_id };
                let arche = unsafe { self.arches.get_unchecked(arche_id) };
                self.slice = arche.entities();
                if !self.slice.is_empty() {
                    return Some(());
                }
            }
        }
    }
}

impl<'w, 's, D: QueryData, F: QueryFilter> Iterator for QueryIter<'w, 's, D, F> {
    type Item = D::Item<'w>;

    fn next(&mut self) -> Option<Self::Item> {
        'looper: loop {
            if self.cursor as usize >= self.slice.len() {
                self.update_slice()?;
            }

            let entity = unsafe { *self.slice.get_unchecked(self.cursor as usize) };

            // cursor is table_row/arche_row, represent the number of entities.
            // 0 < EntityId <= u32::MAX, so cursor <= u32::MAX - 1, will not wrapping.
            self.cursor += 1;

            let table_row = if QueryState::<D, F>::IS_DENSE {
                TableRow(self.cursor - 1)
            } else {
                self.entities.get_spawned(entity).unwrap().table_row
            };

            if F::ENABLE_ENTITY_FILTER {
                let f_state = &self.state.f_state;
                let f_cache = &mut self.f_cache;
                let filter = unsafe { F::filter(f_state, f_cache, entity, table_row) };
                if !filter {
                    continue 'looper;
                }
            }

            let d_state = &self.state.d_state;
            let d_cache = &mut self.d_cache;

            if let Some(data) = unsafe { D::fetch(d_state, d_cache, entity, table_row) } {
                return Some(data);
            }
        }
    }
}

impl<'w, 's, D: QueryData, F: QueryFilter> FusedIterator for QueryIter<'w, 's, D, F> {}

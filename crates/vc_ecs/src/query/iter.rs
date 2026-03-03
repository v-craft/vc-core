use core::iter::FusedIterator;
use core::sync::atomic::Ordering;

use super::{QueryData, QueryFilter, QueryState};
use crate::entity::{Entity, StorageId};
use crate::query::{Query, ReadOnlyQuery};
use crate::storage::TableRow;
use crate::tick::Tick;
use crate::world::{UnsafeWorld, World};

pub struct QueryIter<'w, 's, D: QueryData, F: QueryFilter> {
    pub(super) world: UnsafeWorld<'w>,
    pub(super) state: &'s QueryState<D, F>,
    pub(super) d_cache: D::Cache<'w>,
    pub(super) f_cache: F::Cache<'w>,
    pub(super) storages: core::slice::Iter<'s, StorageId>,
    pub(super) entities: &'w [Entity],
    pub(super) row: usize,
}

impl<D: QueryData, F: QueryFilter> QueryIter<'_, '_, D, F> {
    #[cold]
    #[inline(never)]
    fn update_slice(&mut self) -> Option<()> {
        self.row = 0;
        loop {
            let id = *self.storages.next()?;
            if QueryState::<D, F>::IS_DENSE {
                let table_id = unsafe { id.table_id };
                let storages = unsafe { &self.world.read_only().storages };
                let table = unsafe { storages.tables.get_unchecked(table_id) };
                self.entities = table.entities();
                if !self.entities.is_empty() {
                    unsafe {
                        D::set_for_table(&self.state.d_state, &mut self.d_cache, table);
                        F::set_for_table(&self.state.f_state, &mut self.f_cache, table);
                    }
                    return Some(());
                }
            } else {
                let arche_id = unsafe { id.arche_id };
                let arches = unsafe { &self.world.read_only().archetypes };
                let arche = unsafe { arches.get_unchecked(arche_id) };
                self.entities = arche.entities();
                if !self.entities.is_empty() {
                    let table_id = arche.table_id();
                    let storages = unsafe { &self.world.read_only().storages };
                    let table = unsafe { storages.tables.get_unchecked(table_id) };
                    unsafe {
                        D::set_for_arche(&self.state.d_state, &mut self.d_cache, arche, table);
                        F::set_for_arche(&self.state.f_state, &mut self.f_cache, arche, table);
                    }
                    return Some(());
                }
            }
        }
    }
}

impl<'w, D: QueryData, F: QueryFilter> Iterator for QueryIter<'w, '_, D, F> {
    type Item = D::Item<'w>;

    fn next(&mut self) -> Option<Self::Item> {
        'looper: loop {
            if self.row >= self.entities.len() {
                // If there is no entities, `update_slice` will return None.
                // otherwise `self.entities` is not empty after this function.
                self.update_slice()?;
            }
            // we cannot storage old_row before `update_slice`,
            // because it will reset `self.row` always.
            let old_row = self.row;

            let entity = unsafe { *self.entities.get_unchecked(old_row) };
            // the number of entities < u32::MAX, the row will never wrapping.
            self.row += 1;

            let table_row = if QueryState::<D, F>::IS_DENSE {
                TableRow(old_row as u32)
            } else {
                let infos = unsafe { &self.world.read_only().entities };
                infos.get_spawned(entity).unwrap().table_row
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

impl<D: QueryData, F: QueryFilter> FusedIterator for QueryIter<'_, '_, D, F> {}

const EMPTY_ENTITIES: &[Entity] = &[];

impl<'w, 's, D: QueryData, F: QueryFilter> IntoIterator for Query<'w, 's, D, F> {
    type Item = D::Item<'w>;
    type IntoIter = QueryIter<'w, 's, D, F>;

    fn into_iter(self) -> Self::IntoIter {
        let last_run = self.last_run;
        let this_run = self.this_run;
        let world = self.world;
        let state = self.state;
        unsafe {
            QueryIter {
                world,
                state,
                d_cache: D::build_cache(&state.d_state, world, last_run, this_run),
                f_cache: F::build_cache(&state.f_state, world, last_run, this_run),
                storages: state.storages.iter(),
                entities: EMPTY_ENTITIES,
                row: 0,
            }
        }
    }
}

impl<D: QueryData, F: QueryFilter> QueryState<D, F> {
    pub fn iter_mut<'s, 'w>(&'s self, world: &'w mut World) -> QueryIter<'w, 's, D, F> {
        let last_run = world.last_run;
        let this_run = Tick::new(*world.this_run.get_mut());
        let world = world.unsafe_world();
        unsafe {
            QueryIter {
                world,
                state: self,
                d_cache: D::build_cache(&self.d_state, world, last_run, this_run),
                f_cache: F::build_cache(&self.f_state, world, last_run, this_run),
                storages: self.storages.iter(),
                entities: EMPTY_ENTITIES,
                row: 0,
            }
        }
    }

    pub fn iter<'s, 'w>(&'s self, world: &'w World) -> QueryIter<'w, 's, D, F>
    where
        D: ReadOnlyQuery,
    {
        let last_run = world.last_run;
        let this_run = Tick::new(world.this_run.load(Ordering::Relaxed));
        let world = world.unsafe_world();
        unsafe {
            QueryIter {
                world,
                state: self,
                d_cache: D::build_cache(&self.d_state, world, last_run, this_run),
                f_cache: F::build_cache(&self.f_state, world, last_run, this_run),
                storages: self.storages.iter(),
                entities: EMPTY_ENTITIES,
                row: 0,
            }
        }
    }
}

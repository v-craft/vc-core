#![expect(clippy::module_inception, reason = "For better structure.")]

use super::{QueryData, QueryFilter, QueryIter, QueryState};
use crate::entity::Entity;
use crate::system::{AccessTable, SystemParam};
use crate::tick::Tick;
use crate::world::{UnsafeWorld, World, WorldMode};

pub struct Query<'world, 'state, D: QueryData, F: QueryFilter = ()> {
    pub(super) world: UnsafeWorld<'world>,
    pub(super) state: &'state QueryState<D, F>,
    pub(super) last_run: Tick,
    pub(super) this_run: Tick,
}

unsafe impl<D: QueryData + 'static, F: QueryFilter + 'static> SystemParam for Query<'_, '_, D, F> {
    type State = QueryState<D, F>;
    type Item<'world, 'state> = Query<'world, 'state, D, F>;

    const WORLD_MODE: WorldMode = D::WORLD_MODE;
    const MAIN_THREAD: bool = false;

    unsafe fn init_state(world: &mut World) -> Self::State {
        QueryState::new(world)
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
        state.updata(unsafe { world.read_only() });
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

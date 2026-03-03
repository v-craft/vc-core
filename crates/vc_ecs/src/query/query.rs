#![expect(clippy::module_inception, reason = "For better structure.")]

use super::{QueryData, QueryFilter, QueryState};
use crate::query::ReadOnlyQuery;
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
        state.update(unsafe { world.read_only() });
        Query {
            world,
            state,
            last_run,
            this_run,
        }
    }
}

impl<D: ReadOnlyQuery, F: QueryFilter> Copy for Query<'_, '_, D, F> {}
impl<D: ReadOnlyQuery, F: QueryFilter> Clone for Query<'_, '_, D, F> {
    fn clone(&self) -> Self {
        *self
    }
}

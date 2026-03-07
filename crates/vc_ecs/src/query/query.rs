#![expect(clippy::module_inception, reason = "For better structure.")]

use core::fmt::Debug;

use super::{QueryData, QueryFilter, QueryState, ReadOnlyQueryData};
use crate::system::{AccessTable, ReadOnlySystemParam, SystemParam};
use crate::tick::Tick;
use crate::world::{UnsafeWorld, World};

pub struct Query<'world, 'state, D: QueryData, F: QueryFilter = ()> {
    pub(super) world: UnsafeWorld<'world>,
    pub(super) state: &'state QueryState<D, F>,
    pub(super) last_run: Tick,
    pub(super) this_run: Tick,
}

unsafe impl<D: QueryData + 'static, F: QueryFilter + 'static> SystemParam for Query<'_, '_, D, F> {
    type State = QueryState<D, F>;
    type Item<'world, 'state> = Query<'world, 'state, D, F>;

    const NON_SEND: bool = false;
    const EXCLUSIVE: bool = false;

    fn init_state(world: &mut World) -> Self::State {
        QueryState::new(world)
    }

    fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
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

impl<D: QueryData, F: QueryFilter> Debug for Query<'_, '_, D, F> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("Query")
            .field("state", &self.state)
            .field("last_run", &self.last_run)
            .field("this_run", &self.this_run)
            .finish()
    }
}

// -----------------------------------------------------------------------------
// ReadOnlyQuery

unsafe impl<D: ReadOnlyQueryData + 'static, F: QueryFilter + 'static> ReadOnlySystemParam for Query<'_, '_, D, F> {}

impl<D: ReadOnlyQueryData, F: QueryFilter> Copy for Query<'_, '_, D, F> {}

impl<D: ReadOnlyQueryData, F: QueryFilter> Clone for Query<'_, '_, D, F> {
    fn clone(&self) -> Self {
        *self
    }
}

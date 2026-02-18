use crate::entity::Entity;
use crate::storage::TableRow;
use crate::tick::Tick;
use crate::world::{AccessTable, UnsafeWorld, World, WorldMode};

pub unsafe trait QueryData {
    type State: Send + Sync + Sized;
    type Fetch<'world, 'state>: Clone;
    type Item<'world, 'state>;
    const IS_ARCHETYPAL: bool;
    const IS_DENSE: bool;
    const MODE: WorldMode;

    fn init_state(world: &mut World) -> Self::State;

    unsafe fn init_fetch<'w, 's>(
        world: UnsafeWorld<'w>,
        state: &'s Self::State,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Fetch<'w, 's>;

    unsafe fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool;

    unsafe fn fetch<'w, 's>(
        state: &'s Self::State,
        fetch: &mut Self::Fetch<'w, 's>,
        entity: Entity,
        table_row: TableRow,
    ) -> Option<Self::Item<'w, 's>>;
}

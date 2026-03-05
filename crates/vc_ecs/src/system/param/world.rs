use super::SystemParam;
use crate::system::AccessTable;
use crate::tick::Tick;
use crate::world::{UnsafeWorld, World};

// ---------------------------------------------------------
// World

unsafe impl SystemParam for &World {
    type State = ();
    type Item<'world, 'state> = &'world World;
    const NON_SEND: bool = false;
    const EXCLUSIVE: bool = false;

    unsafe fn init_state(_world: &mut World) -> Self::State {}

    unsafe fn mark_access(table: &mut AccessTable, _state: &Self::State) -> bool {
        table.set_world_ref()
    }

    unsafe fn get_param<'w, 's>(
        world: UnsafeWorld<'w>,
        _state: &'s mut Self::State,
        _last_run: Tick,
        _this_run: Tick,
    ) -> Self::Item<'w, 's> {
        unsafe { world.read_only() }
    }
}

unsafe impl SystemParam for &mut World {
    type State = ();
    type Item<'world, 'state> = &'world mut World;
    const NON_SEND: bool = false;
    // We hold the mutable borrowing of this world,
    // so we cannot parallelize with other systems.
    const EXCLUSIVE: bool = true;

    unsafe fn init_state(_world: &mut World) -> Self::State {}

    unsafe fn mark_access(table: &mut AccessTable, _state: &Self::State) -> bool {
        table.set_world_mut()
    }

    unsafe fn get_param<'w, 's>(
        world: UnsafeWorld<'w>,
        _state: &'s mut Self::State,
        _last_run: Tick,
        _this_run: Tick,
    ) -> Self::Item<'w, 's> {
        unsafe { world.full_mut() }
    }
}

use super::SystemParam;
use crate::system::AccessTable;
use crate::tick::Tick;
use crate::world::{UnsafeWorld, World, WorldMode};

// ---------------------------------------------------------
// World

unsafe impl SystemParam for &World {
    type State = ();
    type Item<'world, 'state> = &'world World;
    const WORLD_MODE: WorldMode = WorldMode::ReadOnly;
    const MAIN_THREAD: bool = false;

    unsafe fn init_state(_world: &mut World) -> Self::State {}

    unsafe fn mark_access(table: &mut AccessTable, _state: &Self::State) -> bool {
        if table.can_world_ref() {
            table.set_world_ref();
            true
        } else {
            false
        }
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
    const WORLD_MODE: WorldMode = WorldMode::FullMut;
    const MAIN_THREAD: bool = false;

    unsafe fn init_state(_world: &mut World) -> Self::State {}

    unsafe fn mark_access(table: &mut AccessTable, _state: &Self::State) -> bool {
        if table.can_world_mut() {
            table.set_world_mut();
            true
        } else {
            false
        }
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

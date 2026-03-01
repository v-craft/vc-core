use super::SystemParam;
use crate::system::AccessTable;
use crate::tick::Tick;
use crate::world::{UnsafeWorld, World, WorldMode};

pub struct MainThread;

unsafe impl SystemParam for MainThread {
    type State = ();
    type Item<'world, 'state> = MainThread;
    const WORLD_MODE: WorldMode = WorldMode::ReadOnly;
    const MAIN_THREAD: bool = true;

    unsafe fn init_state(_: &mut World) -> Self::State {}

    unsafe fn mark_access(_: &mut AccessTable, _: &Self::State) -> bool {
        true
    }

    unsafe fn get_param<'w, 's>(
        _world: UnsafeWorld<'w>,
        _state: &'s mut Self::State,
        _last_run: Tick,
        _this_run: Tick,
    ) -> Self::Item<'w, 's> {
        MainThread
    }
}

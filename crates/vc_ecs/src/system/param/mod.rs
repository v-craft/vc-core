#![allow(clippy::missing_safety_doc, reason = "todo")]

// -----------------------------------------------------------------------------
// Modules

mod main_thread;
mod resource;
mod world;

pub use main_thread::MainThread;

// -----------------------------------------------------------------------------
// SystemParam

use super::AccessTable;
use crate::tick::Tick;
use crate::world::{UnsafeWorld, World, WorldMode};

pub unsafe trait SystemParam: Sized {
    type State: Send + Sync + 'static;
    type Item<'world, 'state>: SystemParam<State = Self::State>;
    const WORLD_MODE: WorldMode;
    const MAIN_THREAD: bool;

    unsafe fn init_state(world: &mut World) -> Self::State;
    unsafe fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool;

    unsafe fn get_param<'w, 's>(
        world: UnsafeWorld<'w>,
        state: &'s mut Self::State,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Item<'w, 's>;
}

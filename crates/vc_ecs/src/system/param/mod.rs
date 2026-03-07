#![allow(clippy::missing_safety_doc, reason = "todo")]

// -----------------------------------------------------------------------------
// Modules

mod local;
mod marker;
mod resource;
mod tuples;
mod world;

// -----------------------------------------------------------------------------
// marker

pub use local::Local;
pub use marker::{Exclusive, MainThread, NonSend};

// -----------------------------------------------------------------------------
// SystemParam

use super::AccessTable;
use crate::tick::Tick;
use crate::world::{UnsafeWorld, World};

pub unsafe trait SystemParam: Sized {
    type State: Send + Sync + 'static;
    type Item<'world, 'state>: SystemParam<State = Self::State>;
    const NON_SEND: bool;
    const EXCLUSIVE: bool;

    fn init_state(world: &mut World) -> Self::State;
    fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool;

    unsafe fn get_param<'w, 's>(
        world: UnsafeWorld<'w>,
        state: &'s mut Self::State,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Item<'w, 's>;
}

pub unsafe trait ReadOnlySystemParam: SystemParam {}

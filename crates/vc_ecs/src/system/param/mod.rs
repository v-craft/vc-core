#![allow(clippy::missing_safety_doc, reason = "todo")]

// -----------------------------------------------------------------------------
// Modules

mod main_thread;
mod resource;
mod world;

pub use main_thread::MainThread;
use vc_utils::range_invoke;

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

// -----------------------------------------------------------------------------
// Tuple

macro_rules! impl_tuple {
    (0: []) => {
        unsafe impl SystemParam for () {
            type State = ();
            type Item<'world, 'state> = ();

            const WORLD_MODE: WorldMode = WorldMode::ReadOnly;
            const MAIN_THREAD: bool = false;

            unsafe fn init_state(_world: &mut World) -> Self::State {}

            unsafe fn mark_access(_table: &mut AccessTable, _state: &Self::State) -> bool { true }

            unsafe fn get_param<'w, 's>(
                _world: UnsafeWorld<'w>,
                _state: &'s mut Self::State,
                _last_run: Tick,
                _this_run: Tick,
            ) -> Self::Item<'w, 's> {}
        }
    };
    (1 : [ $index:tt : $name:ident ]) => {
        #[cfg_attr(docsrs, doc(fake_variadic))]
        #[cfg_attr(docsrs, doc = "This trait is implemented for tuples up to 12 items long.")]
        unsafe impl<$name: SystemParam> SystemParam for ($name,) {
            type State = <$name>::State;
            type Item<'world, 'state> = ( <$name>::Item<'world, 'state>, );

            const WORLD_MODE: WorldMode = <$name>::WORLD_MODE;
            const MAIN_THREAD: bool = <$name>::MAIN_THREAD;

            unsafe fn init_state(world: &mut World) -> Self::State {
                unsafe { <$name>::init_state(world) }
            }

            unsafe fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
                unsafe { <$name>::mark_access(table, state) }
            }

            unsafe fn get_param<'w, 's>(
                world: UnsafeWorld<'w>,
                state: &'s mut Self::State,
                last_run: Tick,
                this_run: Tick,
            ) -> Self::Item<'w, 's> {
                unsafe { ( <$name>::get_param(world, state, last_run, this_run), ) }
            }
        }
    };
    ($num:literal : [$($index:tt : $name:ident),*]) => {
        #[cfg_attr(docsrs, doc(hidden))]
        unsafe impl<$($name: SystemParam),*> SystemParam for ($($name),*) {
            type State = ( $( <$name>::State ),* );
            type Item<'world, 'state> = ( $( <$name>::Item<'world, 'state> ),* );

            const WORLD_MODE: WorldMode = {
                WorldMode::ReadOnly $( .merge(<$name>::WORLD_MODE) )*
            };
            const MAIN_THREAD: bool = {
                false $( || <$name>::MAIN_THREAD )*
            };

            unsafe fn init_state(world: &mut World) -> Self::State {
                unsafe { ( $( <$name>::init_state(world) ),* ) }
            }

            unsafe fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
                unsafe {
                    true $( && <$name>::mark_access(table, &state.$index) )*
                }
            }

            unsafe fn get_param<'w, 's>(
                world: UnsafeWorld<'w>,
                state: &'s mut Self::State,
                last_run: Tick,
                this_run: Tick,
            ) -> Self::Item<'w, 's> {
                unsafe { ( $( <$name>::get_param(world, &mut state.$index, last_run, this_run) ),* ) }
            }
        }
    };
}

range_invoke! {
    impl_tuple, 12: P
}

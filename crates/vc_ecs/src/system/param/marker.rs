use super::{SystemParam, ReadOnlySystemParam};
use crate::system::AccessTable;
use crate::tick::Tick;
use crate::world::{UnsafeWorld, World};

// -----------------------------------------------------------------------------
// MainThread

pub struct MainThread;

unsafe impl ReadOnlySystemParam for MainThread {}

unsafe impl SystemParam for MainThread {
    type State = ();
    type Item<'world, 'state> = MainThread;
    const NON_SEND: bool = true;
    const EXCLUSIVE: bool = false;

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

// -----------------------------------------------------------------------------
// NonSend

pub struct NonSend;

unsafe impl ReadOnlySystemParam for NonSend {}

unsafe impl SystemParam for NonSend {
    type State = ();
    type Item<'world, 'state> = NonSend;
    const NON_SEND: bool = true;
    const EXCLUSIVE: bool = false;

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
        NonSend
    }
}

// -----------------------------------------------------------------------------
// Exclusive

pub struct Exclusive;

unsafe impl ReadOnlySystemParam for Exclusive {}

unsafe impl SystemParam for Exclusive {
    type State = ();
    type Item<'world, 'state> = Exclusive;
    const NON_SEND: bool = false;
    const EXCLUSIVE: bool = true;

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
        Exclusive
    }
}

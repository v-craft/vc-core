use super::{ReadOnlySystemParam, SystemParam};
use crate::error::EcsError;
use crate::system::AccessTable;
use crate::tick::Tick;
use crate::world::{UnsafeWorld, World};

// -----------------------------------------------------------------------------
// MainThreadMarker

pub struct MainThreadMarker;

unsafe impl ReadOnlySystemParam for MainThreadMarker {}

unsafe impl SystemParam for MainThreadMarker {
    type State = ();
    type Item<'world, 'state> = MainThreadMarker;
    const NON_SEND: bool = true;
    const EXCLUSIVE: bool = false;

    fn init_state(_: &mut World) -> Self::State {}

    fn mark_access(_: &mut AccessTable, _: &Self::State) -> bool {
        true
    }

    unsafe fn build_param<'w, 's>(
        _world: UnsafeWorld<'w>,
        _state: &'s mut Self::State,
        _last_run: Tick,
        _this_run: Tick,
    ) -> Result<Self::Item<'w, 's>, EcsError> {
        Ok(MainThreadMarker)
    }
}

// -----------------------------------------------------------------------------
// NonSendMarker

pub struct NonSendMarker;

unsafe impl ReadOnlySystemParam for NonSendMarker {}

unsafe impl SystemParam for NonSendMarker {
    type State = ();
    type Item<'world, 'state> = NonSendMarker;
    const NON_SEND: bool = true;
    const EXCLUSIVE: bool = false;

    fn init_state(_: &mut World) -> Self::State {}

    fn mark_access(_: &mut AccessTable, _: &Self::State) -> bool {
        true
    }

    unsafe fn build_param<'w, 's>(
        _world: UnsafeWorld<'w>,
        _state: &'s mut Self::State,
        _last_run: Tick,
        _this_run: Tick,
    ) -> Result<Self::Item<'w, 's>, EcsError> {
        Ok(NonSendMarker)
    }
}

// -----------------------------------------------------------------------------
// ExclusiveMarker

pub struct ExclusiveMarker;

unsafe impl ReadOnlySystemParam for ExclusiveMarker {}

unsafe impl SystemParam for ExclusiveMarker {
    type State = ();
    type Item<'world, 'state> = ExclusiveMarker;
    const NON_SEND: bool = false;
    const EXCLUSIVE: bool = true;

    fn init_state(_: &mut World) -> Self::State {}

    fn mark_access(_: &mut AccessTable, _: &Self::State) -> bool {
        true
    }

    unsafe fn build_param<'w, 's>(
        _world: UnsafeWorld<'w>,
        _state: &'s mut Self::State,
        _last_run: Tick,
        _this_run: Tick,
    ) -> Result<Self::Item<'w, 's>, EcsError> {
        Ok(ExclusiveMarker)
    }
}

#![expect(unsafe_code, reason = "UnsafeCell")]

use core::cell::UnsafeCell;
use core::fmt;
use core::marker::PhantomData;
use core::ptr;

use super::World;
use crate::entity::{Entity, EntityLocation};
use crate::tick::Tick;

// -----------------------------------------------------------------------------
// UnsafeWorldCell

#[derive(Copy, Clone)]
pub struct UnsafeWorldCell<'w> {
    _marker: PhantomData<&'w UnsafeCell<World>>,
    ptr: *mut World,
    #[cfg(any(debug_assertions, feature = "debug"))]
    allows_mutable_access: bool,
}

unsafe impl Send for UnsafeWorldCell<'_> {}
unsafe impl Sync for UnsafeWorldCell<'_> {}

// -----------------------------------------------------------------------------
// UnsafeEntityCell

#[derive(Copy, Clone)]
pub struct UnsafeEntityCell<'w> {
    world: UnsafeWorldCell<'w>,
    entity: Entity,
    location: EntityLocation,
    last_run: Tick,
    this_run: Tick,
}

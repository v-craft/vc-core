use core::cell::UnsafeCell;
use core::panic::Location;

use crate::tick::Tick;
use crate::utils::DebugLocation;

// -----------------------------------------------------------------------------
// ComponentTicks

/// Records when a component or resource was added
/// and when it was last mutably dereferenced (or added).
#[derive(Debug, Copy, Clone)]
pub struct ComponentTicks {
    pub added: Tick,
    pub changed: Tick,
}

impl ComponentTicks {
    #[inline(always)]
    pub const fn new(tick: Tick) -> Self {
        Self {
            added: tick,
            changed: tick,
        }
    }

    #[inline]
    pub const fn is_added(&self, last_run: Tick, this_run: Tick) -> bool {
        self.added.is_newer_than(last_run, this_run)
    }

    #[inline]
    pub const fn is_changed(&self, last_run: Tick, this_run: Tick) -> bool {
        self.changed.is_newer_than(last_run, this_run)
    }
}

// -----------------------------------------------------------------------------
// ComponentTickCells

/// Interior-mutable access to the [`Tick`]s of a single component or resource.
#[derive(Debug, Copy, Clone)]
pub struct ComponentTickCells<'a> {
    pub added: &'a UnsafeCell<Tick>,
    pub changed: &'a UnsafeCell<Tick>,
    pub changed_by: DebugLocation<&'a UnsafeCell<&'static Location<'static>>>,
}

// -----------------------------------------------------------------------------
// ComponentTickRef

#[derive(Debug, Clone)]
pub(crate) struct ComponentTicksRef<'w> {
    pub(crate) added: &'w Tick,
    pub(crate) changed: &'w Tick,
    pub(crate) changed_by: DebugLocation<&'w &'static Location<'static>>,
    pub(crate) last_run: Tick,
    pub(crate) this_run: Tick,
}

// -----------------------------------------------------------------------------
// ComponentTicksMut

#[derive(Debug)]
pub(crate) struct ComponentTicksMut<'w> {
    pub(crate) added: &'w mut Tick,
    pub(crate) changed: &'w mut Tick,
    pub(crate) changed_by: DebugLocation<&'w mut &'static Location<'static>>,
    pub(crate) last_run: Tick,
    pub(crate) this_run: Tick,
}

impl<'w> From<ComponentTicksMut<'w>> for ComponentTicksRef<'w> {
    #[inline(always)]
    fn from(this: ComponentTicksMut<'w>) -> Self {
        ComponentTicksRef {
            added: this.added,
            changed: this.changed,
            changed_by: this.changed_by.map(|changed_by| &*changed_by),
            last_run: this.last_run,
            this_run: this.this_run,
        }
    }
}

// -----------------------------------------------------------------------------
// ComponentTicksSliceRef

#[derive(Debug, Clone)]
pub(crate) struct ComponentTicksSliceRef<'w> {
    pub(crate) added: &'w [Tick],
    pub(crate) changed: &'w [Tick],
    pub(crate) changed_by: DebugLocation<&'w [&'static Location<'static>]>,
    pub(crate) last_run: Tick,
    pub(crate) this_run: Tick,
}

// -----------------------------------------------------------------------------
// ComponentTicksSliceMut

#[derive(Debug)]
pub(crate) struct ComponentTicksSliceMut<'w> {
    pub(crate) added: &'w mut [Tick],
    pub(crate) changed: &'w mut [Tick],
    pub(crate) changed_by: DebugLocation<&'w mut [&'static Location<'static>]>,
    pub(crate) last_run: Tick,
    pub(crate) this_run: Tick,
}

impl<'w> From<ComponentTicksSliceMut<'w>> for ComponentTicksSliceRef<'w> {
    #[inline(always)]
    fn from(this: ComponentTicksSliceMut<'w>) -> Self {
        ComponentTicksSliceRef {
            added: this.added,
            changed: this.changed,
            changed_by: this.changed_by.map(|changed_by| &*changed_by),
            last_run: this.last_run,
            this_run: this.this_run,
        }
    }
}

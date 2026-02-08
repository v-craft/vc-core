#![expect(unsafe_code, reason = "need UnsafeCell")]

use core::cell::UnsafeCell;
use core::panic::Location;

use vc_utils::UnsafeCellDeref;

use crate::tick::Tick;
use crate::utils::DebugLocation;

// -----------------------------------------------------------------------------
// ComponentTicks

#[derive(Copy, Clone, Debug)]
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

    #[inline(always)]
    pub const fn set_changed(&mut self, change_tick: Tick) {
        self.changed = change_tick;
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

#[derive(Copy, Clone, Debug)]
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

impl<'w> ComponentTicksRef<'w> {
    #[allow(unused, reason = "todo")]
    #[inline]
    pub unsafe fn from_tick_cells(
        cells: ComponentTickCells<'w>,
        last_run: Tick,
        this_run: Tick,
    ) -> Self {
        unsafe {
            Self {
                added: cells.added.deref(),
                changed: cells.changed.deref(),
                changed_by: cells.changed_by.map(|cell| cell.deref()),
                last_run,
                this_run,
            }
        }
    }
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

impl<'w> ComponentTicksMut<'w> {
    #[inline]
    pub unsafe fn from_tick_cells(
        cells: ComponentTickCells<'w>,
        last_run: Tick,
        this_run: Tick,
    ) -> Self {
        unsafe {
            Self {
                added: cells.added.deref_mut(),
                changed: cells.changed.deref_mut(),
                changed_by: cells.changed_by.map(|cell| cell.deref_mut()),
                last_run,
                this_run,
            }
        }
    }
}

impl<'w> From<ComponentTicksMut<'w>> for ComponentTicksRef<'w> {
    #[inline(always)]
    fn from(ticks: ComponentTicksMut<'w>) -> Self {
        ComponentTicksRef {
            added: ticks.added,
            changed: ticks.changed,
            changed_by: ticks.changed_by.map(|changed_by| &*changed_by),
            last_run: ticks.last_run,
            this_run: ticks.this_run,
        }
    }
}

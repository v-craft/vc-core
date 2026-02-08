//! Tick is a timing mechanism in the game world, primarily used for change detection.
//!
//! It is a 32-bit integer representing a point in time, can easily overflow
//! over time and is therefore unsuitable for timeline synchronization. Its main
//! purpose is to work within the change detection system.
//!
//! The World maintains a dynamically changing `Tick` object `now`, representing
//! the current moment. When `now` overflows, it wraps around, so we must also
//! constrain the maximum age difference.
//!
//! We periodically check the Tick markers of all components every `CHECK_CYCLE`
//! ticks to ensure their "age" does not exceed `MAX_TICK_AGE`.

// -----------------------------------------------------------------------------
// Configuration

/// Check cycle for component age validation (prevents overflow issues)
pub const CHECK_CYCLE: u32 = 1 << 29;

/// Maximum allowable Tick age - values exceeding this are clamped to this limit
pub const MAX_TICK_AGE: u32 = u32::MAX - (CHECK_CYCLE << 1) - 1;

// -----------------------------------------------------------------------------
// Tick

/// A 32-bit integer representing a discrete point in time
///
/// Primarily used for change detection mechanisms. Not suitable for timeline
/// synchronization across different clients, as Tick progression rates cannot
/// be guaranteed to match between different instances.
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Tick(u32);

impl Tick {
    /// Maximum valid Tick value, equivalent to [`MAX_TICK_AGE`]
    pub const MAX_AGE: Self = Self::new(MAX_TICK_AGE);

    /// Creates a new `Tick` instance
    #[inline(always)]
    pub const fn new(tick: u32) -> Self {
        Self(tick)
    }

    /// Retrieves the underlying u32 value
    #[inline(always)]
    pub const fn get(self) -> u32 {
        self.0
    }

    /// Sets the Tick value
    #[inline(always)]
    pub const fn set(&mut self, tick: u32) {
        self.0 = tick;
    }

    /// Calculates the relative age difference from another Tick
    #[inline(always)]
    pub const fn relative_to(self, other: Self) -> Self {
        Self(self.0.wrapping_sub(other.0))
    }

    /// Determines whether this Tick is newer than another relative
    /// to current time
    #[inline]
    pub const fn is_newer_than(self, other: Tick, now: Tick) -> bool {
        #[inline(always)]
        const fn min(x: u32, y: u32) -> u32 {
            if x < y { x } else { y }
        }

        let since_insert = min(now.relative_to(self).0, MAX_TICK_AGE);
        let since_system = min(now.relative_to(other).0, MAX_TICK_AGE);

        since_system > since_insert
    }

    /// Validates and clamps age relative to current time
    #[inline]
    pub const fn check_age(&mut self, now: Tick) -> bool {
        let age = now.relative_to(*self);
        // must be `>` instead of `>=`.
        if age.0 > MAX_TICK_AGE {
            *self = now.relative_to(Tick::MAX_AGE);
            true
        } else {
            false
        }
    }

    /// `fall_back == now.relative_to(Tick::MAX_AGE)`
    #[inline(always)]
    pub(crate) fn quick_check(&mut self, now: Tick, fall_back: Tick) {
        let age = now.relative_to(*self);
        if age.0 > MAX_TICK_AGE {
            *self = fall_back;
        }
    }

    pub(crate) fn slice_check(this: &mut [Tick], now: Tick) {
        let arr: &mut [u32] = unsafe { core::mem::transmute(this) };
        let now: u32 = unsafe { core::mem::transmute(now) };

        let fall_back = now.wrapping_sub(MAX_TICK_AGE);

        // `for_each` can generate better code than explicit `for` loops.
        arr.iter_mut().for_each(|x| {
            let age: u32 = now.wrapping_sub(*x);
            if age > MAX_TICK_AGE {
                *x = fall_back;
            }
        });
    }
}

impl core::hash::Hash for Tick {
    #[inline(always)]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        state.write_u32(self.0);
    }
}

impl core::fmt::Debug for Tick {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Tick").field(&self.0).finish()
    }
}

// -----------------------------------------------------------------------------
// CheckTicks

/// Event triggered for periodic Tick age validation
#[derive(Debug, Clone, Copy)]
pub struct CheckTicks(Tick);

impl CheckTicks {
    #[inline(always)]
    pub const fn new(tick: Tick) -> Self {
        CheckTicks(tick)
    }

    #[inline(always)]
    pub const fn tick(self) -> Tick {
        self.0
    }
}

// -----------------------------------------------------------------------------
// DetectChanges

pub trait DetectChanges {
    /// Returns `true` if this value was added after the system last ran.
    fn is_added(&self) -> bool;

    /// Returns `true` if this value was added or mutably dereferenced
    /// either since the last time the system ran or, if the system never ran,
    /// since the beginning of the program.
    fn is_changed(&self) -> bool;

    /// Returns the change tick recording the time this data was added.
    fn added_tick(&self) -> Tick;

    /// Returns the change tick recording the time this data was most recently changed.
    ///
    /// Note that components and resources are also marked as changed upon insertion.
    fn changed_tick(&self) -> Tick;
}

// -----------------------------------------------------------------------------
// TicksBorrow

use vc_ptr::{ThinSlice, ThinSliceMut};

// -------------------------------------------------------------------
// TicksRef

#[derive(Debug, Clone)]
pub struct TicksRef<'w> {
    pub added: &'w Tick,
    pub changed: &'w Tick,
    pub last_run: Tick,
    pub this_run: Tick,
}

// -------------------------------------------------------------------
// TicksMut

#[derive(Debug)]
pub struct TicksMut<'w> {
    pub added: &'w mut Tick,
    pub changed: &'w mut Tick,
    pub last_run: Tick,
    pub this_run: Tick,
}

impl<'w> From<TicksMut<'w>> for TicksRef<'w> {
    #[inline(always)]
    fn from(this: TicksMut<'w>) -> Self {
        TicksRef {
            added: this.added,
            changed: this.changed,
            last_run: this.last_run,
            this_run: this.this_run,
        }
    }
}

// -------------------------------------------------------------------
// TicksSliceRef

#[derive(Debug, Clone)]
pub struct TicksSliceRef<'w> {
    pub length: usize,
    pub added: ThinSlice<'w, Tick>,
    pub changed: ThinSlice<'w, Tick>,
    pub last_run: Tick,
    pub this_run: Tick,
}

// -------------------------------------------------------------------
// TicksSliceMut

#[derive(Debug)]
pub struct TicksSliceMut<'w> {
    pub length: usize,
    pub added: ThinSliceMut<'w, Tick>,
    pub changed: ThinSliceMut<'w, Tick>,
    pub last_run: Tick,
    pub this_run: Tick,
}

impl<'w> From<TicksSliceMut<'w>> for TicksSliceRef<'w> {
    #[inline(always)]
    fn from(this: TicksSliceMut<'w>) -> Self {
        TicksSliceRef {
            length: this.length,
            added: this.added.into(),
            changed: this.changed.into(),
            last_run: this.last_run,
            this_run: this.this_run,
        }
    }
}

#![allow(unused, reason = "todo")]

// -----------------------------------------------------------------------------
// Modules

mod event;
mod observer;
mod trigger;

// -----------------------------------------------------------------------------
// Exports

pub use event::{EntityEvent, Event, SetEntityEventTarget};
pub use observer::{
    CachedComponentObservers, CachedObservers, ObserverMap, ObserverRunner, Observers,
};
pub use trigger::{Trigger, TriggerContext};

// -----------------------------------------------------------------------------
// Inline - Exports

use crate::component::ComponentId;

#[derive(Debug, Copy, Clone, Ord, PartialOrd)]
#[repr(transparent)]
pub struct EventKey(pub(crate) ComponentId);

impl core::hash::Hash for EventKey {
    #[inline(always)]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        state.write_u32(self.0.index_u32());
    }
}

impl PartialEq for EventKey {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.0.index_u32() == other.0.index_u32()
    }
}

impl Eq for EventKey {}

impl core::fmt::Display for EventKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(&self.0, f)
    }
}

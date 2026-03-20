use alloc::boxed::Box;
use core::fmt::Debug;

use super::{Direction, GraphNode};
use crate::system::{AccessTable, System};

// -----------------------------------------------------------------------------
// SystemKey

slotmap::new_key_type! {
    pub struct SystemKey;
}

impl GraphNode for SystemKey {
    type Link = (SystemKey, Direction);
    type Edge = (SystemKey, SystemKey);
}

// -----------------------------------------------------------------------------
// SystemObject

pub type UnitSystem = Box<dyn System<Input = (), Output = ()>>;

/// A bundle of `Box<dyn System>` and it's `AccessTable`.
pub struct SystemObject {
    pub system: UnitSystem,
    pub access: AccessTable,
}

impl SystemObject {
    #[inline]
    pub fn new_uninit(system: UnitSystem) -> Self {
        Self {
            system,
            access: AccessTable::new(),
        }
    }
}

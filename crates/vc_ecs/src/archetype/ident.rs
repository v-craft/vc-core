use core::fmt::{Debug, Display};
use core::hash::Hash;

use vc_utils::num::NonMaxU32;

// -----------------------------------------------------------------------------
// ArcheId

/// Unique identifier for a Archetype.
#[derive(Copy, Clone, PartialOrd, Ord)]
#[repr(transparent)]
pub struct ArcheId(NonMaxU32);

impl ArcheId {
    pub const EMPTY: ArcheId = ArcheId(NonMaxU32::ZERO);

    #[inline(always)]
    pub(crate) const fn new(id: u32) -> Self {
        Self(NonMaxU32::new(id).expect("too many archetypes"))
    }

    /// Returns the archetype index as a usize.
    #[inline(always)]
    pub const fn index(self) -> usize {
        self.0.get() as usize
    }
}

impl Debug for ArcheId {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.0.get(), f)
    }
}

impl Display for ArcheId {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.0.get(), f)
    }
}

impl Hash for ArcheId {
    #[inline(always)]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        // we do not use underlying value here,
        // then `SparseHash` is faster.
        state.write_u32(self.0.get());
    }
}

impl PartialEq for ArcheId {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for ArcheId {}

// -----------------------------------------------------------------------------
// ArcheId

/// Row position within a table.
///
/// Represents an index into a table's columnar storage.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct ArcheRow(pub u32);

impl Debug for ArcheRow {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for ArcheRow {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Hash for ArcheRow {
    #[inline(always)]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        state.write_u32(self.0);
    }
}

use core::fmt;
use core::hash;

// -----------------------------------------------------------------------------
// ArchetypeId

/// An opaque unique ID for a single `Archetype` within a `World`.
///
/// Archetype IDs are only valid for a given World, and are not globally unique.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct ArchetypeId(u32);

impl ArchetypeId {
    pub const EMPTY: ArchetypeId = ArchetypeId(0);

    #[inline(always)]
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    #[inline(always)]
    pub const fn index_u32(self) -> u32 {
        self.0
    }

    #[inline(always)]
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

impl PartialEq for ArchetypeId {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for ArchetypeId {}

impl hash::Hash for ArchetypeId {
    #[inline(always)]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        state.write_u32(self.0);
    }
}

impl fmt::Display for ArchetypeId {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

// -----------------------------------------------------------------------------
// ArchetypeRow

use nonmax::NonMaxU32;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct ArchetypeRow(NonMaxU32);

impl ArchetypeRow {
    #[inline(always)]
    pub const fn new(index: NonMaxU32) -> Self {
        Self(index)
    }

    #[inline(always)]
    pub const fn index_u32(self) -> u32 {
        self.0.get()
    }

    #[inline(always)]
    pub const fn index(self) -> usize {
        self.0.get() as usize
    }
}

impl PartialEq for ArchetypeRow {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.0.get() == other.0.get()
    }
}

impl Eq for ArchetypeRow {}

impl hash::Hash for ArchetypeRow {
    #[inline(always)]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        state.write_u32(self.0.get());
    }
}

impl fmt::Display for ArchetypeRow {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0.get(), f)
    }
}

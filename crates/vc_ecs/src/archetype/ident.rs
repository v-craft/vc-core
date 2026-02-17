use core::fmt;
use core::hash;
use core::mem;

use nonmax::NonMaxU32;

// -----------------------------------------------------------------------------
// ArchetypeId

#[derive(Debug, Copy, Clone, PartialOrd, Ord)]
#[repr(transparent)]
pub struct ArchetypeId(NonMaxU32);

impl ArchetypeId {
    pub const PLACEHOLDER: ArchetypeId = ArchetypeId(NonMaxU32::MAX);
    pub const EMPTY: ArchetypeId = ArchetypeId(NonMaxU32::ZERO);

    #[inline(always)]
    pub(crate) const fn new(index: NonMaxU32) -> Self {
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

impl fmt::Display for ArchetypeId {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl hash::Hash for ArchetypeId {
    #[inline(always)]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        // we do not use underlying value here,
        // then `SparseHash` is faster.
        state.write_u32(self.0.get());
    }
}

impl PartialEq for ArchetypeId {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        unsafe { mem::transmute_copy::<Self, u32>(self) == mem::transmute_copy::<Self, u32>(other) }
    }
}

impl Eq for ArchetypeId {}

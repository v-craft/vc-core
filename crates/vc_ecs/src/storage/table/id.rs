use core::fmt;
use core::hash;

// -----------------------------------------------------------------------------
// TableId

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct TableId(u32);

impl TableId {
    pub const EMPTY: TableId = TableId(0);

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

impl fmt::Display for TableId {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl hash::Hash for TableId {
    #[inline(always)]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        state.write_u32(self.0);
    }
}

// -----------------------------------------------------------------------------
// TableRow

use nonmax::NonMaxU32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct TableRow(NonMaxU32);

impl TableRow {
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

impl fmt::Display for TableRow {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0.get(), f)
    }
}

impl hash::Hash for TableRow {
    #[inline(always)]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        state.write_u32(self.0.get());
    }
}

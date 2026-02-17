use core::fmt;
use core::hash;

// -----------------------------------------------------------------------------
// BundleId

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct BundleId(u32);

impl BundleId {
    pub const PLACEHOLDER: BundleId = BundleId(u32::MAX);
    pub const EMPTY: BundleId = BundleId(0);

    #[inline(always)]
    pub(crate) const fn new(index: u32) -> Self {
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

impl fmt::Display for BundleId {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl hash::Hash for BundleId {
    #[inline(always)]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        state.write_u32(self.0);
    }
}

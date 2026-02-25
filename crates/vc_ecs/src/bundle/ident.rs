use core::fmt::{Debug, Display};
use core::hash::Hash;

// -----------------------------------------------------------------------------
// BundleId

/// Unique identifier for a Bundle.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct BundleId(u32);

impl BundleId {
    pub const EMPTY: BundleId = BundleId(0);

    #[inline(always)]
    pub(crate) const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Returns the bundle index as a usize.
    #[inline(always)]
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

impl Debug for BundleId {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for BundleId {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Hash for BundleId {
    #[inline(always)]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        state.write_u32(self.0);
    }
}

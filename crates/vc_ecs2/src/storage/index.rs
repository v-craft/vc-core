// -----------------------------------------------------------------------------
// StorageType

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
pub enum StorageType {
    #[default]
    Table,
    SparseSet,
}

// -----------------------------------------------------------------------------
// StorageIndex

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct StorageIndex(u32);

impl StorageIndex {
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

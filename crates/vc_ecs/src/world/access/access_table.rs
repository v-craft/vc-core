use core::fmt::Debug;

use fixedbitset::FixedBitSet;

use crate::component::ComponentId;

#[derive(Default)]
pub struct AccessTable {
    pub(crate) full_mut: bool, // holding `&mut world`
    pub(crate) read_all: bool, // holding `&world`
    pub(crate) reading: FixedBitSet,
    pub(crate) writing: FixedBitSet,
}

// `#[derive(Clone)]` does not generate optimized `clone_from`.
impl Clone for AccessTable {
    fn clone(&self) -> Self {
        Self {
            full_mut: self.full_mut,
            read_all: self.read_all,
            reading: self.reading.clone(),
            writing: self.writing.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.full_mut = source.full_mut;
        self.read_all = source.read_all;
        self.reading.clone_from(&source.reading);
        self.writing.clone_from(&source.writing);
    }
}

impl Debug for AccessTable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        struct FormattedBitSet<'a>(&'a FixedBitSet);
        impl Debug for FormattedBitSet<'_> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_list().entries(self.0.ones()).finish()
            }
        }

        f.debug_struct("AccessTable")
            .field("full_mut", &self.full_mut)
            .field("read_all", &self.read_all)
            .field("reading", &FormattedBitSet(&self.reading))
            .field("writing", &FormattedBitSet(&self.writing))
            .finish()
    }
}

impl AccessTable {
    /// Creates an empty [`Access`] collection.
    pub const fn new() -> Self {
        Self {
            full_mut: false,
            read_all: false,
            reading: FixedBitSet::new(),
            writing: FixedBitSet::new(),
        }
    }

    pub fn is_readable(&self, id: ComponentId) -> bool {
        if self.read_all {
            true
        } else if self.full_mut {
            false
        } else {
            !self.writing.contains(id.index())
        }
    }

    pub unsafe fn set_reading(&mut self, id: ComponentId) {
        self.reading.insert(id.index());
    }

    pub fn is_writable(&self, id: ComponentId) -> bool {
        if self.read_all || self.full_mut {
            false
        } else {
            // writing includes reading, so we just check reading.
            !self.reading.contains(id.index())
        }
    }

    pub unsafe fn set_writing(&mut self, id: ComponentId) {
        self.reading.insert(id.index());
        self.writing.insert(id.index());
    }

    pub fn full_mutable(&self) -> bool {
        if self.full_mut || self.read_all {
            return false;
        }
        !self.reading.contains_any_in_range(..) && !self.writing.contains_any_in_range(..)
    }

    pub unsafe fn set_full_mut(&mut self) {
        self.full_mut = true;
        self.reading = FixedBitSet::new();
        self.writing = FixedBitSet::new();
    }

    pub fn all_readable(&self) -> bool {
        if self.read_all {
            return true;
        }
        !(self.full_mut || self.writing.contains_any_in_range(..))
    }

    pub unsafe fn set_read_all(&mut self) {
        self.read_all = true;
        self.reading = FixedBitSet::new();
        self.writing = FixedBitSet::new();
    }
}

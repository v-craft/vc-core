use alloc::vec::Vec;
use core::fmt::{self, Debug};

use fixedbitset::FixedBitSet;

use crate::component::ComponentId;

// -----------------------------------------------------------------------------
// Access

#[derive(PartialEq, Eq, Default)]
pub struct Access {
    pub(crate) component_reads: FixedBitSet,
    pub(crate) component_writes: FixedBitSet,
    pub(crate) resource_reads: FixedBitSet,
    pub(crate) resource_writes: FixedBitSet,
    pub(crate) component_reads_inverted: bool,
    pub(crate) component_writes_inverted: bool,
    pub(crate) read_all_resources: bool,
    pub(crate) write_all_resources: bool,
    pub(crate) archetypal: FixedBitSet,
}

// -----------------------------------------------------------------------------
// AccessFilters

#[derive(PartialEq, Eq, Default)]
pub struct AccessFilters {
    pub(crate) with: FixedBitSet,
    pub(crate) without: FixedBitSet,
}

// -----------------------------------------------------------------------------
// ComponentAccessKind

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ComponentAccessKind {
    /// Archetypical access, such as `Has<Foo>`.
    Archetypal(ComponentId),
    /// Shared access, such as `&Foo`.
    Shared(ComponentId),
    /// Exclusive access, such as `&mut Foo`.
    Exclusive(ComponentId),
}

// -----------------------------------------------------------------------------
// AccessConflicts

#[derive(Debug, PartialEq)]
pub enum AccessConflicts {
    All,
    Individual(FixedBitSet),
}

// -----------------------------------------------------------------------------
// FilteredAccess

#[derive(Debug, PartialEq, Eq)]
pub struct FilteredAccess {
    pub(crate) access: Access,
    pub(crate) required: FixedBitSet,
    pub(crate) filter_sets: Vec<AccessFilters>,
}

// -----------------------------------------------------------------------------
// FilteredAccessSet

#[derive(Debug, PartialEq, Eq, Default)]
pub struct FilteredAccessSet {
    pub(crate) combined_access: Access,
    pub(crate) filtered_accesses: Vec<FilteredAccess>,
}

/// A wrapper struct to make Debug representations
/// of [`FixedBitSet`] easier to read.
struct FormattedBits<'a>(&'a FixedBitSet);

impl<'a> Debug for FormattedBits<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.0.ones()).finish()
    }
}

impl Clone for Access {
    fn clone(&self) -> Self {
        Self {
            component_reads: self.component_reads.clone(),
            component_writes: self.component_writes.clone(),
            resource_reads: self.resource_reads.clone(),
            resource_writes: self.resource_writes.clone(),
            component_reads_inverted: self.component_reads_inverted,
            component_writes_inverted: self.component_writes_inverted,
            read_all_resources: self.read_all_resources,
            write_all_resources: self.write_all_resources,
            archetypal: self.archetypal.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.component_reads.clone_from(&source.component_reads);
        self.component_writes.clone_from(&source.component_writes);
        self.resource_reads.clone_from(&source.resource_reads);
        self.resource_writes.clone_from(&source.resource_writes);
        self.component_reads_inverted = source.component_reads_inverted;
        self.component_writes_inverted = source.component_writes_inverted;
        self.read_all_resources = source.read_all_resources;
        self.write_all_resources = source.write_all_resources;
        self.archetypal.clone_from(&source.archetypal);
    }
}

impl Debug for Access {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Access")
            .field("component_reads", &FormattedBits(&self.component_reads))
            .field("component_writes", &FormattedBits(&self.component_writes))
            .field("resource_reads", &FormattedBits(&self.resource_reads))
            .field("resource_writes", &FormattedBits(&self.resource_writes))
            .field("component_reads_inverted", &self.component_reads_inverted)
            .field("component_writes_inverted", &self.component_writes_inverted)
            .field("read_all_resources", &self.read_all_resources)
            .field("write_all_resources", &self.write_all_resources)
            .field("archetypal", &FormattedBits(&self.archetypal))
            .finish()
    }
}

impl Clone for AccessFilters {
    fn clone(&self) -> Self {
        Self {
            with: self.with.clone(),
            without: self.without.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.with.clone_from(&source.with);
        self.without.clone_from(&source.without);
    }
}

impl Debug for AccessFilters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AccessFilters")
            .field("with", &FormattedBits(&self.with))
            .field("without", &FormattedBits(&self.without))
            .finish()
    }
}

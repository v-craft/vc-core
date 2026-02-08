use core::ops::{Index, IndexMut};

use alloc::boxed::Box;
use alloc::vec::Vec;

use vc_utils::hash::{HashMap, SparseHashSet};

use super::{Archetype, ArchetypeId};
use crate::component::{ComponentId, ComponentIndices};

// -----------------------------------------------------------------------------
// ArchetypeComponents

#[derive(Hash, PartialEq, Eq)]
pub struct ArchetypeComponents {
    table_components: Box<[ComponentId]>,
    sparse_set_components: Box<[ComponentId]>,
}

// -----------------------------------------------------------------------------
// Archetypes

pub struct Archetypes {
    pub archetypes: Vec<Archetype>,
    pub precise_map: HashMap<ArchetypeComponents, ArchetypeId>,
    pub rough_table: Vec<SparseHashSet<ArchetypeId>>,
    pub rough_map: ComponentIndices,
}

use alloc::boxed::Box;
use alloc::vec::Vec;

use vc_utils::hash::SparseHashMap;

use super::{ArchetypeEntity, ArchetypeFlags, ArchetypeId, Edges};
use crate::component::ComponentId;
use crate::entity::Entity;
use crate::storage::{StorageIndex, TableId, TableRow};

// -----------------------------------------------------------------------------
// ArchetypeSwapRemoveResult

pub struct ArchetypeSwapRemoveResult {
    pub swapped_entity: Option<Entity>,
    pub table_row: TableRow,
}

// -----------------------------------------------------------------------------
// Archetype

pub struct Archetype {
    pub(crate) id: ArchetypeId,
    pub(crate) edges: Edges,
    pub(crate) flags: ArchetypeFlags,
    pub(crate) table_id: TableId,
    pub(crate) entities: Vec<ArchetypeEntity>,
    pub(crate) component_ids: Box<[ComponentId]>,
    pub(crate) storage_indices: SparseHashMap<ComponentId, StorageIndex>,
}

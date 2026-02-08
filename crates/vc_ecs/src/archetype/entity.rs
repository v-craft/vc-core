use crate::entity::Entity;
use crate::storage::TableRow;

// -----------------------------------------------------------------------------
// ArchetypeEntity

pub struct ArchetypeEntity {
    pub entity: Entity,
    pub table_row: TableRow,
}

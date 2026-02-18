use crate::entity::Entity;
use crate::storage::{SparseSets, StorageIndex, Table, TableRemoveResult};
use crate::world::World;

impl World {
    pub fn despawn(&mut self, entity: Entity) {
        let location = unsafe { self.entities.set_despawned(entity) };
        let archetype_info = unsafe { self.archetypes.get(location.archetype_id) };

        {
            let table: &mut Table = unsafe { self.storages.tables.get_mut(location.table_id) };
            let remove_result: TableRemoveResult =
                unsafe { table.swap_remove_and_drop(location.table_row) };
            if let Some(swapped) = remove_result.swapped {
                let swapped_info = unsafe { self.entities.get_mut(swapped.id()) };
                swapped_info.location.table_row = location.table_row;
            }
        }

        {
            let sparse_set: &mut SparseSets = &mut self.storages.sparse_sets;
            let storage_indices: &[StorageIndex] = &archetype_info.storage_indices;
            let in_table = archetype_info.in_table as usize;
            let len = archetype_info.components.len();

            (in_table..len).for_each(|index| unsafe {
                sparse_set.remove_and_drop(storage_indices[index], entity.id());
            });
        }
    }
}

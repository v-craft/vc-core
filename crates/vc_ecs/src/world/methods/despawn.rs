use crate::entity::{Entity, EntityError};
use crate::utils::DebugCheckedUnwrap;
use crate::world::World;

impl World {
    pub fn despawn(&mut self, entity: Entity) -> Result<(), EntityError> {
        let location = unsafe { self.entities.set_despawned(entity)? };

        let arche_id = location.arche_id;
        let arche_row = location.arche_row;
        let archetype = unsafe { self.archetypes.get_unchecked_mut(arche_id) };
        let arche_moved = unsafe { archetype.swap_remove(arche_row) };

        let table_id = location.table_id;
        let table_row = location.table_row;
        let table = unsafe { self.storages.tables.get_unchecked_mut(table_id) };
        let table_moved = unsafe { table.swap_remove_and_drop(table_row) };

        let maps = &mut self.storages.maps;
        archetype
            .sparse_components()
            .iter()
            .for_each(|&cid| unsafe {
                let map_id = maps.get_id(cid).debug_checked_unwrap();
                let map = maps.get_unchecked_mut(map_id);
                let map_row = map.get_map_row(entity).debug_checked_unwrap();
                map.drop_item(map_row);
            });

        let new_entity = unsafe { self.entities.free(entity.id(), 1) };
        self.allocator.free(new_entity);

        match (arche_moved, table_moved) {
            (None, None) => Ok(()),
            (None, Some(moved)) => unsafe { self.entities.move_spawned(moved) },
            (Some(moved), None) => unsafe { self.entities.move_spawned(moved) },
            (Some(moved1), Some(moved2)) => {
                let res1 = unsafe { self.entities.move_spawned(moved1) };
                let res2 = unsafe { self.entities.move_spawned(moved2) };
                res1?;
                res2
            }
        }
    }
}

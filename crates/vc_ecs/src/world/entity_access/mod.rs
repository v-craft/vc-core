use crate::entity::{Entity, EntityLocation};
use crate::world::World;

pub struct EntityWorldMut<'w> {
    world: &'w mut World,
    entity: Entity,
    location: Option<EntityLocation>,
}

// pub struct FilteredEntityRef<'w, 's> {
//     entity: UnsafeEntityCell<'w>,
//     access: &'s Access,
// }

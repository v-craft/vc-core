// -----------------------------------------------------------------------------
// Modules

mod despawn;
mod insert;
mod spawn;

// -----------------------------------------------------------------------------
// Worlds

use super::WorldId;

use crate::archetype::Archetypes;
use crate::bundle::Bundles;
use crate::component::{CompIdAllocator, Components};
use crate::entity::{Entities, EntityAllocator};
use crate::storage::Storages;
use crate::tick::Tick;

pub struct World {
    id: WorldId,
    pub(crate) now: Tick,
    pub(crate) archetypes: Archetypes,
    pub(crate) bundles: Bundles,
    pub(crate) storages: Storages,
    pub(crate) entities: Entities,
    pub(crate) components: Components,
    pub(crate) entity_allocator: EntityAllocator,
    pub(crate) compid_allocator: CompIdAllocator,
}

impl core::fmt::Debug for World {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("World")
            .field("id", &self.id)
            .field("now", &self.now)
            .field("archetypes", &self.archetypes)
            .field("bundles", &self.bundles)
            .field("storages", &self.storages)
            .field("entities", &self.entities)
            .field("components", &self.components)
            .field("entity_allocator", &self.entity_allocator)
            .field("compid_allocator", &self.compid_allocator)
            .finish_non_exhaustive()
    }
}

impl World {
    pub fn new(id: WorldId) -> World {
        Self {
            id,
            now: Tick::new(0),
            archetypes: Archetypes::new(),
            bundles: Bundles::new(),
            storages: Storages::new(),
            entities: Entities::new(),
            components: Components::new(),
            entity_allocator: EntityAllocator::new(),
            compid_allocator: CompIdAllocator::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::world::{World, WorldId};

    #[test]
    fn new() {
        let world = World::new(WorldId::new(1));
        let _ = alloc::format!("{world:?}");

        // std::eprintln!("{world:?}");
    }
}

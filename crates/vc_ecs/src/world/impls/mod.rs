// -----------------------------------------------------------------------------
// Modules

mod despawn;
mod insert;
mod register;
mod spawn;

// -----------------------------------------------------------------------------
// Worlds

use vc_os::sync::atomic::AtomicU32;

use super::WorldId;

use crate::archetype::Archetypes;
use crate::bundle::Bundles;
use crate::component::{CompIdAllocator, Components};
use crate::entity::{Entities, EntityAllocator};
use crate::storage::Storages;
use crate::tick::Tick;

pub struct World {
    pub(crate) id: WorldId,
    pub(crate) now_tick: AtomicU32,
    pub(crate) last_change: Tick,
    pub(crate) last_check: Tick,
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
            .field("now_tick", &self.now_tick)
            .field("last_change", &self.last_change)
            .field("last_check", &self.last_check)
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
            now_tick: AtomicU32::new(0),
            last_change: Tick::new(0),
            last_check: Tick::new(0),
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

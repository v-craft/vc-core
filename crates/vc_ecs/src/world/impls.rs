use alloc::boxed::Box;
use core::fmt::Debug;

use vc_os::sync::atomic::AtomicU32;

use crate::archetype::Archetypes;
use crate::bundle::Bundles;
use crate::component::Components;
use crate::entity::{Entities, EntityAllocator};
use crate::resource::Resources;
use crate::storage::Storages;
use crate::tick::Tick;
use crate::world::WorldId;

pub struct World {
    id: WorldId,
    pub(crate) thread_hash: u64,
    pub entities: Entities,
    pub allocator: EntityAllocator,
    pub components: Components,
    pub resources: Resources,
    pub storages: Storages,
    pub bundles: Bundles,
    pub archetypes: Archetypes,
    pub(crate) this_run: AtomicU32,
    pub(crate) last_run: Tick,
}

impl Debug for World {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("World")
            .field("id", &self.id)
            .field("thread_hash", &self.thread_hash)
            .field("entities", &self.entities)
            .field("allocator", &self.allocator)
            .field("components", &self.components)
            .field("resources", &self.resources)
            .field("storages", &self.storages)
            .field("bundles", &self.bundles)
            .field("archetypes", &self.archetypes)
            .finish()
    }
}

impl World {
    pub fn new(id: WorldId) -> Box<World> {
        Box::new(Self {
            id,
            thread_hash: crate::utils::thread_hash(),
            entities: Entities::new(),
            allocator: EntityAllocator::new(),
            components: Components::new(),
            resources: Resources::new(),
            storages: Storages::new(),
            bundles: Bundles::new(),
            archetypes: Archetypes::new(),
            this_run: AtomicU32::new(1),
            last_run: Tick::new(0),
        })
    }

    pub fn id(&self) -> WorldId {
        self.id
    }
}

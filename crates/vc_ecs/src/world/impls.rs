use alloc::boxed::Box;
use core::fmt::Debug;
use core::sync::atomic::Ordering;

use vc_os::sync::atomic::AtomicU32;

use crate::archetype::Archetypes;
use crate::bundle::Bundles;
use crate::component::Components;
use crate::entity::{Entities, EntityAllocator};
use crate::resource::Resources;
use crate::storage::Storages;
use crate::tick::{CHECK_CYCLE, CheckTicks, Tick};
use crate::world::WorldId;

pub struct World {
    id: WorldId,
    pub(crate) thread_hash: u64,
    pub(crate) entities: Entities,
    pub(crate) allocator: EntityAllocator,
    pub(crate) components: Components,
    pub(crate) resources: Resources,
    pub(crate) storages: Storages,
    pub(crate) bundles: Bundles,
    pub(crate) archetypes: Archetypes,
    pub(crate) this_run: AtomicU32,
    pub(crate) last_run: Tick,
    pub(crate) last_check: Tick,
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
            last_check: Tick::new(0),
        })
    }

    pub fn id(&self) -> WorldId {
        self.id
    }

    pub fn last_run(&self) -> Tick {
        self.last_run
    }

    pub fn this_run(&self) -> Tick {
        Tick::new(self.this_run.load(Ordering::Relaxed))
    }

    pub fn advance_tick(&self) -> Tick {
        Tick::new(self.this_run.fetch_add(1, Ordering::Relaxed))
    }

    pub fn update_tick(&mut self) {
        let last = *self.this_run.get_mut();
        self.last_run = Tick::new(last);
        *self.this_run.get_mut() = last.wrapping_add(1);
    }

    pub fn check_ticks(&mut self) -> Option<CheckTicks> {
        let this_run = *self.this_run.get_mut();
        let this_run = Tick::new(this_run);
        if this_run.relative_to(self.last_check).get() < CHECK_CYCLE {
            return None;
        }

        let checker = CheckTicks::new(this_run);
        self.storages.check_ticks(checker);
        self.last_check = this_run;

        Some(checker)
    }
}

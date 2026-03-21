#![expect(clippy::module_inception, reason = "For better structure.")]

use core::fmt::Debug;
use core::sync::atomic::Ordering;

use vc_os::sync::atomic::AtomicU32;

use crate::archetype::Archetypes;
use crate::bundle::Bundles;
use crate::command::CommandQueue;
use crate::component::Components;
use crate::entity::{Entities, Entity, EntityAllocator};
use crate::error::{DefaultErrorHandler, ErrorContext};
use crate::resource::Resources;
use crate::storage::Storages;
use crate::tick::{CHECK_CYCLE, CheckTicks, Tick};
use crate::world::{EntityMut, EntityOwned, EntityRef, WorldId, WorldIdAllocator};

// -----------------------------------------------------------------------------
// World

/// Central ECS container holding entities, type registries, and all storages.
///
/// A [`World`] owns:
/// - entity allocation/state,
/// - component/resource metadata registries,
/// - dense/sparse storage backends,
/// - bundle/archetype metadata,
/// - change-detection ticks.
///
/// Most high-level ECS operations (`spawn`, `despawn`, resource access, query
/// construction) are methods on this type.
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
    pub(crate) command_queue: CommandQueue,
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
            .field("command_queue", &self.command_queue)
            .finish()
    }
}

impl Default for World {
    fn default() -> Self {
        static DEFAULT_ALLOCATOR: WorldIdAllocator = WorldIdAllocator::new();
        World::new(DEFAULT_ALLOCATOR.alloc())
    }
}

impl World {
    /// Creates a new world with the given unique id.
    pub fn new(id: WorldId) -> World {
        Self {
            id,
            thread_hash: crate::utils::thread_hash(),
            entities: Entities::new(),
            allocator: EntityAllocator::new(),
            components: Components::new(),
            resources: Resources::new(),
            storages: Storages::new(),
            bundles: Bundles::new(),
            archetypes: Archetypes::new(),
            command_queue: CommandQueue::new(),
            this_run: AtomicU32::new(1),
            last_run: Tick::new(0),
            last_check: Tick::new(0),
        }
    }

    /// Returns this world's unique id.
    pub fn id(&self) -> WorldId {
        self.id
    }

    /// Returns the tick used as `last_run` for change detection.
    pub fn last_run(&self) -> Tick {
        self.last_run
    }

    /// Returns the current world tick (`this_run`).
    pub fn this_run(&self) -> Tick {
        Tick::new(self.this_run.load(Ordering::Relaxed))
    }

    /// Returns the thread hash captured when the world was created.
    pub fn thread_hash(&self) -> u64 {
        self.thread_hash
    }

    /// Returns the entity storage.
    pub fn entities(&self) -> &Entities {
        &self.entities
    }

    /// Returns mutable access to the entity storage.
    pub fn entities_mut(&mut self) -> &mut Entities {
        &mut self.entities
    }

    /// Returns the entity id allocator.
    pub fn allocator(&self) -> &EntityAllocator {
        &self.allocator
    }

    /// Returns mutable access to the entity id allocator.
    pub fn allocator_mut(&mut self) -> &mut EntityAllocator {
        &mut self.allocator
    }

    /// Returns the component registry.
    pub fn components(&self) -> &Components {
        &self.components
    }

    /// Returns mutable access to the component registry.
    pub fn components_mut(&mut self) -> &mut Components {
        &mut self.components
    }

    /// Returns the resource registry.
    pub fn resources(&self) -> &Resources {
        &self.resources
    }

    /// Returns mutable access to the resource registry.
    pub fn resources_mut(&mut self) -> &mut Resources {
        &mut self.resources
    }

    /// Returns all storage backends.
    pub fn storages(&self) -> &Storages {
        &self.storages
    }

    /// Returns mutable access to all storage backends.
    pub fn storages_mut(&mut self) -> &mut Storages {
        &mut self.storages
    }

    /// Returns the bundle registry.
    pub fn bundles(&self) -> &Bundles {
        &self.bundles
    }

    /// Returns mutable access to the bundle registry.
    pub fn bundles_mut(&mut self) -> &mut Bundles {
        &mut self.bundles
    }

    /// Returns the archetype registry.
    pub fn archetypes(&self) -> &Archetypes {
        &self.archetypes
    }

    /// Returns mutable access to the archetype registry.
    pub fn archetypes_mut(&mut self) -> &mut Archetypes {
        &mut self.archetypes
    }

    /// Returns the archetype registry.
    pub fn command_queue(&self) -> &CommandQueue {
        &self.command_queue
    }
}

// -----------------------------------------------------------------------------
// Basic

impl World {
    pub fn alloc_entity(&self) -> Entity {
        self.allocator.alloc()
    }

    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    pub fn entity_owned(&mut self, entity: Entity) -> EntityOwned<'_> {
        let location = self.entities.get_spawned(entity).unwrap();
        EntityOwned {
            world: self.into(),
            entity,
            location,
        }
    }

    pub fn entity_mut(&mut self, entity: Entity) -> EntityMut<'_> {
        let location = self.entities.get_spawned(entity).unwrap();
        let last_run = self.last_run();
        let this_run = self.this_run();
        EntityMut {
            world: self,
            entity,
            location,
            last_run,
            this_run,
        }
    }

    pub fn entity_ref(&self, entity: Entity) -> EntityRef<'_> {
        let location = self.entities.get_spawned(entity).unwrap();
        let last_run = self.last_run();
        let this_run = self.this_run();
        EntityRef {
            world: self,
            entity,
            location,
            last_run,
            this_run,
        }
    }

    pub fn advance_tick(&self) -> Tick {
        Tick::new(self.this_run.fetch_add(1, Ordering::Relaxed))
    }

    pub fn update_tick(&mut self) -> Tick {
        let last_run = *self.this_run.get_mut();
        let this_run = last_run.wrapping_add(1);

        self.last_run = Tick::new(last_run);
        *self.this_run.get_mut() = this_run;

        if this_run.wrapping_sub(last_run) >= CHECK_CYCLE {
            vc_utils::cold_path();
            self.check_ticks();
        }

        Tick::new(this_run)
    }

    pub fn check_ticks(&mut self) -> CheckTicks {
        let this_run = Tick::new(*self.this_run.get_mut());
        let checker = CheckTicks::new(this_run);
        self.storages.check_ticks(checker);
        self.last_check = this_run;
        checker
    }
}

// -----------------------------------------------------------------------------
// Advance

impl World {
    pub fn default_error_handler(&self) -> DefaultErrorHandler {
        self.get_resource::<DefaultErrorHandler>()
            .copied()
            .unwrap_or_default()
    }

    pub fn apply_commands(&mut self) {
        let handler = self.default_error_handler();

        while let Some(cmd) = self.command_queue.pop() {
            let location = cmd.location();
            if let Err(err) = cmd.run(self) {
                vc_utils::cold_path();
                let this_run = self.this_run();
                let ctx = ErrorContext::Command { location, this_run };
                (handler)(err, ctx);
            }
        }
    }
}

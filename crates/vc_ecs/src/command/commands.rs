use core::fmt::Debug;

use alloc::vec::Vec;

use super::CommandObject;
use crate::bundle::Bundle;
use crate::command::EntityCommands;
use crate::entity::Entity;
use crate::error::EcsError;
use crate::system::{AccessTable, ReadOnlySystemParam, SystemParam};
use crate::tick::Tick;
use crate::world::{UnsafeWorld, World, WorldId};

pub struct Commands<'a> {
    world: &'a World,
    buffer: Vec<CommandObject>,
}

unsafe impl ReadOnlySystemParam for Commands<'_> {}

unsafe impl SystemParam for Commands<'_> {
    type State = ();
    type Item<'world, 'state> = Commands<'world>;
    const NON_SEND: bool = false;
    const EXCLUSIVE: bool = false;

    fn init_state(_world: &mut World) -> Self::State {}

    fn mark_access(_table: &mut AccessTable, _state: &Self::State) -> bool {
        true
    }

    unsafe fn build_param<'w, 's>(
        world: UnsafeWorld<'w>,
        _state: &'s mut Self::State,
        _last_run: Tick,
        _this_run: Tick,
    ) -> Result<Self::Item<'w, 's>, EcsError> {
        Ok(Commands {
            world: unsafe { world.read_only() },
            buffer: Vec::new(),
        })
    }
}

impl Debug for Commands<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Commands")
            .field("world", &self.world_id())
            .finish()
    }
}

impl Drop for Commands<'_> {
    fn drop(&mut self) {
        self.flush();
    }
}

impl<'a> Commands<'a> {
    pub fn flush(&mut self) {
        if !self.buffer.is_empty() {
            let commands = ::core::mem::take(&mut self.buffer);
            self.world.command_queue.extend(commands);
        }
    }

    pub fn new(world: &'a World) -> Self {
        Self {
            world,
            buffer: Vec::new(),
        }
    }

    pub fn world_id(&self) -> WorldId {
        self.world.id()
    }

    pub fn reborrow(&mut self) -> Commands<'_> {
        self.flush();
        Commands {
            world: self.world,
            buffer: Vec::new(),
        }
    }

    #[inline]
    #[track_caller]
    pub fn push<F>(&mut self, func: F)
    where
        F: Send + 'static,
        F: FnOnce(&mut World) -> Result<(), EcsError>,
    {
        self.buffer.push(CommandObject::new(func));
    }

    pub fn alloc_entity(&self) -> Entity {
        self.world.alloc_entity()
    }

    #[inline]
    #[track_caller]
    pub fn spawn_in<B: Bundle>(&mut self, bundle: B, entity: Entity) -> EntityCommands<'_> {
        self.buffer.push(CommandObject::new(move |world| {
            world.entities.can_spawned(entity)?;
            world.spawn_in(bundle, entity);
            Ok(())
        }));

        self.with_entity(entity)
    }

    #[inline]
    #[track_caller]
    pub fn spawn<B: Bundle>(&mut self, bundle: B) -> EntityCommands<'_> {
        let entity = self.world.alloc_entity();

        self.buffer.push(CommandObject::new(move |world| {
            world.spawn_in(bundle, entity);
            Ok(())
        }));

        self.with_entity(entity)
    }

    #[inline]
    #[track_caller]
    pub fn despawn(&mut self, entity: Entity) {
        self.buffer.push(CommandObject::new(move |world| {
            world.despawn(entity).map_err(Into::into)
        }));
    }

    #[inline]
    #[track_caller]
    pub fn try_despawn(&mut self, entity: Entity) {
        self.buffer.push(CommandObject::new(move |world| {
            let _ = world.despawn(entity);
            Ok(())
        }));
    }

    #[inline]
    pub fn with_entity(&mut self, entity: Entity) -> EntityCommands<'_> {
        // We need to flush in advance to ensure that the
        // entity spawn is earlier than all entity operation.
        self.flush();

        EntityCommands {
            entity,
            commands: Commands::new(self.world),
        }
    }
}

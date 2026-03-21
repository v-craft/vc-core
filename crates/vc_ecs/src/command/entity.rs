use core::fmt::Debug;

use super::Commands;
use crate::bundle::Bundle;
use crate::entity::Entity;
use crate::error::EcsError;
use crate::world::{EntityOwned, WorldId};

pub struct EntityCommands<'a> {
    pub(super) entity: Entity,
    pub(super) commands: Commands<'a>,
}

impl Debug for EntityCommands<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("EntityCommand")
            .field("world", &self.commands.world_id())
            .field("entity", &self.entity)
            .finish()
    }
}

impl<'a> EntityCommands<'a> {
    pub fn flush(&mut self) {
        self.commands.flush();
    }

    pub fn world_id(&self) -> WorldId {
        self.commands.world_id()
    }

    pub fn entity(&self) -> Entity {
        self.entity
    }

    pub fn reborrow(&mut self) -> EntityCommands<'_> {
        EntityCommands {
            entity: self.entity,
            commands: self.commands.reborrow(),
        }
    }

    #[inline]
    #[track_caller]
    pub fn push<F>(&mut self, func: F)
    where
        F: Send + 'static,
        F: FnOnce(EntityOwned) -> Result<(), EcsError>,
    {
        let entity = self.entity;
        self.commands.push(move |world| {
            let location = world.entities.get_spawned(entity)?;
            func(EntityOwned {
                world: world.into(),
                entity,
                location,
            })
        });
    }

    #[inline]
    #[track_caller]
    pub fn despawn(&mut self) {
        self.commands.despawn(self.entity);
    }

    #[inline]
    #[track_caller]
    pub fn try_despawn(&mut self) {
        self.commands.try_despawn(self.entity);
    }

    #[inline]
    #[track_caller]
    pub fn insert<B: Bundle>(&mut self, bundle: B) {
        self.push(move |mut entity| {
            entity.insert(bundle);
            Ok(())
        });
    }

    #[inline]
    #[track_caller]
    pub fn remove<B: Bundle>(&mut self) {
        self.push(move |mut entity| {
            entity.remove::<B>();
            Ok(())
        });
    }
}

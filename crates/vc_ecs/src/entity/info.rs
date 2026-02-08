use core::fmt::Debug;

use alloc::vec::Vec;

use crate::archetype::{ArcheId, ArcheRow};
use crate::entity::error::{DespawnError, FetchError, MoveError, SpawnError};
use crate::entity::{Entity, EntityError, EntityGeneration, EntityId};
use crate::storage::{TableId, TableRow};

// -----------------------------------------------------------------------------
// EntityToken

#[derive(Debug, Clone, Copy)]
pub struct EntityLocation {
    pub arche_id: ArcheId,
    pub table_id: TableId,
    pub arche_row: ArcheRow,
    pub table_row: TableRow,
}

// -----------------------------------------------------------------------------
// EntityInfo

#[derive(Debug, Clone, Copy)]
struct EntityInfo {
    generation: EntityGeneration,
    location: Option<EntityLocation>,
}

// -----------------------------------------------------------------------------
// Entities

pub struct Entities {
    infos: Vec<EntityInfo>,
}

impl Debug for Entities {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list()
            .entries(
                self.infos
                    .iter()
                    .enumerate()
                    .filter(|(_, info)| info.location.is_some())
                    .map(|(id, info)| alloc::format!("{id}v{}", info.generation)),
            )
            .finish()
    }
}

impl Entities {
    pub(crate) const fn new() -> Self {
        Self { infos: Vec::new() }
    }

    pub fn get_spawned(&self, entity: Entity) -> Result<EntityLocation, EntityError> {
        let Some(info) = self.infos.get(entity.index()) else {
            return Err(FetchError::NotFound(entity.id()).into());
        };
        if info.generation != entity.generation() {
            return Err(FetchError::Mismatch {
                expect: entity,
                actual: Entity::new(entity.id(), info.generation),
            }
            .into());
        }
        info.location.ok_or(FetchError::NotSpawned(entity).into())
    }

    #[cold]
    #[inline(never)]
    fn resize(&mut self, len: usize) {
        self.infos.reserve(len - self.infos.len());
        self.infos.resize(
            self.infos.capacity(),
            const {
                EntityInfo {
                    generation: EntityGeneration::FIRST,
                    location: None,
                }
            },
        );
    }

    pub fn resolve(&self, id: EntityId) -> Entity {
        if let Some(info) = self.infos.get(id.index()) {
            Entity::new(id, info.generation)
        } else {
            Entity::from_id(id)
        }
    }

    /// # Safety
    /// Ensure by caller.
    pub unsafe fn free(&mut self, id: EntityId, generation: u32) -> Entity {
        let index = id.index();
        if index >= self.infos.len() {
            self.resize(index + 1);
        }

        let info = unsafe { self.infos.get_unchecked_mut(index) };
        debug_assert!(info.location.is_none());

        let (new_gen, wrapping) = info.generation.checked_add(generation);
        info.generation = new_gen;
        if wrapping {
            log::warn!("Entity({id}) generation wrapped on Entities::free, aliasing may occur.");
        }

        Entity::new(id, new_gen)
    }

    pub fn can_spawned(&mut self, entity: Entity) -> Result<(), EntityError> {
        let index = entity.index();
        if index >= self.infos.len() {
            self.resize(index + 1);
        }

        let info = unsafe { self.infos.get_unchecked(index) };
        if info.location.is_some() {
            return Err(SpawnError::AlreadySpawned(entity).into());
        }
        if info.generation != entity.generation() {
            return Err(SpawnError::Mismatch {
                expect: entity,
                actual: Entity::new(entity.id(), info.generation),
            }
            .into());
        }

        Ok(())
    }

    /// # Safety
    /// Ensure by caller.
    pub unsafe fn set_spawned(
        &mut self,
        entity: Entity,
        location: EntityLocation,
    ) -> Result<(), EntityError> {
        let index = entity.index();
        if index >= self.infos.len() {
            self.resize(index + 1);
        }

        let info = unsafe { self.infos.get_unchecked_mut(index) };
        if info.generation != entity.generation() {
            return Err(SpawnError::Mismatch {
                expect: entity,
                actual: Entity::new(entity.id(), info.generation),
            }
            .into());
        }
        if info.location.is_some() {
            return Err(SpawnError::AlreadySpawned(entity).into());
        }

        info.location = Some(location);
        Ok(())
    }

    /// # Safety
    /// Ensure by caller.
    pub unsafe fn set_despawned(&mut self, entity: Entity) -> Result<EntityLocation, EntityError> {
        let Some(info) = self.infos.get_mut(entity.index()) else {
            return Err(DespawnError::NotFound(entity.id()).into());
        };
        if info.generation != entity.generation() {
            return Err(DespawnError::Mismatch {
                expect: entity,
                actual: Entity::new(entity.id(), info.generation),
            }
            .into());
        }

        core::mem::take(&mut info.location).ok_or(DespawnError::NotSpawned(entity).into())
    }

    /// # Safety
    /// Ensure by caller.
    pub unsafe fn move_spawned(&mut self, moved: MovedEntity) -> Result<(), EntityError> {
        let entity = moved.entity;

        let Some(info) = self.infos.get_mut(entity.index()) else {
            return Err(MoveError::NotFound(entity.id()).into());
        };
        if info.generation != entity.generation() {
            return Err(MoveError::Mismatch {
                expect: entity,
                actual: Entity::new(entity.id(), info.generation),
            }
            .into());
        }
        let Some(location) = &mut info.location else {
            return Err(MoveError::NotSpawned(entity).into());
        };
        match moved.new_row {
            Row::Arche(arche_row) => location.arche_row = arche_row,
            Row::Table(table_row) => location.table_row = table_row,
        }
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// Update Row

#[derive(Debug, Clone, Copy)]
enum Row {
    Arche(ArcheRow),
    Table(TableRow),
}

#[derive(Debug, Clone, Copy)]
pub struct MovedEntity {
    entity: Entity,
    new_row: Row,
}

impl MovedEntity {
    #[inline(always)]
    pub const fn in_table(entity: Entity, row: TableRow) -> Self {
        Self {
            entity,
            new_row: Row::Table(row),
        }
    }
    #[inline(always)]
    pub const fn in_arche(entity: Entity, row: ArcheRow) -> Self {
        Self {
            entity,
            new_row: Row::Arche(row),
        }
    }
}

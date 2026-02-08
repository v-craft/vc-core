#![expect(unsafe_code, reason = "Need unsafe code.")]

use core::panic::Location;

use alloc::vec::Vec;

use super::error::{InvalidEntityError, ValidEntityButNotSpawnedError};
use super::error::{NotSpawnedError, SpawnError};
use super::{Entity, EntityGeneration, EntityId, EntityLocation};
use crate::cfg;
use crate::tick::{CheckTicks, Tick};
use crate::utils::DebugLocation;

// -----------------------------------------------------------------------------
// SpawnedOrDespawned

#[derive(Debug, Copy, Clone)]
struct SpawnedOrDespawned {
    tick: Tick,
    by: DebugLocation,
}

// -----------------------------------------------------------------------------
// EntityMeta

#[derive(Debug, Copy, Clone)]
struct EntityMeta {
    generation: EntityGeneration,
    location: Option<EntityLocation>,
    spawned_or_despawned: SpawnedOrDespawned,
}

impl EntityMeta {
    const FRESH: EntityMeta = EntityMeta {
        generation: EntityGeneration::FIRST,
        location: None,
        spawned_or_despawned: SpawnedOrDespawned {
            by: DebugLocation::caller(),
            tick: Tick::new(0),
        },
    };
}

// -----------------------------------------------------------------------------
// Entities

#[derive(Debug, Clone)]
pub struct Entities {
    meta: Vec<EntityMeta>,
}

impl Entities {
    #[inline]
    pub const fn empty() -> Self {
        Self { meta: Vec::new() }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.meta.clear();
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.meta.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.meta.len() == 0
    }

    #[inline]
    pub fn check_ticks(&mut self, check: CheckTicks) {
        let tick = check.tick();
        for meta in &mut self.meta {
            meta.spawned_or_despawned.tick.check_age(tick);
        }
    }

    #[inline]
    pub fn get_location_spawned(&self, entity: Entity) -> Result<EntityLocation, NotSpawnedError> {
        let meta = self.meta.get(entity.index()).unwrap_or(&EntityMeta::FRESH);

        if entity.generation() != meta.generation {
            return Err(NotSpawnedError::Invalid(InvalidEntityError {
                entity,
                current_generation: meta.generation,
            }));
        };

        meta.location.ok_or(NotSpawnedError::ValidButNotSpawned(
            ValidEntityButNotSpawnedError {
                entity,
                location: meta.spawned_or_despawned.by,
            },
        ))
    }

    #[inline]
    pub fn get_location(
        &self,
        entity: Entity,
    ) -> Result<Option<EntityLocation>, InvalidEntityError> {
        let meta = self.meta.get(entity.index()).unwrap_or(&EntityMeta::FRESH);

        if entity.generation() != meta.generation {
            return Err(InvalidEntityError {
                entity,
                current_generation: meta.generation,
            });
        };

        Ok(meta.location)
    }

    #[inline]
    pub fn get_by_id(&self, id: EntityId) -> Entity {
        self.meta
            .get(id.index())
            .map(|meta| Entity::new(id, meta.generation))
            .unwrap_or(Entity::from_id(id))
    }

    #[inline]
    pub fn contains(&self, entity: Entity) -> bool {
        self.meta
            .get(entity.index())
            .is_some_and(|meta| entity.generation() == meta.generation)
    }

    #[inline]
    pub fn contains_spawned_by_id(&self, id: EntityId) -> bool {
        self.meta
            .get(id.index())
            .is_some_and(|meta| meta.location.is_some())
    }

    #[inline]
    pub fn contains_spawned(&self, entity: Entity) -> bool {
        self.meta
            .get(entity.index())
            .is_some_and(|meta| entity.generation() == meta.generation && meta.location.is_some())
    }

    #[inline]
    pub fn check_spawnable(&self, entity: Entity) -> Result<(), SpawnError> {
        match self.get_location(entity) {
            Ok(None) => Ok(()),
            Ok(Some(_)) => Err(SpawnError::AlreadySpawned),
            Err(err) => Err(SpawnError::Invalid(err)),
        }
    }

    #[inline(always)]
    fn ensure_id_is_valid(&mut self, id: EntityId) {
        #[cold]
        #[inline(never)]
        fn expand(meta: &mut Vec<EntityMeta>, len: usize) {
            meta.resize(len, EntityMeta::FRESH);
            meta.resize(meta.capacity(), EntityMeta::FRESH);
        }

        let index = id.index();
        if self.meta.len() <= index {
            expand(&mut self.meta, index + 1);
        }
    }

    #[inline(always)]
    unsafe fn set_location_unchecked(
        &mut self,
        id: EntityId,
        location: Option<EntityLocation>,
    ) -> Option<EntityLocation> {
        let meta = unsafe { self.meta.get_unchecked_mut(id.index()) };
        core::mem::replace(&mut meta.location, location)
    }

    #[inline]
    pub fn set_location(
        &mut self,
        id: EntityId,
        location: Option<EntityLocation>,
    ) -> Option<EntityLocation> {
        self.ensure_id_is_valid(id);

        unsafe { self.set_location_unchecked(id, location) }
    }

    /// 更新（增加）一个 Entity 的 generation，并返回其结果。
    ///
    /// 更新 generation，表示旧的 Entity 已经失效，调用前应当预先清理资源。
    /// 返回的新 Entity 应当立刻通过 `EntityAllocater::free` 回收。
    ///
    /// # Safety
    ///
    /// - 无论是否具备数据，此函数调用后应当通过 `EntityAllocater::free` 回收实体编号。
    /// - 如果实体具备数据，应当在此函数调用前先销毁表和稀疏集中的组件资源。
    pub unsafe fn make_free(&mut self, id: EntityId, generation: u32) -> Entity {
        self.ensure_id_is_valid(id);

        let meta = unsafe { self.meta.get_unchecked_mut(id.index()) };

        let (new_generation, aliased) = meta.generation.after_check_alias(generation);

        meta.generation = new_generation;

        if aliased {
            log::warn!(
                "EntityIndex({id}) generation wrapped on Entities::free, aliasing may occur",
            );
        }

        Entity::new(id, meta.generation)
    }

    #[inline]
    pub fn set_spawned_or_despawned(&mut self, id: EntityId, by: DebugLocation, tick: Tick) {
        // SAFETY: Caller guarantees that `index` already had a location, so `declare` must have made the index valid already.
        let meta = unsafe { self.meta.get_unchecked_mut(id.index()) };
        meta.spawned_or_despawned = SpawnedOrDespawned { by, tick };
    }

    #[inline]
    fn get_spawned_or_despawned(&self, entity: Entity) -> Option<SpawnedOrDespawned> {
        self.meta
            .get(entity.index())
            .filter(|meta| {
                meta.generation == entity.generation() || {
                    meta.location.is_none() && meta.generation == entity.generation().after(1)
                }
            })
            .map(|meta| meta.spawned_or_despawned)
    }

    pub fn get_spawned_or_despawned_by(
        &self,
        entity: Entity,
    ) -> DebugLocation<Option<&'static Location<'static>>> {
        cfg::debug! {
            if {
                DebugLocation::untranspose({
                    self.get_spawned_or_despawned(entity)
                    .map(|spawned_or_despawned| spawned_or_despawned.by)
                })
            } else {
                DebugLocation::new(None)
            }
        }
    }

    pub fn get_spawn_or_despawn_tick(&self, entity: Entity) -> Option<Tick> {
        self.get_spawned_or_despawned(entity)
            .map(|spawned_or_despawned| spawned_or_despawned.tick)
    }

    pub unsafe fn get_spawned_or_despawned_unchecked(
        &self,
        entity: Entity,
    ) -> (DebugLocation, Tick) {
        let meta = unsafe { self.meta.get_unchecked(entity.index()) };
        (meta.spawned_or_despawned.by, meta.spawned_or_despawned.tick)
    }

    pub fn any_spawned(&self) -> bool {
        self.meta.iter().any(|meta| meta.location.is_some())
    }

    pub fn count_spawned(&self) -> usize {
        self.meta
            .iter()
            .filter(|meta| meta.location.is_some())
            .count()
    }
}

#![allow(clippy::new_without_default, reason = "internal function")]

use core::fmt::Debug;

use alloc::collections::BTreeSet;
use alloc::vec::Vec;

use super::EntityGeneration;

use crate::archetype::ArchetypeId;
use crate::entity::{Entity, EntityId};
use crate::storage::{TableId, TableRow};
use crate::utils::DebugCheckedUnwrap;

/// A location of an entity in an archetype.
#[derive(Debug, Copy, Clone)]
pub struct EntityLocation {
    pub archetype_id: ArchetypeId,
    pub table_id: TableId,
    pub table_row: TableRow,
}

/// Info does not need to include an Id,
/// as its index in `Vec` represents the EntityId.
pub struct EntityInfo {
    // pub id: EntityId,
    pub generation: EntityGeneration,
    pub location: EntityLocation,
}

impl Debug for EntityInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("EntityInfo")
            .field(&self.generation)
            .field(&self.location.archetype_id)
            .field(&self.location.table_id)
            .field(&self.location.table_row)
            .finish()
    }
}

pub struct Entities {
    pub(crate) infos: Vec<Option<EntityInfo>>,
    pub(crate) archetype_map: Vec<BTreeSet<EntityId>>,
}

impl Debug for Entities {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_map()
            .entries(self.archetype_map.iter().enumerate())
            .finish()
    }
}

impl Entities {
    pub(crate) const fn new() -> Self {
        Self {
            infos: Vec::new(),
            archetype_map: Vec::new(),
        }
    }

    #[inline]
    pub unsafe fn get(&self, id: EntityId) -> &EntityInfo {
        unsafe {
            self.infos
                .get_unchecked(id.index())
                .as_ref()
                .debug_checked_unwrap()
        }
    }

    #[inline]
    pub unsafe fn get_mut(&mut self, id: EntityId) -> &mut EntityInfo {
        unsafe {
            self.infos
                .get_unchecked_mut(id.index())
                .as_mut()
                .debug_checked_unwrap()
        }
    }

    #[inline]
    pub fn try_get(&self, id: EntityId) -> Option<&EntityInfo> {
        self.infos.get(id.index()).and_then(Option::as_ref)
    }

    pub(crate) fn set_despawned(&mut self, entity: Entity) -> EntityLocation {
        unsafe {
            let info_mut = self.infos.get_unchecked_mut(entity.index());

            let info = Option::take(info_mut).expect("already despawned");
            assert_eq!(info.generation, entity.generation(), "mismathed generation");

            let archetype_index = info.location.archetype_id.index();
            self.archetype_map
                .get_unchecked_mut(archetype_index)
                .remove(&entity.id());

            info.location
        }
    }

    pub(crate) fn set_spawned(&mut self, entity: Entity, location: EntityLocation) {
        #[cold]
        #[inline(never)]
        fn resize_infos(this: &mut Entities, len: usize) {
            this.infos.reserve(len - this.infos.len());
            this.infos.resize_with(this.infos.capacity(), || None);
        }

        #[cold]
        #[inline(never)]
        fn resize_archetype_map(this: &mut Entities, len: usize) {
            this.archetype_map.reserve(len - this.archetype_map.len());
            this.archetype_map
                .resize_with(this.archetype_map.capacity(), BTreeSet::new);
        }

        let entity_index = entity.index();
        let archetype_index = location.archetype_id.index();

        if self.infos.len() <= entity_index {
            resize_infos(self, entity_index + 1);
        }

        if self.archetype_map.len() <= archetype_index {
            resize_archetype_map(self, archetype_index + 1);
        }

        unsafe {
            *self.infos.get_unchecked_mut(entity_index) = Some(EntityInfo {
                generation: entity.generation(),
                location,
            });
            self.archetype_map
                .get_unchecked_mut(archetype_index)
                .insert(entity.id());
        }
    }
}

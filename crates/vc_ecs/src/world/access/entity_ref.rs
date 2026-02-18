use core::any::TypeId;
use core::fmt::Debug;

use crate::archetype::Archetype;
use crate::borrow::{Ref, UntypedRef};
use crate::component::{Component, ComponentId};
use crate::entity::{Entity, EntityLocation};
use crate::storage::StorageType;
use crate::tick::Tick;
use crate::world::{EntityMut, World};

pub struct EntityRef<'w> {
    pub(crate) world: &'w World,
    pub(crate) entity: Entity,
    pub(crate) location: EntityLocation,
    pub(crate) now_tick: Tick,
}

impl<'w> From<EntityMut<'w>> for EntityRef<'w> {
    fn from(value: EntityMut<'w>) -> Self {
        EntityRef {
            world: value.world,
            entity: value.entity,
            location: value.location,
            now_tick: value.now_tick,
        }
    }
}

impl Debug for EntityRef<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("EntityRef")
            .field("world", &self.world.id)
            .field("entity", &self.entity)
            .finish()
    }
}

impl<'w> EntityRef<'w> {
    pub fn entity(&self) -> Entity {
        self.entity
    }

    #[inline]
    pub fn archetype(&self) -> &Archetype {
        unsafe { self.world.archetypes.get(self.location.archetype_id) }
    }

    #[inline]
    pub fn contains<T: Component>(&self) -> bool {
        let Some(id) = self.world.components.get_component_id(TypeId::of::<T>()) else {
            return false;
        };
        self.contains_id(id)
    }

    #[inline]
    pub fn contains_id(&self, id: ComponentId) -> bool {
        self.archetype().contains_component(id)
    }

    #[inline]
    pub fn contains_type_id(&self, type_id: TypeId) -> bool {
        let Some(id) = self.world.components.get_component_id(type_id) else {
            return false;
        };
        self.contains_id(id)
    }

    #[inline]
    pub fn get_ref<T: Component>(&self) -> Option<Ref<'_, T>> {
        let id = self.world.components.get_component_id(TypeId::of::<T>())?;
        match T::STORAGE_TYPE {
            StorageType::Table => unsafe { Some(self.get_ref_in_table(id).with_type::<T>()) },
            StorageType::SparseSet => unsafe { Some(self.get_ref_in_sparse(id).with_type::<T>()) },
        }
    }

    #[inline]
    pub fn get_untyped_ref(&self, id: ComponentId) -> Option<UntypedRef<'_>> {
        let info = self.world.components.try_get(id)?;

        match info.storage_type() {
            StorageType::Table => unsafe { Some(self.get_ref_in_table(id)) },
            StorageType::SparseSet => unsafe { Some(self.get_ref_in_sparse(id)) },
        }
    }

    unsafe fn get_ref_in_table(&self, id: ComponentId) -> UntypedRef<'_> {
        unsafe {
            let table = self.world.storages.tables.get(self.location.table_id);
            let storag_index = table.get_index(id);
            let table_row = self.location.table_row;
            let last_run = self.world.last_change;
            let this_run = self.now_tick;
            table.get_ref(storag_index, table_row, last_run, this_run)
        }
    }

    unsafe fn get_ref_in_sparse(&self, id: ComponentId) -> UntypedRef<'_> {
        unsafe {
            let storag_index = self.world.storages.sparse_sets.get_index(id);
            let sparse_set = self.world.storages.sparse_sets.get(storag_index);
            let entity_id = self.entity.id();
            let last_run = self.world.last_change;
            let this_run = self.now_tick;
            sparse_set.get_ref(entity_id, last_run, this_run)
        }
    }
}

use core::fmt::Debug;
use core::ptr::NonNull;

use crate::entity::{Entity, EntityLocation};
use crate::tick::Tick;
use crate::world::{FetchComponent, World};

pub struct EntityMut<'a> {
    pub(crate) world: &'a mut World,
    pub(crate) entity: Entity,
    pub(crate) location: EntityLocation,
    pub(crate) last_run: Tick,
    pub(crate) this_run: Tick,
}

pub struct EntityRef<'a> {
    pub(crate) world: &'a World,
    pub(crate) entity: Entity,
    pub(crate) location: EntityLocation,
    pub(crate) last_run: Tick,
    pub(crate) this_run: Tick,
}

impl<'a> From<EntityMut<'a>> for EntityRef<'a> {
    fn from(value: EntityMut<'a>) -> Self {
        EntityRef {
            world: value.world,
            entity: value.entity,
            location: value.location,
            last_run: value.last_run,
            this_run: value.this_run,
        }
    }
}

impl Debug for EntityMut<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("EntityMut")
            .field("entity", &self.entity)
            .field("location", &self.location)
            .finish()
    }
}

impl Debug for EntityRef<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("EntityRef")
            .field("entity", &self.entity)
            .field("location", &self.location)
            .finish()
    }
}

impl<'a> EntityMut<'a> {
    pub fn entity(&self) -> Entity {
        self.entity
    }

    pub fn get<T: FetchComponent>(&self) -> Option<T::Raw<'_>> {
        unsafe {
            T::get(
                NonNull::from_ref(self.world),
                self.entity,
                self.location.table_id,
                self.location.table_row,
            )
        }
    }

    pub fn get_ref<T: FetchComponent>(&self) -> Option<T::Ref<'_>> {
        unsafe {
            T::get_ref(
                NonNull::from_ref(self.world),
                self.entity,
                self.location.table_id,
                self.location.table_row,
                self.last_run,
                self.this_run,
            )
        }
    }

    pub fn get_mut<T: FetchComponent>(&mut self) -> Option<T::Mut<'_>> {
        unsafe {
            T::get_mut(
                NonNull::from_mut(self.world),
                self.entity,
                self.location.table_id,
                self.location.table_row,
                self.last_run,
                self.this_run,
            )
        }
    }
}

impl<'a> EntityRef<'a> {
    pub fn entity(&self) -> Entity {
        self.entity
    }

    pub fn get<T: FetchComponent>(&self) -> Option<T::Raw<'_>> {
        unsafe {
            T::get(
                NonNull::from_ref(self.world),
                self.entity,
                self.location.table_id,
                self.location.table_row,
            )
        }
    }

    pub fn get_ref<T: FetchComponent>(&self) -> Option<T::Ref<'_>> {
        unsafe {
            T::get_ref(
                NonNull::from_ref(self.world),
                self.entity,
                self.location.table_id,
                self.location.table_row,
                self.last_run,
                self.this_run,
            )
        }
    }
}

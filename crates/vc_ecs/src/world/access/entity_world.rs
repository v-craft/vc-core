use core::ptr::NonNull;

use crate::entity::{Entity, EntityError, EntityLocation};
use crate::tick::Tick;
use crate::world::{EntityMut, EntityRef, FetchComponent, World};

pub struct WorldEntityMut<'a> {
    pub(crate) world: &'a mut World,
    pub(crate) entity: Entity,
    pub(crate) location: EntityLocation,
}

impl<'a> From<WorldEntityMut<'a>> for EntityMut<'a> {
    fn from(value: WorldEntityMut<'a>) -> Self {
        let this_run = Tick::new(*value.world.this_run.get_mut());
        let last_run = value.world.last_run;
        EntityMut {
            world: value.world,
            entity: value.entity,
            location: value.location,
            last_run,
            this_run,
        }
    }
}

impl<'a> From<WorldEntityMut<'a>> for EntityRef<'a> {
    fn from(value: WorldEntityMut<'a>) -> Self {
        let this_run = Tick::new(*value.world.this_run.get_mut());
        let last_run = value.world.last_run;
        EntityRef {
            world: value.world,
            entity: value.entity,
            location: value.location,
            last_run,
            this_run,
        }
    }
}

impl<'a> WorldEntityMut<'a> {
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
        let world = self.world as *const World as *mut World;
        let this_run = Tick::new(*(unsafe { &mut *world }).this_run.get_mut());
        let last_run = self.world.last_run;
        unsafe {
            T::get_ref(
                NonNull::from_ref(self.world),
                self.entity,
                self.location.table_id,
                self.location.table_row,
                last_run,
                this_run,
            )
        }
    }

    pub fn get_mut<T: FetchComponent>(&mut self) -> Option<T::Mut<'_>> {
        let this_run = Tick::new(*self.world.this_run.get_mut());
        let last_run = self.world.last_run;
        unsafe {
            T::get_mut(
                NonNull::from_mut(self.world),
                self.entity,
                self.location.table_id,
                self.location.table_row,
                last_run,
                this_run,
            )
        }
    }

    pub fn despawn(self) -> Result<(), EntityError> {
        self.world.despawn(self.entity)
    }
}

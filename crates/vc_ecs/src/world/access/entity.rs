use core::fmt::Debug;

use crate::entity::{Entity, EntityError, EntityLocation};
use crate::tick::Tick;
use crate::world::{GetComponent, UnsafeWorld, World};

pub struct EntityOwned<'a> {
    pub(crate) world: UnsafeWorld<'a>,
    pub(crate) entity: Entity,
    pub(crate) location: EntityLocation,
    // We have World's exclusive borrowing.
    // pub(crate) last_run: Tick,
    // pub(crate) this_run: Tick,
}

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

impl<'a> From<EntityOwned<'a>> for EntityMut<'a> {
    fn from(value: EntityOwned<'a>) -> Self {
        EntityMut {
            last_run: value.last_run(),
            this_run: value.this_run(),
            world: unsafe { value.world.data_mut() },
            entity: value.entity,
            location: value.location,
        }
    }
}

impl<'a> From<EntityOwned<'a>> for EntityRef<'a> {
    fn from(value: EntityOwned<'a>) -> Self {
        EntityRef {
            last_run: value.last_run(),
            this_run: value.this_run(),
            world: unsafe { value.world.read_only() },
            entity: value.entity,
            location: value.location,
        }
    }
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

macro_rules! impl_debug {
    ($name:ident) => {
        impl Debug for $name<'_> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct(stringify!($name))
                    .field("entity", &self.entity)
                    .field("location", &self.location)
                    .finish()
            }
        }
    };
}

impl_debug!(EntityOwned);
impl_debug!(EntityMut);
impl_debug!(EntityRef);

impl<'a> EntityMut<'a> {
    pub fn entity(&self) -> Entity {
        self.entity
    }

    pub fn get<T: GetComponent>(&self) -> Option<T::Raw<'_>> {
        unsafe {
            T::get(
                self.world.unsafe_world(),
                self.entity,
                self.location.table_id,
                self.location.table_row,
            )
        }
    }

    pub fn get_ref<T: GetComponent>(&self) -> Option<T::Ref<'_>> {
        unsafe {
            T::get_ref(
                self.world.unsafe_world(),
                self.entity,
                self.location.table_id,
                self.location.table_row,
                self.last_run,
                self.this_run,
            )
        }
    }

    pub fn get_mut<T: GetComponent>(&mut self) -> Option<T::Mut<'_>> {
        unsafe {
            T::get_mut(
                self.world.unsafe_world(),
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

    pub fn get<T: GetComponent>(&self) -> Option<T::Raw<'_>> {
        unsafe {
            T::get(
                self.world.unsafe_world(),
                self.entity,
                self.location.table_id,
                self.location.table_row,
            )
        }
    }

    pub fn get_ref<T: GetComponent>(&self) -> Option<T::Ref<'_>> {
        unsafe {
            T::get_ref(
                self.world.unsafe_world(),
                self.entity,
                self.location.table_id,
                self.location.table_row,
                self.last_run,
                self.this_run,
            )
        }
    }
}

impl<'a> EntityOwned<'a> {
    #[inline(always)]
    fn this_run(&self) -> Tick {
        let world = unsafe { self.world.data_mut() };
        Tick::new(*world.this_run.get_mut())
    }

    #[inline(always)]
    fn last_run(&self) -> Tick {
        let world = unsafe { self.world.data_mut() };
        Tick::new(*world.this_run.get_mut())
    }

    pub fn entity(&self) -> Entity {
        self.entity
    }

    pub fn get<T: GetComponent>(&self) -> Option<T::Raw<'_>> {
        unsafe {
            T::get(
                self.world,
                self.entity,
                self.location.table_id,
                self.location.table_row,
            )
        }
    }

    pub fn get_ref<T: GetComponent>(&self) -> Option<T::Ref<'_>> {
        unsafe {
            T::get_ref(
                self.world,
                self.entity,
                self.location.table_id,
                self.location.table_row,
                self.last_run(),
                self.this_run(),
            )
        }
    }

    pub fn get_mut<T: GetComponent>(&mut self) -> Option<T::Mut<'_>> {
        unsafe {
            T::get_mut(
                self.world,
                self.entity,
                self.location.table_id,
                self.location.table_row,
                self.last_run(),
                self.this_run(),
            )
        }
    }

    pub fn despawn(self) -> Result<(), EntityError> {
        let world = unsafe { self.world.full_mut() };
        world.despawn(self.entity)
    }
}

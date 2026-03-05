use core::fmt::Debug;

use crate::system::{AccessTable, SystemParam};
use crate::tick::Tick;
use crate::utils::DebugName;
use crate::world::{UnsafeWorld, World, WorldId};

use bitflags::bitflags;

bitflags! {
    /// Bitflags representing system states and requirements.
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct SystemFlags: u8 {
        /// Set if system cannot be sent across threads
        const NON_SEND = 1 << 0;
        /// Set if system requires exclusive World access
        const EXCLUSIVE = 1 << 1;
    }
}

#[derive(Clone, Copy)]
pub struct SystemMeta {
    flags: SystemFlags,
    last_run: Tick,
    name: DebugName,
}

impl Debug for SystemMeta {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SystemMeta")
            .field("name", &self.name)
            .field("last_run", &self.last_run)
            .field("non_send", &self.is_non_send())
            .field("exclusive", &self.is_exclusive())
            .finish()
    }
}

impl SystemMeta {
    pub(crate) fn new<T>() -> Self {
        Self {
            name: DebugName::type_name::<T>(),
            flags: SystemFlags::empty(),
            last_run: Tick::new(0),
        }
    }

    #[inline]
    pub fn flags(&self) -> SystemFlags {
        self.flags
    }

    #[inline]
    pub fn name(&self) -> DebugName {
        self.name
    }

    #[inline]
    pub fn set_name(&mut self, name: DebugName) {
        self.name = name;
    }

    #[inline]
    pub fn last_run(&self) -> Tick {
        self.last_run
    }

    #[inline]
    pub fn set_last_run(&mut self, last_run: Tick) {
        self.last_run = last_run;
    }

    #[inline]
    pub fn is_non_send(&self) -> bool {
        self.flags.intersects(SystemFlags::NON_SEND)
    }

    #[inline]
    pub fn is_exclusive(&self) -> bool {
        self.flags.intersects(SystemFlags::EXCLUSIVE)
    }

    #[inline]
    pub fn set_non_send(&mut self) {
        self.flags |= SystemFlags::NON_SEND;
    }

    #[inline]
    pub fn set_exclusive(&mut self) {
        self.flags |= SystemFlags::EXCLUSIVE;
    }
}

pub struct SystemState<Param: SystemParam + 'static> {
    world_id: WorldId,
    meta: SystemMeta,
    state: Param::State,
}

impl<T: SystemParam + 'static>  Debug for SystemState<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SystemState")
            .field("world_id", &self.world_id)
            .field("name", &self.meta.name)
            .field("last_run", &self.meta.last_run)
            .field("non_send", &self.meta.is_non_send())
            .field("exclusive", &self.meta.is_exclusive())
            .finish()
    }
}

impl<Param: SystemParam> SystemState<Param> {
    pub fn new(world: &mut World) -> Self {
        let mut meta = SystemMeta::new::<Param>();
        meta.last_run = world.last_run().relative_to(Tick::MAX_AGE);
        if Param::NON_SEND {
            meta.set_non_send();
        }
        if Param::EXCLUSIVE {
            meta.set_exclusive();
        }

        let state = unsafe { Param::init_state(world) };
        
        unsafe {
            // We need to call `mark_access` to ensure there are no panics
            // from conflicts within `Param`, even though we don't use the calculated access.
            let mut access_table = AccessTable::new();
            assert!{
                Param::mark_access(&mut access_table, &state),
                "invalid system params: {}",
                DebugName::type_name::<Param>(),
            }
        }

        Self {
            meta,
            state,
            world_id: world.id(),
        }
    }

    #[inline]
    pub fn world_id(&self) -> WorldId {
        self.world_id
    }

    #[inline]
    pub fn meta(&self) -> &SystemMeta {
        &self.meta
    }

    #[inline]
    pub fn meta_mut(&mut self) -> &mut SystemMeta {
        &mut self.meta
    }
    
    #[inline]
    pub fn state(&self) -> &Param::State {
        &self.state
    }

    #[inline]
    pub fn state_mut(&mut self) -> &mut Param::State {
        &mut self.state
    }

    pub unsafe fn fetch<'w, 's>(&'s mut self, world: UnsafeWorld<'w>) -> Param::Item<'w, 's> {
        unsafe {
            self.validate_world(world);
            let this_run = world.read_only().advance_tick();
            let last_run = self.meta.last_run;
            let state = &mut self.state;
            Param::get_param(world, state, last_run, this_run)
        }
    }

    pub unsafe fn fetch_with<'w, 's>(
        &'s mut self,
        world: UnsafeWorld<'w>,
        this_run: Tick,
    ) -> Param::Item<'w, 's> {
        unsafe {
            self.validate_world(world);
            let last_run = self.meta.last_run;
            let state = &mut self.state;
            Param::get_param(world, state, last_run, this_run)
        }
    }

    #[inline(always)]
    fn validate_world(&self, world: UnsafeWorld) {
        if ::core::cfg!(debug_assertions) {
            let world_id = unsafe { world.read_only().id() };
            if self.world_id != world_id {
                mismatched_world(self.meta.name, self.world_id, world_id);
            }
        }
    }
}

#[cold]
#[inline(never)]
fn mismatched_world(name: DebugName, state: WorldId, input: WorldId) -> ! {
    panic!("System<{name}> was registered in World<{state}>, but is used in World<{input}>.")
}

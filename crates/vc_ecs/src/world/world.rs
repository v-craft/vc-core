use core::fmt;

use vc_os::sync::atomic::AtomicU32;

use super::WorldId;
use crate::archetype::Archetypes;
use crate::component::{ComponentIdAllocator, Components};
use crate::entity::{Entities, EntityAllocator};
use crate::storage::Storages;
use crate::tick::Tick;

pub struct World {
    pub(crate) id: WorldId,
    pub(crate) archetypes: Archetypes,
    pub(crate) storages: Storages,
    pub(crate) entities: Entities,
    pub(crate) allocator: EntityAllocator,
    pub(crate) components: Components,
    pub(crate) component_allocator: ComponentIdAllocator,
    pub(crate) change_tick: AtomicU32,
    pub(crate) last_check_tick: Tick,
    pub(crate) last_change_tick: Tick,
    // TODO
}

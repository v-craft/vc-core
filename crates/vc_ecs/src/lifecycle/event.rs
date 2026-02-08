use vc_reflect::derive::Reflect;

use crate::component::ComponentId;
use crate::entity::Entity;
use crate::event::EventKey;

// -----------------------------------------------------------------------------
// EventKeys

pub const ADD: EventKey = EventKey(ComponentId::from_u32(1));
pub const INSERT: EventKey = EventKey(ComponentId::from_u32(2));
pub const REPLACE: EventKey = EventKey(ComponentId::from_u32(3));
pub const REMOVE: EventKey = EventKey(ComponentId::from_u32(4));
pub const DESPAWN: EventKey = EventKey(ComponentId::from_u32(5));

// -----------------------------------------------------------------------------
// Event - Add

#[derive(Reflect, Debug, Clone)]
#[reflect(clone, debug, auto_register)]
pub struct Add {
    pub entity: Entity,
}

// -----------------------------------------------------------------------------
// Event - Insert

#[derive(Reflect, Debug, Clone)]
#[reflect(clone, debug, auto_register)]
pub struct Insert {
    pub entity: Entity,
}

// -----------------------------------------------------------------------------
// Event - Insert

#[derive(Reflect, Debug, Clone)]
#[reflect(clone, debug, auto_register)]
pub struct Replace {
    pub entity: Entity,
}

// -----------------------------------------------------------------------------
// Event - Remove

#[derive(Reflect, Debug, Clone)]
#[reflect(clone, debug, auto_register)]
pub struct Remove {
    pub entity: Entity,
}

// -----------------------------------------------------------------------------
// Event - Despawn

#[derive(Reflect, Debug, Clone)]
#[reflect(clone, debug, auto_register)]
pub struct Despawn {
    pub entity: Entity,
}

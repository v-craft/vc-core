use super::Trigger;
use crate::entity::Entity;
use crate::component::ComponentId;




pub trait Event: Send + Sync + Sized + 'static {
    type Trigger<'a>: Trigger<Self>;
}


pub trait EntityEvent: Event {
    fn event_target(&self) -> Entity;
}

pub trait SetEntityEventTarget: EntityEvent {
    fn set_event_target(&mut self, entity: Entity);
}



use core::marker::PhantomData;

use super::Trigger;
use crate::component::ComponentId;
use crate::entity::Entity;

pub trait Event: Send + Sync + Sized + 'static {
    type Trigger<'a>: Trigger<Self>;
}

pub trait EntityEvent: Event {
    fn event_target(&self) -> Entity;
}

pub trait SetEntityEventTarget: EntityEvent {
    fn set_event_target(&mut self, entity: Entity);
}

struct EventWrapperComponent<E: Event>(PhantomData<E>);

#[derive(Debug, Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct EventKey(ComponentId);

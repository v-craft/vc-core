use core::marker::PhantomData;

use super::{EventKey, Event, CachedObservers};
use crate::component::ComponentId;
use crate::event::EntityEvent;
use crate::utils::DebugLocation;
use crate::world::DeferredWorld;
use crate::bundle::Bundle;
use crate::entity::Entity;

pub struct TriggerContext {
    /// The [`EventKey`] the trigger targeted.
    pub event_key: EventKey,
    /// The location of the source code that triggered the observer.
    pub caller: DebugLocation,
}

pub unsafe trait Trigger<E: Event> {
    unsafe fn trigger(
        &mut self,
        world: DeferredWorld,
        observers: &CachedObservers,
        trigger_context: &TriggerContext,
        event: &mut E,
    );
}

#[derive(Default, Debug)]
pub struct GlobalTrigger;

#[derive(Default, Debug)]
pub struct EntityTrigger;

pub struct PropagateEntityTrigger<const AUTO_PROPAGATE: bool, E: EntityEvent, T> {
    /// The original [`Entity`] the [`Event`] was _first_ triggered for.
    pub original_event_target: Entity,

    /// Whether or not to continue propagating using the `T` [`Traversal`]. If this is false,
    /// The [`Traversal`] will stop on the current entity.
    pub propagate: bool,

    _marker: PhantomData<(E, T)>,
}

#[derive(Default)]
pub struct EntityComponentsTrigger<'a> {
    /// All of the components whose observers were triggered together for the target entity. For example,
    /// if components `A` and `B` are added together, producing the [`Add`](crate::lifecycle::Add) event, this will
    /// contain the [`ComponentId`] for both `A` and `B`.
    pub components: &'a [ComponentId],
}



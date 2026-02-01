use vc_ptr::PtrMut;
use vc_utils::hash::SparseHashMap;

use super::{TriggerContext, EventKey};
use crate::world::DeferredWorld;
use crate::component::ComponentId;
use crate::entity::{EntityHashMap, Entity};

pub type ObserverRunner =
    unsafe fn(DeferredWorld, observer: Entity, &TriggerContext, event: PtrMut, trigger: PtrMut);


pub type ObserverMap = EntityHashMap<ObserverRunner>;

#[derive(Default, Debug)]
pub struct CachedComponentObservers {
    // Observers watching for events targeting this component, but not a specific entity
    pub(super) global_observers: ObserverMap,
    // Observers watching for events targeting this component on a specific entity
    pub(super) entity_component_observers: EntityHashMap<ObserverMap>,
}

#[derive(Default, Debug)]
pub struct CachedObservers {
    /// Observers watching for any time this event is triggered, regardless of target.
    /// These will also respond to events targeting specific components or entities
    pub(super) global_observers: ObserverMap,
    /// Observers watching for triggers of events for a specific component
    pub(super) component_observers: SparseHashMap<ComponentId, CachedComponentObservers>,
    /// Observers watching for triggers of events for a specific entity
    pub(super) entity_observers: EntityHashMap<ObserverMap>,
}

#[derive(Default, Debug)]
pub struct Observers {
    // Cached ECS observers to save a lookup for high-traffic built-in event types.
    add: CachedObservers,
    insert: CachedObservers,
    replace: CachedObservers,
    remove: CachedObservers,
    despawn: CachedObservers,
    // Map from event type to set of observers watching for that event
    cache: SparseHashMap<EventKey, CachedObservers>,
}

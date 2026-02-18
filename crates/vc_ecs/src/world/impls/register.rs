use crate::component::{Component, ComponentId, NonSendResource, Resource};
use crate::storage::StorageType;
use crate::world::World;

impl World {
    pub fn register_resouce<T: Resource>(&mut self) -> ComponentId {
        let id = self
            .components
            .register_resource::<T>(&mut self.compid_allocator);
        let info = unsafe { self.components.get(id) };
        self.storages.resources.prepare(info);
        id
    }

    pub fn register_non_send<T: NonSendResource>(&mut self) -> ComponentId {
        let id = self
            .components
            .register_non_send::<T>(&mut self.compid_allocator);
        let info = unsafe { self.components.get(id) };
        self.storages.non_sends.prepare(info);
        id
    }

    pub fn register_component<T: Component>(&mut self) -> ComponentId {
        let id = self
            .components
            .register_component::<T>(&mut self.compid_allocator);
        if T::STORAGE_TYPE == StorageType::SparseSet {
            let info = unsafe { self.components.get(id) };
            self.storages.sparse_sets.prepare(info);
        }
        id
    }
}

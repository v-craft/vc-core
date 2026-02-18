use core::any::TypeId;

use super::{CompIdAllocator, ComponentId, Components};
use super::{Component, NonSendResource, Resource};
use super::{ComponentDescriptor, ComponentInfo};

impl Components {
    #[inline]
    pub(crate) fn register_component<T: Component>(
        &mut self,
        allocator: &mut CompIdAllocator,
    ) -> ComponentId {
        #[cold]
        #[inline(never)]
        fn register_component_internal(
            this: &mut Components,
            id: ComponentId,
            descriptor: ComponentDescriptor,
        ) {
            let type_id = descriptor.type_id;
            this.register_dynamic(id, descriptor);
            this.components.insert(type_id, id);
        }

        if let Some(id) = self.get_component_id(TypeId::of::<T>()) {
            return id;
        }
        let id = allocator.alloc_mut();
        let descriptor = ComponentDescriptor::new_component::<T>();
        register_component_internal(self, id, descriptor);
        id
    }

    #[inline]
    pub(crate) fn register_resource<T: Resource>(
        &mut self,
        allocator: &mut CompIdAllocator,
    ) -> ComponentId {
        #[cold]
        #[inline(never)]
        fn register_resource_internal(
            this: &mut Components,
            id: ComponentId,
            descriptor: ComponentDescriptor,
        ) {
            let type_id = descriptor.type_id;
            this.register_dynamic(id, descriptor);
            this.resources.insert(type_id, id);
        }

        if let Some(id) = self.get_resource_id(TypeId::of::<T>()) {
            return id;
        }
        let id = allocator.alloc_mut();
        let descriptor = ComponentDescriptor::new_resource::<T>();
        register_resource_internal(self, id, descriptor);
        id
    }

    #[inline]
    pub(crate) fn register_non_send<T: NonSendResource>(
        &mut self,
        allocator: &mut CompIdAllocator,
    ) -> ComponentId {
        #[cold]
        #[inline(never)]
        fn register_non_send_internal(
            this: &mut Components,
            id: ComponentId,
            descriptor: ComponentDescriptor,
        ) {
            let type_id = descriptor.type_id;
            this.register_dynamic(id, descriptor);
            this.non_sends.insert(type_id, id);
        }

        if let Some(id) = self.get_non_send_id(TypeId::of::<T>()) {
            return id;
        }
        let id = allocator.alloc_mut();
        let descriptor = ComponentDescriptor::new_non_send::<T>();
        register_non_send_internal(self, id, descriptor);
        id
    }

    #[inline(always)]
    fn register_dynamic(&mut self, id: ComponentId, descriptor: ComponentDescriptor) {
        #[cold]
        #[inline(never)]
        fn resize_infos(this: &mut Components, len: usize) {
            this.infos.resize_with(len, || None);
            // we fill Vec to reduce the resize function calls,
            // so you cannot infer the number of components based on len.
            this.infos.resize_with(this.infos.capacity(), || None);
        }

        let index = id.index();
        if index >= self.infos.len() {
            resize_infos(self, index + 1);
        }

        let info = ComponentInfo::new(id, descriptor);
        // SAFETY: We just extended the vec to make this index valid.
        unsafe {
            *self.infos.get_unchecked_mut(index) = Some(info);
        }
    }
}

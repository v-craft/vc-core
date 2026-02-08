use core::any::TypeId;

use super::{CompIdAllocator, ComponentDescriptor, Components};
use super::{Component, ComponentId, NoSendResource, Resource};

use crate::component::ComponentInfo;

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
        let id = allocator.next_mut();
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
        let id = allocator.next_mut();
        let descriptor = ComponentDescriptor::new_resource::<T>();
        register_resource_internal(self, id, descriptor);
        id
    }

    #[inline]
    pub(crate) fn register_no_send<T: NoSendResource>(
        &mut self,
        allocator: &mut CompIdAllocator,
    ) -> ComponentId {
        #[cold]
        #[inline(never)]
        fn register_no_send_internal(
            this: &mut Components,
            id: ComponentId,
            descriptor: ComponentDescriptor,
        ) {
            let type_id = descriptor.type_id;
            this.register_dynamic(id, descriptor);
            this.no_sends.insert(type_id, id);
        }

        if let Some(id) = self.get_no_send_id(TypeId::of::<T>()) {
            return id;
        }
        let id = allocator.next_mut();
        let descriptor = ComponentDescriptor::new_no_send::<T>();
        register_no_send_internal(self, id, descriptor);
        id
    }

    #[inline(always)]
    fn register_dynamic(&mut self, id: ComponentId, descriptor: ComponentDescriptor) {
        #[cold]
        #[inline(never)]
        fn resize_infos(this: &mut Components, len: usize) {
            this.infos.resize_with(len, || None);
            // 强制填充以减少 resize 调用次数，但这导致你不能用 infos 判断有效组件数
            this.infos.resize_with(this.infos.capacity(), || None);
        }

        let index = id.index();

        let least_len = index + 1;
        if least_len > self.infos.len() {
            resize_infos(self, least_len);
        }

        let info = ComponentInfo::new(id, descriptor);
        // SAFETY: We just extended the vec to make this index valid.
        unsafe {
            *self.infos.get_unchecked_mut(index) = Some(info);
        }
    }
}

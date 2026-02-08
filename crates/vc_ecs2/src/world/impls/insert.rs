use vc_ptr::OwningPtr;

use super::World;
use crate::component::{ComponentId, NoSendResource, Resource};
use crate::utils::{DebugCheckedUnwrap, DebugLocation};

impl World {
    #[inline(always)]
    #[cfg_attr(any(debug_assertions, feature = "debug"), track_caller)]
    pub fn insert_resource<T: Resource>(&mut self, value: T) {
        let id = self
            .components
            .register_resource::<T>(&mut self.compid_allocator);

        vc_ptr::into_owning!(value);

        self.insert_resouce_internal(value, id, DebugLocation::caller());
    }

    #[inline(never)]
    fn insert_resouce_internal(
        &mut self,
        value: OwningPtr<'_>,
        id: ComponentId,
        caller: DebugLocation,
    ) {
        let info = unsafe { self.components.get(id) };
        self.storages.resources.prepare(info);

        unsafe {
            let data = self.storages.resources.get_mut(id).debug_checked_unwrap();
            data.init(value, self.now, caller);
        }
    }

    #[inline(always)]
    #[cfg_attr(any(debug_assertions, feature = "debug"), track_caller)]
    pub fn insert_no_send<T: NoSendResource>(&mut self, value: T) {
        let id = self
            .components
            .register_no_send::<T>(&mut self.compid_allocator);

        vc_ptr::into_owning!(value);

        self.insert_no_send_internal(value, id, DebugLocation::caller());
    }

    #[inline(never)]
    fn insert_no_send_internal(
        &mut self,
        value: OwningPtr<'_>,
        id: ComponentId,
        caller: DebugLocation,
    ) {
        let info = unsafe { self.components.get(id) };
        self.storages.no_sends.prepare(info);

        unsafe {
            let data = self.storages.no_sends.get_mut(id).debug_checked_unwrap();
            data.init(value, self.now, caller);
        }
    }
}

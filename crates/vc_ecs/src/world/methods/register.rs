use core::any::TypeId;

use crate::bundle::{Bundle, BundleId};
use crate::component::{CollectResult, Component, ComponentCollector, ComponentId};
use crate::resource::{Resource, ResourceId};
use crate::world::World;

impl World {
    #[inline]
    pub fn register_resource<T: Resource>(&mut self) -> ResourceId {
        self.resources.register::<T>()
    }

    #[inline]
    pub fn register_component<T: Component>(&mut self) -> ComponentId {
        self.components.register::<T>()
    }

    #[inline]
    pub fn prepare_resource(&mut self, id: ResourceId) {
        if let Some(info) = self.resources.get(id) {
            self.storages.prepare_resource(info);
        }
    }

    #[inline]
    pub fn prepare_component(&mut self, id: ComponentId) {
        if let Some(info) = self.components.get(id) {
            self.storages.prepare_component(info);
        }
    }

    #[inline]
    pub fn register_bundle<T: Bundle>(&mut self) -> BundleId {
        if let Some(id) = self.bundles.get_id_by_type(TypeId::of::<T>()) {
            id
        } else {
            self.register_bundle_slow(TypeId::of::<T>(), T::collect_components)
        }
    }

    #[cold]
    #[inline(never)]
    fn register_bundle_slow(
        &mut self,
        type_id: TypeId,
        register_fn: unsafe fn(&mut ComponentCollector),
    ) -> BundleId {
        let mut collector = ComponentCollector::new(&mut self.components);
        unsafe {
            register_fn(&mut collector);
        }

        let CollectResult{
            mut dense,
            mut sparse,
        } = collector.sorted();
        
        // 0 <= ComponentId < u32::MAX, so dense_len < u32::MAX.
        let dense_len = dense.len() as u32;

        dense.append(&mut sparse);
        unsafe { self.bundles.register(type_id, &dense, dense_len) }
    }
}

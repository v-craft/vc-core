use core::any::TypeId;

use crate::archetype::ArcheId;
use crate::bundle::{Bundle, BundleId};
use crate::component::{Component, ComponentCollector, ComponentId};
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
        if let Some(id) = self.bundles.get_id(TypeId::of::<T>()) {
            id
        } else {
            self.register_bundle_slow(TypeId::of::<T>(), T::collect_components)
        }
    }

    #[inline]
    pub fn register_archetype(&mut self, bundle_id: BundleId) -> ArcheId {
        if let Some(id) = self.archetypes.get_by_bundle(bundle_id) {
            id
        } else {
            self.register_archetype_slow(bundle_id)
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

        let (mut dense, mut sparse) = collector.split();
        dense.sort_unstable();
        sparse.sort_unstable();
        dense.dedup();
        sparse.dedup();
        // 0 <= ComponentId < u32::MAX, so dense_len < u32::MAX.
        let dense_len = dense.len() as u32;

        let mut buf = dense;
        buf.append(&mut sparse);

        unsafe { self.bundles.register(type_id, &buf, dense_len) }
    }

    #[cold]
    #[inline(never)]
    fn register_archetype_slow(&mut self, bundle_id: BundleId) -> ArcheId {
        let info = self.bundles.get(bundle_id).unwrap();
        if let Some(id) = self.archetypes.get_id(&info.components) {
            unsafe {
                self.archetypes.insert_bundle_id(bundle_id, id);
            }
            return id;
        }

        let dense_len = info.dense_len as usize;
        let components = info.components.clone();
        let table_id = unsafe {
            let sparses = info.sparse_components();
            self.storages.maps.register(&self.components, sparses);
            let denses = info.dense_components();
            self.storages.tables.register(&self.components, denses)
        };

        unsafe {
            let id = self.archetypes.register(table_id, dense_len, components);
            self.archetypes.insert_bundle_id(bundle_id, id);
            id
        }
    }
}

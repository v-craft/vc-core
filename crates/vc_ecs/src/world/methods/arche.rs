use core::ptr::NonNull;

use alloc::vec::Vec;

use vc_os::sync::Arc;
use vc_utils::hash::SparseHashSet;

use crate::archetype::ArcheId;
use crate::bundle::BundleId;
use crate::component::{CollectResult, ComponentCollector, ComponentId};
use crate::world::World;

impl World {
    pub fn arche_after_insert(&mut self, arche_id: ArcheId, bundle_id: BundleId) -> ArcheId {
        let arche = unsafe { self.archetypes.get_unchecked(arche_id) };
        if let Some(cached) = arche.after_insert(bundle_id) {
            return cached;
        }
        unsafe { self.after_insert_slow(arche_id, bundle_id) }
    }

    #[cold]
    #[inline(never)]
    unsafe fn after_insert_slow(&mut self, arche_id: ArcheId, bundle_id: BundleId) -> ArcheId {
        let arche = unsafe { self.archetypes.get_unchecked(arche_id) };
        let bundle = unsafe { self.bundles.get_unchecked(bundle_id) };

        let expect_dense_len = arche.dense_components().len() + bundle.dense_components().len();
        let expect_sparse_len = arche.sparse_components().len() + bundle.sparse_components().len();

        let mut dense = Vec::with_capacity(expect_dense_len + expect_sparse_len);
        dense.extend_from_slice(arche.dense_components());
        dense.extend_from_slice(bundle.dense_components());
        dense.sort();
        dense.dedup();

        let mut sparse = Vec::with_capacity(expect_sparse_len);
        sparse.extend_from_slice(arche.sparse_components());
        sparse.extend_from_slice(bundle.sparse_components());
        sparse.sort();
        sparse.dedup();

        let dense_len = dense.len();
        dense.append(&mut sparse);

        if let Some(result) = self.archetypes.get_id(&dense) {
            let arche = unsafe { self.archetypes.get_unchecked_mut(arche_id) };
            unsafe {
                arche.set_after_insert(bundle_id, result);
            }
            return result;
        }

        let components = <Arc<[ComponentId]>>::from(dense);

        let table_id = unsafe {
            let sparse: &[ComponentId] = &components[dense_len..];
            self.storages.maps.register(&self.components, sparse);
            let denses: &[ComponentId] = &components[..dense_len];
            self.storages.tables.register(&self.components, denses)
        };

        unsafe {
            let result = self.archetypes.register(table_id, dense_len, components);
            let arche = self.archetypes.get_unchecked_mut(arche_id);
            arche.set_after_insert(bundle_id, result);
            result
        }
    }

    pub fn arche_after_remove(&mut self, arche_id: ArcheId, bundle_id: BundleId) -> ArcheId {
        let arche = unsafe { self.archetypes.get_unchecked(arche_id) };
        if let Some(cached) = arche.after_remove(bundle_id) {
            return cached;
        }
        unsafe { self.after_remove_slow(arche_id, bundle_id) }
    }

    #[cold]
    #[inline(never)]
    unsafe fn after_remove_slow(&mut self, arche_id: ArcheId, bundle_id: BundleId) -> ArcheId {
        let arche = unsafe { self.archetypes.get_unchecked(arche_id) };
        let bundle = unsafe { self.bundles.get_unchecked(bundle_id) };

        assert! {
            bundle.components().iter().all(|&id|arche.contains_component(id)),
            "invalid remove operation: remove {:?} from {:?}",
            bundle.components(),
            arche.components(),
        }

        let dense_upper = arche.dense_components().len();
        let mut dense: SparseHashSet<ComponentId> = SparseHashSet::with_capacity(dense_upper);
        dense.extend(arche.dense_components());
        bundle.dense_components().iter().for_each(|&id| {
            dense.remove(&id);
        });

        let sparse_upper = arche.sparse_components().len();
        let mut sparse: SparseHashSet<ComponentId> = SparseHashSet::with_capacity(sparse_upper);
        sparse.extend(arche.sparse_components());
        bundle.sparse_components().iter().for_each(|&id| {
            dense.remove(&id);
        });

        // HACK: `Collector` requires a mutable reference to `Components`, but accessing
        // a component's `Required` needs a shared reference, violating aliasing rules.
        // Using raw pointers to bypass this and avoid complexity. The caller ensures
        // safety as the internal implementation is deterministic.
        let mut ptr = NonNull::from_mut(&mut self.components);
        let mut collector = ComponentCollector::new(unsafe { ptr.as_mut() });
        dense.iter().chain(sparse.iter()).for_each(|&id| {
            let info = unsafe { ptr.as_ref().get_unchecked(id) };
            if let Some(required) = info.required() {
                required.collect(&mut collector);
            }
        });

        let CollectResult {
            dense: mut dense_vec,
            sparse: mut sparse_vec,
        } = collector.unsorted();
        dense_vec.extend(dense);
        dense_vec.sort();
        dense_vec.dedup();
        sparse_vec.extend(sparse);
        sparse_vec.sort();
        sparse_vec.dedup();

        let dense_len = dense_vec.len();
        dense_vec.append(&mut sparse_vec);

        if let Some(result) = self.archetypes.get_id(&dense_vec) {
            let arche = unsafe { self.archetypes.get_unchecked_mut(arche_id) };
            unsafe {
                arche.set_after_insert(bundle_id, result);
            }
            return result;
        }

        let components = <Arc<[ComponentId]>>::from(dense_vec);

        let table_id = unsafe {
            let sparse: &[ComponentId] = &components[dense_len..];
            self.storages.maps.register(&self.components, sparse);
            let denses: &[ComponentId] = &components[..dense_len];
            self.storages.tables.register(&self.components, denses)
        };

        unsafe {
            let result = self.archetypes.register(table_id, dense_len, components);
            let arche = self.archetypes.get_unchecked_mut(arche_id);
            arche.set_after_insert(bundle_id, result);
            result
        }
    }
}


use std::vec::Vec;

use vc_os::sync::Arc;

use crate::archetype::ArcheId;
use crate::component::{CollectResult, ComponentCollector, ComponentId, ComponentStorage};
use crate::world::World;

impl World {
    pub unsafe fn arche_after_insert(&mut self, arche_id: ArcheId, comp_id: ComponentId) -> ArcheId {
        let arche = unsafe { self.archetypes.get_unchecked(arche_id) };
        if let Some(cached) = arche.after_insert(comp_id) {
            return cached;
        }
        unsafe { self.after_insert_slow(arche_id, comp_id) }
    }

    #[cold]
    #[inline(never)]
    unsafe fn after_insert_slow(&mut self, arche_id: ArcheId, comp_id: ComponentId) -> ArcheId {
        let arche = unsafe { self.archetypes.get_unchecked_mut(arche_id) };

        if arche.contains_component(comp_id) {
            unsafe { arche.set_after_insert(comp_id, arche_id); }
            return arche_id;
        }

        let info = unsafe { self.components.get_unchecked(comp_id) };
        let storage = info.storage();
        let collector_required = info.collect_required();

        let mut collector = ComponentCollector::new(&mut self.components);
        unsafe { collector_required(&mut collector); }
        let CollectResult{
            mut dense,
            mut sparse,
        } = collector.unsorted();

        match storage {
            ComponentStorage::Dense => dense.push(comp_id),
            ComponentStorage::Sparse => sparse.push(comp_id),
        }

        dense.extend(arche.dense_components());
        sparse.extend(arche.sparse_components());
        dense.sort();
        sparse.sort();
        dense.dedup();
        sparse.dedup();

        let dense_len = dense.len();
        dense.append(&mut sparse);

        if let Some(result) = self.archetypes.get_id(&dense) {
            let arche = unsafe { self.archetypes.get_unchecked_mut(arche_id) };
            unsafe { arche.set_after_insert(comp_id, result); }
            return result;
        }

        let components = <Arc<[ComponentId]>>::from(dense);

        let table_id = unsafe {
            let sparse: &[ComponentId] = &components[dense_len..];
            self.storages.maps.register(&self.components, sparse);
            let denses: &[ComponentId] = &components[0..dense_len];
            self.storages.tables.register(&self.components, denses)
        };

        unsafe {
            let result = self.archetypes.register(table_id, dense_len, components);
            let arche = self.archetypes.get_unchecked_mut(arche_id);
            arche.set_after_insert(comp_id, result);
            result
        }
    }

    pub unsafe fn arche_after_remove(&mut self, arche_id: ArcheId, comp_id: ComponentId) -> ArcheId {
        let arche = unsafe { self.archetypes.get_unchecked(arche_id) };

        debug_assert!(arche.contains_component(comp_id));

        if let Some(cached) = arche.after_remove(comp_id) {
            return cached;
        }
        unsafe { self.after_remove_slow(arche_id, comp_id) }
    }

    #[cold]
    #[inline(never)]
    unsafe fn after_remove_slow(&mut self, arche_id: ArcheId, comp_id: ComponentId) -> ArcheId {
        let arche = unsafe { self.archetypes.get_unchecked_mut(arche_id) };

        let old_components = arche.components();
        let mut collector_fn = Vec::with_capacity(old_components.len());
        old_components.iter().for_each(|&cid| {
            if cid != comp_id {
                let info = unsafe { self.components.get_unchecked(comp_id) };
                collector_fn.push(info.collect_required());
            }
        });

        let mut collector = ComponentCollector::new(&mut self.components);
        collector_fn.iter().for_each(|&func| unsafe {
            func(&mut collector);
        });

        let CollectResult{
            mut dense,
            mut sparse,
        } = collector.sorted();
        let dense_len = dense.len();
        dense.append(&mut sparse);

        if let Some(result) = self.archetypes.get_id(&dense) {
            let arche = unsafe { self.archetypes.get_unchecked_mut(arche_id) };
            unsafe { arche.set_after_remove(comp_id, result); }
            return result;
        }

        let components = <Arc<[ComponentId]>>::from(dense);

        let table_id = unsafe {
            let sparse: &[ComponentId] = &components[dense_len..];
            self.storages.maps.register(&self.components, sparse);
            let denses: &[ComponentId] = &components[0..dense_len];
            self.storages.tables.register(&self.components, denses)
        };

        unsafe {
            let result = self.archetypes.register(table_id, dense_len, components);
            let arche = self.archetypes.get_unchecked_mut(arche_id);
            arche.set_after_remove(comp_id, result);
            result
        }
    }
}

use alloc::boxed::Box;
use alloc::vec::Vec;
use vc_utils::hash::SparseHashSet;
use vc_utils::index::{SparseIndexMap, SparseIndexSet};

use super::BundleId;
use crate::component::RequiredComponent;
use crate::component::{ComponentId, Components};
use crate::storage::Storages;

pub struct BundleInfo {
    pub(super) id: BundleId,
    pub(super) contributed_components: Box<[ComponentId]>,
    pub(super) required_components: Box<[RequiredComponent]>,
}

impl BundleInfo {
    pub unsafe fn new(
        bundle_name: &'static str,
        storages: &mut Storages,
        components: &Components,
        mut component_ids: Vec<ComponentId>,
        id: BundleId,
    ) -> BundleInfo {
        #[cold]
        #[inline(never)]
        fn duplicated_component(
            bundle_name: &'static str,
            components: &Components,
            component_ids: Vec<ComponentId>,
        ) -> ! {
            let mut seen = <SparseHashSet<ComponentId>>::new();
            let mut dups = Vec::new();

            for id in component_ids {
                if !seen.insert(id) {
                    dups.push(id);
                }
            }
            let names = dups
                .into_iter()
                .map(|id| components.get_debug_name(id))
                .collect::<Vec<_>>();

            panic!("Bundle {bundle_name} has duplicate components: {names:?}")
        }

        let explicit_component_ids = component_ids
            .iter()
            .copied()
            .collect::<SparseIndexSet<ComponentId>>();

        if explicit_component_ids.len() != component_ids.len() {
            duplicated_component(bundle_name, components, component_ids);
        }

        let mut depth_first_components = SparseIndexMap::<ComponentId, RequiredComponent>::new();
        for &component_id in &component_ids {
            // SAFETY: caller has verified that all ids are valid
            let info = unsafe { components.get_info_unchecked(component_id) };

            for (&required_id, required_component) in &info.required_components().all {
                depth_first_components
                    .entry(required_id)
                    .or_insert_with(|| required_component.clone());
            }

            storages.prepare_component(info);
        }

        let required_components = depth_first_components
            .into_iter()
            .filter(|&(required_id, _)| !explicit_component_ids.contains(&required_id))
            .inspect(|&(required_id, _)| {
                // SAFETY: These ids came out of the passed `components`, so they must be valid.
                storages.prepare_component(unsafe { components.get_info_unchecked(required_id) });
                component_ids.push(required_id);
            })
            .map(|(_, required_component)| required_component)
            .collect::<Box<[RequiredComponent]>>();

        BundleInfo {
            id,
            contributed_components: component_ids.into_boxed_slice(),
            required_components,
        }
    }

    #[inline]
    pub const fn id(&self) -> BundleId {
        self.id
    }

    #[inline]
    pub fn explicit_components_len(&self) -> usize {
        self.contributed_components.len() - self.required_components.len()
    }

    #[inline]
    pub fn contributed_components(&self) -> &[ComponentId] {
        &self.contributed_components
    }

    #[inline]
    pub fn explicit_components(&self) -> &[ComponentId] {
        &self.contributed_components[0..self.explicit_components_len()]
    }

    #[inline]
    pub fn required_components(&self) -> &[ComponentId] {
        &self.contributed_components[self.explicit_components_len()..]
    }

    #[inline]
    pub fn iter_explicit_components(&self) -> impl Iterator<Item = ComponentId> + Clone + '_ {
        self.explicit_components().iter().copied()
    }

    #[inline]
    pub fn iter_contributed_components(&self) -> impl Iterator<Item = ComponentId> + Clone + '_ {
        self.contributed_components().iter().copied()
    }

    #[inline]
    pub fn iter_required_components(&self) -> impl Iterator<Item = ComponentId> + '_ {
        self.required_components().iter().copied()
    }
}

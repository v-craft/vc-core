#![expect(unsafe_code)]

use alloc::boxed::Box;
use alloc::vec::Vec;
use vc_utils::hash::SparseHashMap;

use super::ArchetypeId;

use crate::bundle::{BundleComponentStatus, BundleId, ComponentStatus};
use crate::cfg;
use crate::component::ComponentId;
use crate::component::RequiredComponent;

// -----------------------------------------------------------------------------
// ArchetypeWithBundle

pub struct ArchetypeInsertedBundle {
    pub archetype_id: ArchetypeId,
    pub required_components: Box<[RequiredComponent]>,
    bundle_status: Box<[ComponentStatus]>,
    inserted: Box<[ComponentId]>,
    added_len: usize,
}

impl BundleComponentStatus for ArchetypeInsertedBundle {
    unsafe fn get_status(&self, index: usize) -> ComponentStatus {
        cfg::debug! { assert!(index < self.bundle_status.len()); }
        unsafe { *self.bundle_status.get_unchecked(index) }
    }
}

impl ArchetypeInsertedBundle {
    pub fn inserted(&self) -> &[ComponentId] {
        &self.inserted
    }

    pub fn added(&self) -> &[ComponentId] {
        cfg::debug! { assert!(self.added_len <= self.bundle_status.len()); }
        // SAFETY: `added_len` is always in range `0..=inserted.len()`
        unsafe { self.inserted.get_unchecked(..self.added_len) }
    }

    pub fn existing(&self) -> &[ComponentId] {
        cfg::debug! { assert!(self.added_len <= self.bundle_status.len()); }
        // SAFETY: `added_len` is always in range `0..=inserted.len()`
        unsafe { self.inserted.get_unchecked(self.added_len..) }
    }
}

// -----------------------------------------------------------------------------
// Edges

pub struct Edges {
    insert_bundle: SparseHashMap<BundleId, ArchetypeInsertedBundle>,
    remove_bundle: SparseHashMap<BundleId, Option<ArchetypeId>>,
    take_bundle: SparseHashMap<BundleId, Option<ArchetypeId>>,
}

impl Edges {
    pub const fn empty() -> Self {
        Self {
            insert_bundle: SparseHashMap::new(),
            remove_bundle: SparseHashMap::new(),
            take_bundle: SparseHashMap::new(),
        }
    }

    #[inline]
    pub fn get_archetype_inserted_bundle_internal(
        &self,
        bundle_id: BundleId,
    ) -> Option<&ArchetypeInsertedBundle> {
        self.insert_bundle.get(&bundle_id)
    }

    #[inline]
    pub fn get_archetype_inserted_bundle(&self, bundle_id: BundleId) -> Option<ArchetypeId> {
        self.get_archetype_inserted_bundle_internal(bundle_id)
            .map(|arche| arche.archetype_id)
    }

    #[inline]
    pub fn cache_archetype_inserted_bundle(
        &mut self,
        bundle_id: BundleId,
        archetype_id: ArchetypeId,
        bundle_status: Box<[ComponentStatus]>,
        required_components: Box<[RequiredComponent]>,
        mut added: Vec<ComponentId>,
        existing: Vec<ComponentId>,
    ) {
        let added_len = added.len();
        // Make sure `extend` doesn't over-reserve, since the conversion to `Box<[_]>` would reallocate to shrink.
        added.reserve_exact(existing.len());
        added.extend(existing);
        self.insert_bundle.insert(
            bundle_id,
            ArchetypeInsertedBundle {
                archetype_id,
                bundle_status: bundle_status,
                required_components: required_components,
                added_len,
                inserted: added.into_boxed_slice(),
            },
        );
    }

    #[inline]
    pub fn get_archetype_removed_bundle(&self, bundle_id: BundleId) -> Option<Option<ArchetypeId>> {
        self.remove_bundle.get(&bundle_id).copied()
    }

    #[inline]
    pub fn cache_archetype_removed_bundle(
        &mut self,
        bundle_id: BundleId,
        archetype_id: Option<ArchetypeId>,
    ) {
        self.remove_bundle.insert(bundle_id, archetype_id);
    }

    #[inline]
    pub fn get_archetype_taken_bundle(&self, bundle_id: BundleId) -> Option<Option<ArchetypeId>> {
        self.take_bundle.get(&bundle_id).copied()
    }

    pub fn cache_archetype_taken_bundle(
        &mut self,
        bundle_id: BundleId,
        archetype_id: Option<ArchetypeId>,
    ) {
        self.take_bundle.insert(bundle_id, archetype_id);
    }
}

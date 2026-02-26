#![allow(clippy::len_without_is_empty, reason = "internal type")]

use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use core::fmt::Debug;

use vc_os::sync::Arc;
use vc_utils::hash::{HashMap, SparseHashSet};

use crate::archetype::{ArcheId, Archetype};
use crate::bundle::BundleId;
use crate::component::ComponentId;
use crate::storage::TableId;

// -----------------------------------------------------------------------------
// Archetypes

/// A collection of all archetypes in the ECS world.
///
/// # Overview
/// `Archetypes` serves as the central registry for all archetype instances,
/// providing efficient lookup and filtering capabilities for the ECS query system.
/// It maintains multiple indexing structures to support different access patterns:
///
/// - **Direct access**: By [`ArcheId`] (primary key)
/// - **Bundle-based**: Maps [`BundleId`] to the corresponding archetype
/// - **Component-based**: Maps each [`ComponentId`] to all archetypes containing it
/// - **Precise matching**: Maps exact component sets to their archetype IDs
///
/// # Initial State
/// Always contains at least one archetype: the **empty archetype** (no components),
/// which serves as the starting point for all entities.
pub struct Archetypes {
    /// Primary storage for all archetype instances.
    /// Index corresponds directly to [`ArcheId`].
    arches: Vec<Archetype>,
    /// Maps bundle IDs to their corresponding archetype IDs.
    bundle_map: Vec<Option<ArcheId>>,
    /// Inverted index mapping component IDs to sets of archetype IDs.
    component_map: Vec<SparseHashSet<ArcheId>>,
    /// Maps exact component sets to archetype IDs.
    precise_map: HashMap<Arc<[ComponentId]>, ArcheId>,
}

impl Debug for Archetypes {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.arches, f)
    }
}

impl Archetypes {
    /// Creates a new archetypes collection, initialized with the empty archetype.
    pub(crate) fn new() -> Self {
        let mut val = const {
            Archetypes {
                arches: Vec::new(),
                bundle_map: Vec::new(),
                component_map: Vec::new(),
                precise_map: HashMap::new(),
            }
        };

        let arche = unsafe { Archetype::new(ArcheId::EMPTY, TableId::EMPTY, 0, Arc::new([])) };
        val.arches.push(arche);
        val.bundle_map.push(Some(ArcheId::EMPTY));
        val.precise_map.insert(Arc::new([]), ArcheId::EMPTY);

        val
    }

    /// Inserts a mapping from a bundle ID to an archetype ID.
    ///
    /// This mapping enables fast archetype lookup when spawning entities
    /// from known bundles.
    ///
    /// # Safety
    /// This method is unsafe because it modifies internal indexing structures
    /// and requires the caller to uphold the following invariants:
    ///
    /// - **Bundle validity**: The `bundle_id` must be valid and properly initialized
    ///   (i.e., corresponds to a registered bundle type).
    /// - **Archetype validity**: The `arche_id` must reference a valid, already-registered
    ///   archetype that exactly matches the component set of the bundle.
    /// - **No concurrent access**: This method may resize the bundle map; ensure no
    ///   other operations are concurrently reading or writing the bundle map.
    pub(crate) unsafe fn set_bundle_map(&mut self, bundle_id: BundleId, arche_id: ArcheId) {
        #[cold]
        #[inline(never)]
        fn resize_bundle_map(map: &mut Vec<Option<ArcheId>>, len: usize) {
            map.reserve(len - map.len());
            map.resize_with(map.capacity(), || None);
        }

        let index = bundle_id.index();
        if index >= self.bundle_map.len() {
            resize_bundle_map(&mut self.bundle_map, index + 1);
        }
        unsafe {
            *self.bundle_map.get_unchecked_mut(index) = Some(arche_id);
        }
    }

    /// Registers a new archetype with the given component set.
    ///
    /// This method creates a new archetype and updates all indexing structures
    /// to make it discoverable through various lookup paths.
    ///
    /// # Safety
    /// This method is unsafe and requires the caller to ensure:
    ///
    /// - **Component validity**: All `ComponentId`s in `components` must be valid and
    ///   properly registered in the component registry.
    /// - **Uniqueness**: The exact component set must not already have an archetype
    ///   (no duplicates), unless intentionally creating a new archetype for the same
    ///   set (which would violate ECS invariants).
    /// - **Sorting**: The `components` slice must be sorted, as this is relied upon
    ///   for binary search operations in archetype methods.
    /// - **Bundle consistency**: If a bundle corresponds to this component set, its
    ///   mapping should be updated separately via [`insert_bundle_id`](Self::insert_bundle_id).
    pub(crate) unsafe fn register(
        &mut self,
        table_id: TableId,
        dense_len: usize,
        components: Arc<[ComponentId]>,
    ) -> ArcheId {
        #[cold]
        #[inline(never)]
        fn resize_component_map(map: &mut Vec<SparseHashSet<ArcheId>>, len: usize) {
            map.reserve(len - map.len());
            map.resize_with(map.capacity(), SparseHashSet::new);
        }

        let arche_id = ArcheId::new(self.arches.len() as u32);

        let arche = unsafe { Archetype::new(arche_id, table_id, dense_len, components.clone()) };

        self.arches.push(arche);

        components.iter().for_each(|&cid| {
            let index = cid.index();
            if index >= self.component_map.len() {
                resize_component_map(&mut self.component_map, index + 1);
            }
            unsafe {
                self.component_map
                    .get_unchecked_mut(index)
                    .insert_unique_unchecked(arche_id);
            }
        });

        self.precise_map.insert(components, arche_id);

        arche_id
    }
}

impl Archetypes {
    /// Returns the number of registered archetypes.
    ///
    /// Similar to [`Archetypes::version`] .
    #[inline]
    pub fn len(&self) -> usize {
        self.arches.len()
    }

    /// Returns a reference to the archetype with the given ID, if it exists.
    #[inline]
    pub fn get(&self, id: ArcheId) -> Option<&Archetype> {
        self.arches.get(id.index())
    }

    /// Returns a mutable reference to the archetype with the given ID, if it exists.
    #[inline]
    pub fn get_mut(&mut self, id: ArcheId) -> Option<&mut Archetype> {
        self.arches.get_mut(id.index())
    }

    /// Returns a reference to the archetype with the given ID without bounds checking.
    ///
    /// # Safety
    /// The caller must ensure that `id` is valid (within bounds of `arches`).
    /// Violating this condition leads to undefined behavior.
    #[inline]
    pub unsafe fn get_unchecked(&self, id: ArcheId) -> &Archetype {
        debug_assert!(id.index() < self.arches.len());
        unsafe { self.arches.get_unchecked(id.index()) }
    }

    /// Returns a mutable reference to the archetype with the given ID without bounds checking.
    ///
    /// # Safety
    /// The caller must ensure that `id` is valid (within bounds of `arches`).
    /// Violating this condition leads to undefined behavior.
    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, id: ArcheId) -> &mut Archetype {
        debug_assert!(id.index() < self.arches.len());
        unsafe { self.arches.get_unchecked_mut(id.index()) }
    }

    /// Finds the archetype ID for an exact component set.
    #[inline]
    pub fn get_id(&self, components: &[ComponentId]) -> Option<ArcheId> {
        self.precise_map.get(components).copied()
    }

    /// Returns the archetype ID associated with a specific bundle.
    #[inline]
    pub fn get_id_by_bundle(&self, id: BundleId) -> Option<ArcheId> {
        self.bundle_map.get(id.index()).and_then(|t| *t)
    }

    /// Creates a new filter builder for querying archetypes by component requirements.
    #[inline]
    pub fn filter(&self) -> ArcheFilter<'_> {
        ArcheFilter {
            arches: self,
            with: None,
            without: SparseHashSet::new(),
        }
    }
}

// -----------------------------------------------------------------------------
// ArcheFilter

/// A builder for filtering archetypes based on component requirements.
#[derive(Debug, Clone)]
pub struct ArcheFilter<'a> {
    /// Reference to the parent archetypes collection
    arches: &'a Archetypes,
    /// Set of candidate archetype IDs that satisfy all `with` constraints so far.
    /// `None` means "all archetypes" (initial state), which is distinct from
    /// an empty set (which would mean no archetypes can match).
    with: Option<SparseHashSet<ArcheId>>,
    /// Set of archetype IDs to exclude (those containing any `without` component).
    /// This grows as more `without` constraints are added.
    without: SparseHashSet<ArcheId>,
}

impl ArcheFilter<'_> {
    /// Adds a requirement that archetypes must contain the specified component.
    ///
    /// This narrows down the candidate set to only archetypes that include
    /// this component. If the component doesn't exist in any archetype,
    /// the filter becomes empty (no matches possible).
    pub fn with(&mut self, id: ComponentId) {
        if let Some(set) = self.arches.component_map.get(id.index()) {
            if let Some(with) = &mut self.with {
                with.retain(|x| set.contains(x));
            } else {
                let mut with = set.clone();
                with.retain(|x| !self.without.contains(x));
                self.with = Some(with);
            }
        } else {
            self.with = Some(SparseHashSet::new());
        }
    }

    /// Adds a requirement that archetypes must NOT contain the specified component.
    ///
    /// This excludes all archetypes that include this component from the results.
    pub fn without(&mut self, id: ComponentId) {
        if let Some(set) = self.arches.component_map.get(id.index()) {
            self.without.extend(set.iter());
            if let Some(with) = &mut self.with {
                with.retain(|x| !set.contains(x));
            }
        }
    }

    /// Adds a requirement that archetypes must NOT contain the specified component.
    ///
    /// This excludes all archetypes that include this component from the results.
    pub fn collect(self) -> Vec<ArcheId> {
        if let Some(with) = self.with {
            let mut ret: Vec<ArcheId> = with.into_iter().collect();
            ret.sort();
            ret
        } else {
            (0..self.arches.arches.len())
                .map(|idx| unsafe { ArcheId::new_unchecked(idx as u32) })
                .filter(|id| !self.without.contains(id))
                .collect()
        }
    }

    pub fn collect_to(self, set: &mut BTreeSet<ArcheId>) {
        if let Some(with) = self.with {
            with.into_iter().for_each(|item| {
                set.insert(item);
            });
        } else {
            (0..self.arches.arches.len())
                .map(|idx| unsafe { ArcheId::new_unchecked(idx as u32) })
                .filter(|id| !self.without.contains(id))
                .for_each(|item| {
                    set.insert(item);
                });
        }
    }
}

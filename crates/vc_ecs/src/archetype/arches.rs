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
pub struct Archetypes {
    arches: Vec<Archetype>,
    bundle_map: Vec<Option<ArcheId>>,
    component_map: Vec<SparseHashSet<ArcheId>>,
    precise_map: HashMap<Arc<[ComponentId]>, ArcheId>,
}

impl Debug for Archetypes {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.arches, f)
    }
}

impl Archetypes {
    /// Creates a new archetypes, initializes with
    /// the empty archetype (no components),
    pub(crate) fn new() -> Self {
        let mut val = const {
            Archetypes {
                arches: Vec::new(),
                bundle_map: Vec::new(),
                component_map: Vec::new(),
                precise_map: HashMap::new(),
            }
        };

        val.arches.push(Archetype {
            id: ArcheId::EMPTY,
            table_id: TableId::EMPTY,
            dense_len: 0,
            entities: Vec::new(),
            components: Arc::new([]),
        });
        val.bundle_map.push(Some(ArcheId::EMPTY));
        val.precise_map.insert(Arc::new([]), ArcheId::EMPTY);

        val
    }

    /// Inserts a mapping from bundle ID to archetype ID.
    ///
    /// # Safety
    /// - The bundle ID must be valid and properly initialized.
    /// - The archetype ID must reference a valid archetype.
    /// - This method may resize the bundle map; ensure no concurrent access.
    pub(crate) unsafe fn insert_bundle_id(&mut self, bundle_id: BundleId, arche_id: ArcheId) {
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
    /// # Safety
    /// - Component IDs must be valid and properly registered.
    /// - The component set must be unique (no duplicate archetype with same set).
    /// - Bundle ID must not already have a mapping (unless intentionally overwriting).
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

        let id = ArcheId::new(self.arches.len() as u32);

        self.arches.push(Archetype {
            id,
            table_id,
            dense_len,
            entities: Vec::new(),
            components: components.clone(),
        });

        components.iter().for_each(|&cid| {
            let index = cid.index();
            if index >= self.component_map.len() {
                resize_component_map(&mut self.component_map, index + 1);
            }
            unsafe {
                self.component_map.get_unchecked_mut(index).insert(id);
            }
        });

        self.precise_map.insert(components, id);

        id
    }

    /// Returns the current version (number of archetypes).
    ///
    /// The version increments each time a new archetype is created.
    #[inline]
    pub fn version(&self) -> u32 {
        // See `ArcheId::new`, arches.len is <= u32::MAX.
        self.arches.len() as u32
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
    /// `id` is in valid.
    #[inline]
    pub unsafe fn get_unchecked(&self, id: ArcheId) -> &Archetype {
        debug_assert!(id.index() < self.arches.len());
        unsafe { self.arches.get_unchecked(id.index()) }
    }

    /// Returns a mutable reference to the archetype with the given ID without bounds checking.
    ///
    /// # Safety
    /// `id` is in valid.
    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, id: ArcheId) -> &mut Archetype {
        debug_assert!(id.index() < self.arches.len());
        unsafe { self.arches.get_unchecked_mut(id.index()) }
    }

    /// Returns the archetype ID for an exact set of components, if it exists.
    #[inline]
    pub fn get_id(&self, components: &[ComponentId]) -> Option<ArcheId> {
        self.precise_map.get(components).copied()
    }

    /// Returns the archetype ID associated with a bundle ID, if it exists.
    #[inline]
    pub fn get_by_bundle(&self, id: BundleId) -> Option<ArcheId> {
        self.bundle_map.get(id.index()).and_then(|t| *t)
    }

    /// Creates a new filter for querying archetypes by component presence/absence.
    #[inline]
    pub fn get_filter(&self) -> ArcheFilter<'_> {
        ArcheFilter {
            arches: self,
            with: None,
            without: SparseHashSet::new(),
        }
    }
}

// -----------------------------------------------------------------------------
// ArcheFilter

#[derive(Debug)]
pub struct ArcheFilter<'a> {
    arches: &'a Archetypes,
    // ↓ None means all ArcheId, instead of empty.
    with: Option<SparseHashSet<ArcheId>>,
    without: SparseHashSet<ArcheId>,
}

impl ArcheFilter<'_> {
    /// Adds a requirement that archetypes must contain the specified component.
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
    pub fn without(&mut self, id: ComponentId) {
        if let Some(set) = self.arches.component_map.get(id.index()) {
            self.without.extend(set.iter());
            if let Some(with) = &mut self.with {
                with.retain(|x| !set.contains(x));
            }
        }
    }

    pub fn filter(&self, id: ArcheId) -> bool {
        if let Some(with) = &self.with {
            with.contains(&id)
        } else {
            !self.without.contains(&id)
        }
    }
}

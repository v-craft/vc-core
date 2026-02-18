// -----------------------------------------------------------------------------
// Inline

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::fmt::Debug;

use nonmax::NonMaxU32;
use vc_os::sync::Arc;
use vc_utils::hash::{HashMap, SparseHashMap, SparseHashSet};

use super::ArchetypeId;

use crate::bundle::BundleId;
use crate::component::ComponentId;
use crate::storage::{StorageIndex, StorageType, TableId};

// -----------------------------------------------------------------------------
// Archetype

pub struct Archetype {
    pub(crate) id: ArchetypeId,
    pub(crate) table_id: TableId,
    // The number of components stored in the table
    pub(crate) in_table: u32,
    // - `[0..in_table]` stored in Table
    // - `[in_table..]` stored in SparseSets
    pub(crate) components: Arc<[ComponentId]>,
    // - `[0..in_table]` is table_column_index
    // - `[in_table..]` is sparse_column_index
    pub(crate) storage_indices: Box<[StorageIndex]>,
    pub(crate) after_insert: SparseHashMap<BundleId, ArchetypeId>,
    pub(crate) after_remove: SparseHashMap<BundleId, ArchetypeId>,
}

impl Debug for Archetype {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let in_table = self.in_table as usize;
        let tables = &self.components[0..in_table];
        let sparse = &self.components[in_table..];

        f.debug_struct("Archetype")
            .field("id", &self.id)
            .field("table_id", &self.table_id)
            .field("in_table", &tables)
            .field("in_sparse_set", &sparse)
            .finish()
    }
}

impl Archetype {
    #[inline(always)]
    pub fn sparse_set_components(&self) -> &[ComponentId] {
        &self.components[self.in_table as usize..]
    }

    #[inline(always)]
    pub fn table_components(&self) -> &[ComponentId] {
        &self.components[0..self.in_table as usize]
    }

    pub fn get_after_insert(&self, bundle_id: BundleId) -> Option<ArchetypeId> {
        self.after_insert.get(&bundle_id).copied()
    }

    pub fn get_after_remove(&self, bundle_id: BundleId) -> Option<ArchetypeId> {
        self.after_remove.get(&bundle_id).copied()
    }

    pub fn get_storage_info(&self, component_id: ComponentId) -> (StorageType, StorageIndex) {
        let table_len = self.in_table as usize;

        let table_comps = &self.components[0..table_len];
        if let Ok(idx) = table_comps.binary_search(&component_id) {
            return (StorageType::Table, self.storage_indices[idx]);
        }

        let sparse_comps = &self.components[table_len..];
        if let Ok(idx) = sparse_comps.binary_search(&component_id) {
            return (StorageType::SparseSet, self.storage_indices[idx]);
        }

        unreachable!("The given component was not found")
    }

    pub fn contains_component(&self, component_id: ComponentId) -> bool {
        let table_len = self.in_table as usize;

        let table_comps = &self.components[0..table_len];
        if table_comps.binary_search(&component_id).is_ok() {
            return true;
        }

        let sparse_comps = &self.components[table_len..];
        if sparse_comps.binary_search(&component_id).is_ok() {
            return true;
        }

        false
    }
}

// -----------------------------------------------------------------------------
// Archetypes

pub struct Archetypes {
    pub(crate) archetypes: Vec<Archetype>,
    pub(crate) bundle_map: Vec<Option<ArchetypeId>>,
    pub(crate) component_map: Vec<SparseHashSet<ArchetypeId>>,
    pub(crate) precise_map: HashMap<Arc<[ComponentId]>, ArchetypeId>,
}

impl Debug for Archetypes {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.archetypes, f)
    }
}

impl Archetypes {
    pub(crate) fn new() -> Self {
        let mut val = const {
            Archetypes {
                archetypes: Vec::new(),
                bundle_map: Vec::new(),
                component_map: Vec::new(),
                precise_map: HashMap::new(),
            }
        };

        let empty_id = unsafe {
            val.register(
                BundleId::EMPTY,
                TableId::EMPTY,
                0,
                Arc::new([]),
                Box::new([]),
            )
        };

        assert_eq!(empty_id, ArchetypeId::EMPTY);

        val
    }

    #[cold]
    #[inline(never)]
    fn resize_component_map(&mut self, len: usize) {
        self.component_map.reserve(len - self.component_map.len());
        self.component_map
            .resize_with(self.component_map.capacity(), SparseHashSet::new);
    }

    #[cold]
    #[inline(never)]
    fn resize_bundle_map(&mut self, len: usize) {
        self.bundle_map.reserve(len - self.bundle_map.len());
        self.bundle_map.resize(self.bundle_map.capacity(), None);
    }

    pub(crate) unsafe fn register(
        &mut self,
        bundle_id: BundleId,
        table_id: TableId,
        in_table: u32,
        components: Arc<[ComponentId]>,
        storage_indices: Box<[StorageIndex]>,
    ) -> ArchetypeId {
        let id = unsafe { NonMaxU32::new_unchecked(self.archetypes.len() as u32) };
        assert!(id < NonMaxU32::MAX, "too many archetypes");
        let id = ArchetypeId::new(id);

        self.archetypes.push(Archetype {
            id,
            table_id,
            in_table,
            storage_indices,
            components: components.clone(),
            after_insert: SparseHashMap::new(),
            after_remove: SparseHashMap::new(),
        });

        let bundle_index = bundle_id.index();
        if self.bundle_map.len() <= bundle_index {
            self.resize_bundle_map(bundle_index + 1);
        }
        unsafe {
            *self.bundle_map.get_unchecked_mut(bundle_index) = Some(id);
        }

        components.iter().for_each(|cid| {
            let index = cid.index();
            if self.component_map.len() <= index {
                self.resize_component_map(index + 1);
            }
            unsafe {
                self.component_map
                    .get_unchecked_mut(index)
                    .insert_unique_unchecked(id);
            }
        });

        self.precise_map.insert(components, id);

        id
    }

    /// Returns a reference to an `Archetype` depending on the `ArchetypeId`.
    #[inline]
    pub unsafe fn get(&self, id: ArchetypeId) -> &Archetype {
        unsafe { self.archetypes.get_unchecked(id.index()) }
    }

    #[inline]
    pub fn get_id(&self, components: &[ComponentId]) -> Option<ArchetypeId> {
        self.precise_map.get(components).copied()
    }

    #[inline]
    pub fn get_id_by_bundle(&self, id: BundleId) -> Option<ArchetypeId> {
        self.bundle_map.get(id.index()).and_then(|v| *v)
    }

    #[inline]
    pub fn get_id_by_components(&self, components: &[ComponentId]) -> Option<ArchetypeId> {
        self.precise_map.get(components).copied()
    }

    pub(crate) fn set_bundle_map(&mut self, bundle_id: BundleId, archetype_id: ArchetypeId) {
        let index = bundle_id.index();
        if self.bundle_map.len() <= index {
            self.resize_bundle_map(index + 1);
        }
        unsafe {
            *self.bundle_map.get_unchecked_mut(index) = Some(archetype_id);
        }
    }
}

// -----------------------------------------------------------------------------
// Tests

#[cfg(test)]
mod tests {
    use crate::archetype::ArchetypeId;
    use crate::storage::TableId;

    use super::Archetypes;

    #[test]
    fn archetypes_new() {
        let archetypes = Archetypes::new();
        let id = archetypes.get_id(&[]).unwrap();
        assert_eq!(id, ArchetypeId::EMPTY);

        let archetype = unsafe { archetypes.get(id) };
        assert_eq!(archetype.id, ArchetypeId::EMPTY);
        assert_eq!(archetype.table_id, TableId::EMPTY);
        assert_eq!(&*archetype.components, &[]);
    }
}

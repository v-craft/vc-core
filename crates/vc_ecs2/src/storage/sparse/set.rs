#![allow(clippy::new_without_default, reason = "internal type")]

use alloc::vec::Vec;

use crate::component::{ComponentId, ComponentIndices};
use crate::storage::StorageIndex;

// -----------------------------------------------------------------------------
// SparseSet

#[derive(Debug)]
pub(crate) struct SparseSet<V> {
    data: Vec<V>,
    ids: Vec<ComponentId>,
    indices: ComponentIndices,
}

// -----------------------------------------------------------------------------
// Basic Methods

impl<V> SparseSet<V> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            data: Vec::new(),
            ids: Vec::new(),
            indices: ComponentIndices::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            ids: Vec::with_capacity(capacity),
            indices: ComponentIndices::new(),
        }
    }

    #[inline]
    pub unsafe fn get_index(&self, id: ComponentId) -> StorageIndex {
        unsafe { StorageIndex::new(self.indices.get_unchecked(id)) }
    }

    #[inline(always)]
    pub unsafe fn get(&self, index: StorageIndex) -> &V {
        debug_assert!(index.index() < self.data.len());
        unsafe { self.data.get_unchecked(index.index()) }
    }

    #[inline(always)]
    pub unsafe fn get_mut(&mut self, index: StorageIndex) -> &mut V {
        debug_assert!(index.index() < self.data.len());
        unsafe { self.data.get_unchecked_mut(index.index()) }
    }

    pub fn values(&self) -> impl ExactSizeIterator<Item = &V> {
        self.data.iter()
    }

    pub fn values_mut(&mut self) -> impl ExactSizeIterator<Item = &mut V> {
        self.data.iter_mut()
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.ids.clear();
        self.indices.clear();
    }
}


// -----------------------------------------------------------------------------
// Optional Methods

impl<V> SparseSet<V> {
    // #[inline]
    // pub fn len(&self) -> usize {
    //     self.data.len()
    // }

    // #[inline]
    // pub fn is_empty(&self) -> bool {
    //     self.data.len() == 0
    // }

    // #[inline]
    // pub fn capacity(&self) -> usize {
    //     self.data.capacity()
    // }

    // #[inline]
    // pub fn contains(&self, id: ComponentId) -> bool {
    //     self.indices.contains(id)
    // }

    // #[inline]
    // pub fn get_raw_index(&self, id: ComponentId) -> Option<u32> {
    //     self.indices.get(id)
    // }

    // #[inline]
    // pub fn component_ids(&self) -> &[ComponentId] {
    //     &self.ids
    // }

    // #[inline]
    // pub fn get(&self, id: ComponentId) -> Option<&V> {
    //     self.indices.map(id, |index| unsafe {
    //         self.data.get_unchecked(index as usize)
    //     })
    // }

    // #[inline]
    // pub fn get_mut(&mut self, id: ComponentId) -> Option<&mut V> {
    //     self.indices.map(id, |index| unsafe {
    //         self.data.get_unchecked_mut(index as usize)
    //     })
    // }

    // #[inline(always)]
    // pub unsafe fn get_raw(&self, raw_index: u32) -> &V {
    //     cfg::debug! { assert!((raw_index as usize) < self.data.len()); }
    //     unsafe { self.data.get_unchecked(raw_index as usize) }
    // }

    // #[inline(always)]
    // pub unsafe fn get_mut_raw(&mut self, raw_index: u32) -> &mut V {
    //     cfg::debug! { assert!((raw_index as usize) < self.data.len()); }
    //     unsafe { self.data.get_unchecked_mut(raw_index as usize) }
    // }

    // pub fn values(&self) -> impl ExactSizeIterator<Item = &V> {
    //     self.data.iter()
    // }

    // pub fn values_mut(&mut self) -> impl ExactSizeIterator<Item = &mut V> {
    //     self.data.iter_mut()
    // }

    // pub fn iter(&self) -> impl ExactSizeIterator<Item = (&ComponentId, &V)> {
    //     self.ids.iter().zip(self.data.iter())
    // }

    // pub fn iter_mut(&mut self) -> impl ExactSizeIterator<Item = (&ComponentId, &mut V)> {
    //     self.ids.iter().zip(self.data.iter_mut())
    // }

    // pub fn insert(&mut self, id: ComponentId, value: V) {
    //     if let Some(raw_index) = self.indices.get(id) {
    //         unsafe {
    //             *self.data.get_unchecked_mut(raw_index as usize) = value;
    //         }
    //     } else {
    //         let len = self.data.len();

    //         self.indices.set(id, len as u32);
    //         self.ids.push(id);
    //         self.data.push(value);
    //     }
    // }

    // pub fn clear(&mut self) {
    //     self.data.clear();
    //     self.ids.clear();
    //     self.indices.clear();
    // }

    // pub fn get_or_insert_with(&mut self, id: ComponentId, func: impl FnOnce() -> V) -> &mut V {
    //     if let Some(index) = self.indices.get(id) {
    //         // SAFETY: dense indices stored in self.sparse always exist
    //         unsafe { self.data.get_unchecked_mut(index as usize) }
    //     } else {
    //         let raw_index = self.data.len();

    //         cfg::debug! {
    //             assert!(raw_index < u32::MAX as usize);
    //         }

    //         let value = func();

    //         self.indices.set(id, raw_index as u32);
    //         self.ids.push(id);
    //         self.data.push(value);

    //         // SAFETY: dense index was just populated above
    //         unsafe { self.data.get_unchecked_mut(raw_index) }
    //     }
    // }

    // pub(crate) unsafe fn insert_unchecked(&mut self, id: ComponentId, value: V) {
    //     let len = self.data.len();

    //     self.indices.set(id, len as u32);
    //     self.ids.push(id);
    //     self.data.push(value);
    // }
}

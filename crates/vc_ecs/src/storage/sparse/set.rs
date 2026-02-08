#![expect(unsafe_code, reason = "original implementation need unsafe codes.")]

use alloc::vec::Vec;

use nonmax::NonMaxU32;

use crate::cfg;
use crate::component::{ComponentId, ComponentIndices};

// -----------------------------------------------------------------------------
// SparseSet

#[derive(Debug)]
pub struct SparseSet<V> {
    data: Vec<V>,
    ids: Vec<ComponentId>,
    indices: ComponentIndices,
}

// -----------------------------------------------------------------------------
// SparseSet Implementation

impl<V> SparseSet<V> {
    #[inline]
    pub const fn empty() -> Self {
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
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.len() == 0
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    #[inline]
    pub fn contains(&self, index: ComponentId) -> bool {
        self.indices.contains(index)
    }

    #[inline]
    pub fn get_raw_index(&self, index: ComponentId) -> Option<u32> {
        self.indices.get(index).map(|v| v.get())
    }

    #[inline]
    pub fn component_ids(&self) -> &[ComponentId] {
        &self.ids
    }

    #[inline]
    pub fn get(&self, id: ComponentId) -> Option<&V> {
        self.indices
            .get(id)
            .map(|index| unsafe { self.data.get_unchecked(index.get() as usize) })
    }

    #[inline]
    pub fn get_mut(&mut self, id: ComponentId) -> Option<&mut V> {
        self.indices
            .get(id)
            .map(|index| unsafe { self.data.get_unchecked_mut(index.get() as usize) })
    }

    #[inline(always)]
    pub unsafe fn get_raw(&self, raw_index: u32) -> &V {
        cfg::debug! { assert!((raw_index as usize) < self.data.len()); }
        unsafe { self.data.get_unchecked(raw_index as usize) }
    }

    #[inline(always)]
    pub unsafe fn get_mut_raw(&mut self, raw_index: u32) -> &mut V {
        cfg::debug! { assert!((raw_index as usize) < self.data.len()); }
        unsafe { self.data.get_unchecked_mut(raw_index as usize) }
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.data.iter()
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.data.iter_mut()
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = (&ComponentId, &V)> {
        self.ids.iter().zip(self.data.iter())
    }

    pub fn iter_mut(&mut self) -> impl ExactSizeIterator<Item = (&ComponentId, &mut V)> {
        self.ids.iter().zip(self.data.iter_mut())
    }

    pub fn insert(&mut self, id: ComponentId, value: V) -> u32 {
        if let Some(index) = self.indices.get(id) {
            let raw_index = index.get();
            unsafe {
                *self.data.get_unchecked_mut(raw_index as usize) = value;
            }
            raw_index
        } else {
            let len = self.data.len();

            cfg::debug! {
                assert!(len < u32::MAX as usize);
            }

            let raw_index = len as u32;

            self.indices
                .set(id, unsafe { NonMaxU32::new_unchecked(raw_index) });
            self.ids.push(id);
            self.data.push(value);

            raw_index
        }
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.ids.clear();
        self.indices.clear();
    }

    pub fn get_or_insert_with(&mut self, id: ComponentId, func: impl FnOnce() -> V) -> &mut V {
        if let Some(index) = self.indices.get(id) {
            // SAFETY: dense indices stored in self.sparse always exist
            unsafe { self.data.get_unchecked_mut(index.get() as usize) }
        } else {
            let raw_index = self.data.len();

            cfg::debug! {
                assert!(raw_index < u32::MAX as usize);
            }

            let value = func();

            self.indices
                .set(id, unsafe { NonMaxU32::new_unchecked(raw_index as u32) });
            self.ids.push(id);
            self.data.push(value);

            // SAFETY: dense index was just populated above
            unsafe { self.data.get_unchecked_mut(raw_index) }
        }
    }
}

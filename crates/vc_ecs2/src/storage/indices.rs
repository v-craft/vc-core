use core::fmt::Debug;

use alloc::vec::Vec;

use nonmax::NonMaxU32;

use crate::component::ComponentId;

// -----------------------------------------------------------------------------
// ComponentIndices

/// A two-level index table for mapping `ComponentId` to secondary indices.
///
/// This is used in `Archetypes` and `SparseSets`.
///
/// Using `NonMaxU32` instead of `u32` reduces memory usage by half.
pub(crate) struct ComponentIndices {
    indices: Vec<Option<NonMaxU32>>,
}

impl Debug for ComponentIndices {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut map = f.debug_map();
        self.indices.iter().enumerate().for_each(|(id, index)| {
            if let Some(index) = index {
                map.key(&id).value(&index.get());
            }
        });
        map.finish()
    }
}

impl ComponentIndices {
    /// Creates a new empty `ComponentIndices`.
    #[inline]
    pub const fn new() -> Self {
        Self {
            indices: Vec::new(),
        }
    }

    /// Returns `true` if the `ComponentId` exists in the mapping.
    #[inline]
    pub fn contains(&self, id: ComponentId) -> bool {
        matches!(self.indices.get(id.index()), Some(Some(_)))
    }

    /// Applies a function to the index associated with a
    /// `ComponentId` if it exists.
    #[inline]
    pub fn map<R>(&self, id: ComponentId, f: impl FnOnce(u32) -> R) -> Option<R> {
        if let Some(Some(x)) = self.indices.get(id.index()) {
            Some(f(x.get()))
        } else {
            None
        }
    }

    /// Retrieves the index associated with a `ComponentId`.
    #[inline]
    pub fn get(&self, id: ComponentId) -> Option<u32> {
        if let Some(Some(x)) = self.indices.get(id.index()) {
            Some(x.get())
        } else {
            None
        }
    }

    /// Sets or updates the index for `ComponentId`.
    #[inline]
    pub fn set(&mut self, id: ComponentId, value: u32) {
        #[cold]
        #[inline(never)]
        fn resize_indices(this: &mut ComponentIndices, len: usize) {
            let indices = &mut this.indices;
            indices.reserve(len - indices.len());
            indices.resize(indices.capacity(), None);
        }

        let index = id.index();
        if index >= self.indices.len() {
            resize_indices(self, index + 1);
        }
        // SAFETY: already resized.
        unsafe {
            *self.indices.get_unchecked_mut(index) = NonMaxU32::new(value);
        }
    }
}

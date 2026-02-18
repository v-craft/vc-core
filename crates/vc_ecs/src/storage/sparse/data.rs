#![allow(clippy::new_without_default, reason = "internal type")]

use alloc::vec::Vec;
use core::alloc::Layout;
use core::fmt::Debug;
use core::num::NonZeroUsize;

use vc_ptr::OwningPtr;
use vc_utils::hash::SparseHashMap;

use crate::borrow::{UntypedMut, UntypedRef};
use crate::cfg;
use crate::entity::EntityId;
use crate::storage::{AbortOnPanic, Column};
use crate::tick::{CheckTicks, Tick};
use crate::utils::DebugCheckedUnwrap;

// -----------------------------------------------------------------------------
// SparseSet

pub struct SparseSet {
    column: Column,
    entities: Vec<EntityId>,
    sparse: SparseHashMap<EntityId, u32>,
}

impl Debug for SparseSet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SparseSet")
            .field("entities", &self.entities)
            .finish()
    }
}

impl Drop for SparseSet {
    fn drop(&mut self) {
        let len = self.entity_count();
        let current_capacity = self.capacity();
        unsafe {
            self.column.dealloc(current_capacity, len);
        }
    }
}

unsafe impl Send for SparseSet {}
unsafe impl Sync for SparseSet {}

// -----------------------------------------------------------------------------
// Basic methods

impl SparseSet {
    #[inline]
    pub(crate) const unsafe fn new(
        layout: Layout,
        drop_fn: Option<unsafe fn(OwningPtr<'_>)>,
    ) -> Self {
        Self {
            column: unsafe { Column::new(layout, drop_fn) },
            entities: Vec::new(),
            sparse: SparseHashMap::new(),
        }
    }

    #[inline(always)]
    fn capacity(&self) -> usize {
        self.entities.capacity()
    }

    #[inline(always)]
    fn entity_count(&self) -> usize {
        self.entities.len()
    }

    #[inline]
    pub fn check_ticks(&mut self, check: CheckTicks) {
        unsafe { self.column.check_ticks(self.entities.len(), check) };
    }

    pub unsafe fn init_component(&mut self, id: EntityId, data: OwningPtr<'_>, tick: Tick) {
        #[cold]
        #[inline(never)]
        fn reserve_one(this: &mut SparseSet) {
            let abort_guard = AbortOnPanic;

            let old_capacity = this.capacity();
            this.entities.reserve(1);
            let new_capacity = this.entities.capacity();
            // Provide redundant space to reduce hash collisions.
            this.sparse.reserve(new_capacity);

            unsafe {
                let new_capacity = NonZeroUsize::new_unchecked(new_capacity);
                if let Some(current_capacity) = NonZeroUsize::new(old_capacity) {
                    this.column.realloc(current_capacity, new_capacity);
                } else {
                    this.column.alloc(new_capacity);
                }
            }

            ::core::mem::forget(abort_guard);
        }

        cfg::debug! {
            assert!( !self.sparse.contains_key(&id), "already exist" );
        }

        // SAFETY: `0 < EntityId < u32::MAX`, so `len < u32::MAX`.
        let last_index = self.entities.len();
        if last_index == self.entities.capacity() {
            reserve_one(self);
        }

        self.entities.push(id);
        unsafe {
            self.column.init_item(last_index, data, tick);
            self.sparse.insert(id, last_index as u32);
        }
    }

    pub unsafe fn remove_and_drop(&mut self, id: EntityId) {
        use crate::storage::VecRemoveExt;

        let index_u32 = unsafe { self.sparse.remove(&id).debug_checked_unwrap() };
        let index = index_u32 as usize;
        debug_assert_eq!(id, self.entities[index]);

        let last_index = self.entities.len() - 1;

        unsafe {
            if index != last_index {
                let swapped_id = self.entities.copy_return_not_last(last_index, index);
                *self.sparse.get_mut(&swapped_id).debug_checked_unwrap() = index_u32;
                self.column.swap_remove_and_drop_not_last(index, last_index);
            } else {
                self.entities.set_len(last_index);
                self.column.remove_and_drop_last(last_index);
            }
        }
    }

    #[inline]
    pub unsafe fn get_ref(&self, id: EntityId, last_run: Tick, this_run: Tick) -> UntypedRef<'_> {
        let index = unsafe { *self.sparse.get(&id).debug_checked_unwrap() as usize };
        debug_assert_eq!(id, self.entities[index]);

        unsafe { self.column.get_ref(index, last_run, this_run) }
    }

    #[inline]
    pub unsafe fn get_mut(
        &mut self,
        id: EntityId,
        last_run: Tick,
        this_run: Tick,
    ) -> UntypedMut<'_> {
        let index = unsafe { *self.sparse.get(&id).debug_checked_unwrap() as usize };
        debug_assert_eq!(id, self.entities[index]);

        unsafe { self.column.get_mut(index, last_run, this_run) }
    }
}

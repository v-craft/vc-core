#![allow(clippy::new_without_default, reason = "internal type")]

use alloc::vec::Vec;
use core::alloc::Layout;
use core::fmt::Debug;
use core::num::NonZeroUsize;

use vc_ptr::OwningPtr;
use vc_utils::hash::SparseHashMap;

use crate::cfg;
use crate::entity::EntityId;
use crate::storage::{AbortOnPanic, Column};
use crate::tick::{CheckTicks, Tick};
use crate::utils::DebugLocation;

// -----------------------------------------------------------------------------
// SparseComponent

pub struct SparseComponent {
    column: Column,
    entities: Vec<EntityId>,
    sparse: SparseHashMap<EntityId, u32>,
}

impl Debug for SparseComponent {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SparseComponent")
            .field("entities", &self.entities)
            .finish()
    }
}

impl Drop for SparseComponent {
    fn drop(&mut self) {
        let len = self.entity_count();
        let current_capacity = self.capacity();
        unsafe {
            self.column.dealloc(current_capacity, len);
        }
    }
}

unsafe impl Send for SparseComponent {}
unsafe impl Sync for SparseComponent {}

// -----------------------------------------------------------------------------
// Basic methods

impl SparseComponent {
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

    pub fn init(&mut self, id: EntityId, data: OwningPtr<'_>, tick: Tick, caller: DebugLocation) {
        #[cold]
        #[inline(never)]
        fn reserve_one(this: &mut SparseComponent) {
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
            self.column.init_item(last_index, data, tick, caller);
            self.sparse.insert(id, last_index as u32);
        }
    }
}

// -----------------------------------------------------------------------------
// Optional methods

impl SparseComponent {
    // // #[inline]
    // // pub fn get_drop_fn(&self) -> Option<unsafe fn(OwningPtr<'_>)> {
    // //     self.column.drop_fn()
    // // }

    // #[inline]
    // pub fn contains_entity(&self, id: EntityId) -> bool {
    //     self.sparse.contains_key(&id)
    // }

    // #[inline]
    // pub fn get_ptr(&self, id: EntityId) -> Option<Ptr<'_>> {
    //     self.sparse.get(&id).map(|&index| {
    //         let index = index as usize;
    //         debug_assert_eq!(id, self.entities[index]);

    //         unsafe { self.column.get_data_ptr(index) }
    //     })
    // }

    // #[inline]
    // pub fn get_ptr_mut(&mut self, id: EntityId) -> Option<PtrMut<'_>> {
    //     self.sparse.get(&id).map(|&index| {
    //         let index = index as usize;
    //         debug_assert_eq!(id, self.entities[index]);

    //         unsafe { self.column.get_data_ptr_mut(index) }
    //     })
    // }

    // #[inline]
    // pub fn get_ticks(&self, id: EntityId) -> Option<ComponentTicks> {
    //     let index = *self.sparse.get(&id)? as usize;
    //     debug_assert_eq!(id, self.entities[index]);

    //     unsafe {
    //         Some(ComponentTicks {
    //             added: self.column.get_added_tick(index),
    //             changed: self.column.get_changed_tick(index),
    //         })
    //     }
    // }

    // #[inline]
    // pub fn get_added_tick_cell(&self, id: EntityId) -> Option<&UnsafeCell<Tick>> {
    //     let index = *self.sparse.get(&id)? as usize;
    //     debug_assert_eq!(id, self.entities[index]);

    //     unsafe { Some(self.column.get_added_tick_cell(index)) }
    // }

    // #[inline]
    // pub fn get_changed_tick_cell(&self, id: EntityId) -> Option<&UnsafeCell<Tick>> {
    //     let index = *self.sparse.get(&id)? as usize;
    //     debug_assert_eq!(id, self.entities[index]);

    //     unsafe { Some(self.column.get_changed_tick_cell(index)) }
    // }

    // #[inline]
    // pub fn get_changed_by_cell(
    //     &self,
    //     id: EntityId,
    // ) -> DebugLocation<Option<&UnsafeCell<&'static Location<'static>>>> {
    //     cfg::debug! {
    //         if let Some(index) = self.sparse.get(&id) {
    //             let index = *index as usize;
    //             assert_eq!(id, self.entities[index]);

    //             let cb = unsafe {
    //                 self.column.get_changed_by_cell(index)
    //             };

    //             return DebugLocation::untranspose(Some(cb));
    //         }
    //     }

    //     DebugLocation::new(None)
    // }

    // #[inline]
    // pub fn get_untyped(&self, id: EntityId, last_run: Tick, this_run: Tick) -> Option<UntypedRef<'_>> {
    //     let index = *self.sparse.get(&id)? as usize;
    //     debug_assert_eq!(id, self.entities[index]);
    //     unsafe {
    //         Some(UntypedRef {
    //             value: self.column.get_data_ptr(index),
    //             ticks: ComponentTicksRef {
    //                 added: &*self.column.get_added_tick_cell(index).get(),
    //                 changed: &*self.column.get_changed_tick_cell(index).get(),
    //                 changed_by: self.column.get_changed_by_cell(index).map(|cb| &*cb.get()),
    //                 last_run,
    //                 this_run,
    //             }
    //         })
    //     }
    // }

    // #[inline]
    // pub fn get_untyped_mut(&mut self, id: EntityId, last_run: Tick, this_run: Tick) -> Option<UntypedMut<'_>> {
    //     let index = *self.sparse.get(&id)? as usize;
    //     debug_assert_eq!(id, self.entities[index]);
    //     unsafe {
    //         Some(UntypedMut {
    //             ticks: ComponentTicksMut {
    //                 added: &mut *self.column.get_added_tick_cell(index).get(),
    //                 changed: &mut *self.column.get_changed_tick_cell(index).get(),
    //                 changed_by: self.column.get_changed_by_cell(index).map(|cb| &mut *cb.get()),
    //                 last_run,
    //                 this_run,
    //             },
    //             value: self.column.get_data_ptr_mut(index),
    //         })
    //     }
    // }

    // #[must_use = "The returned pointer must be used to drop the removed component."]
    // pub fn take(&mut self, id: EntityId) -> Option<OwningPtr<'_>> {
    //     use crate::storage::VecCopyRemove;

    //     let index_u32 = self.sparse.remove(&id)?;
    //     let index = index_u32 as usize;
    //     debug_assert_eq!(id, self.entities[index]);

    //     let last_index = self.entities.len() - 1;

    //     unsafe {
    //         if index != last_index {
    //             let swapped_id = self.entities.copy_remove_last(index, last_index);
    //             *self.sparse.get_mut(&swapped_id).debug_checked_unwrap() = index_u32;
    //             Some(self.column.swap_remove_nonoverlapping(index, last_index))
    //         } else {
    //             self.entities.set_len(last_index);
    //             Some(self.column.get_data_ptr_mut(last_index).promote())
    //         }
    //     }
    // }

    // pub fn remove(&mut self, id: EntityId) -> bool {
    //     use crate::storage::VecCopyRemove;

    //     let Some(index_u32) = self.sparse.remove(&id) else {
    //         return false;
    //     };

    //     let index = index_u32 as usize;
    //     debug_assert_eq!(id, self.entities[index]);

    //     let last_index = self.entities.len() - 1;

    //     unsafe {
    //         if index != last_index {
    //             let swapped_id = self.entities.copy_remove_last(index, last_index);
    //             *self.sparse.get_mut(&swapped_id).debug_checked_unwrap() = index_u32;
    //             self.column.swap_remove_and_drop_nonoverlapping(index, last_index);
    //         } else {
    //             self.entities.set_len(last_index);
    //             self.column.drop_last(last_index);
    //         }
    //         true
    //     }
    // }
}

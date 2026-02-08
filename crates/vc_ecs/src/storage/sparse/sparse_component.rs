#![expect(unsafe_code, reason = "original implementation requires unsafe code.")]

use alloc::vec::Vec;
use core::alloc::Layout;
use core::cell::UnsafeCell;
use core::num::NonZeroUsize;
use core::panic::Location;

use vc_ptr::{OwningPtr, Ptr, PtrMut};
use vc_utils::hash::SparseHashMap;

use crate::cfg;
use crate::component::{ComponentTickCells, ComponentTicks};
use crate::entity::EntityId;
use crate::storage::{AbortOnPanic, Column};
use crate::tick::{CheckTicks, Tick};
use crate::utils::{DebugCheckedUnwrap, DebugLocation};

// -----------------------------------------------------------------------------
// SparseComponent

#[derive(Debug)]
pub struct SparseComponent {
    column: Column,
    entities: Vec<EntityId>,
    sparse: SparseHashMap<EntityId, u32>,
}

impl SparseComponent {
    #[inline]
    pub const unsafe fn empty(
        item_layout: Layout,
        drop_fn: Option<unsafe fn(OwningPtr<'_>)>,
    ) -> Self {
        Self {
            column: unsafe { Column::empty(item_layout, drop_fn) },
            entities: Vec::new(),
            sparse: SparseHashMap::new(),
        }
    }

    pub unsafe fn with_capacity(
        item_layout: Layout,
        drop_fn: Option<unsafe fn(OwningPtr<'_>)>,
        capacity: usize,
    ) -> Self {
        let hash_capacity = capacity + (capacity >> 1);

        Self {
            column: unsafe { Column::with_capacity(item_layout, drop_fn, capacity) },
            entities: Vec::with_capacity(capacity),
            sparse: SparseHashMap::with_capacity(hash_capacity),
        }
    }

    #[inline]
    pub fn drop_fn(&self) -> Option<unsafe fn(OwningPtr<'_>)> {
        self.column.drop_fn()
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.entities.capacity()
    }

    #[inline(always)]
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    pub fn clear_entities(&mut self) {
        let len = self.entity_count();
        self.entities.clear();
        self.sparse.clear();
        unsafe {
            self.column.clear(len);
        }
    }

    pub fn dealloc(&mut self) {
        let len = self.entity_count();
        let capacity = self.capacity();

        self.entities = Vec::new();
        self.sparse = SparseHashMap::new();

        unsafe {
            self.column.dealloc(capacity, len);
        }
    }

    #[inline]
    pub fn check_ticks(&mut self, check: CheckTicks) {
        unsafe { self.column.check_ticks(self.entities.len(), check) };
    }

    #[inline]
    pub fn contains_entity(&self, id: EntityId) -> bool {
        self.sparse.contains_key(&id)
    }

    #[cold]
    #[inline(never)]
    fn reserve_one(&mut self) {
        let old_capacity = self.capacity();

        let abort_guard = AbortOnPanic;

        self.entities.reserve(1);

        let new_capacity = self.entities.capacity();

        // Provide redundant space to reduce hash collisions.
        self.sparse.reserve(new_capacity);

        unsafe {
            let new_capacity = NonZeroUsize::new_unchecked(new_capacity);

            if let Some(current_capacity) = NonZeroUsize::new(old_capacity) {
                self.column.realloc(current_capacity, new_capacity);
            } else {
                self.column.alloc(new_capacity);
            }
        }

        ::core::mem::forget(abort_guard);
    }

    pub fn insert(&mut self, id: EntityId, data: OwningPtr<'_>, tick: Tick, caller: DebugLocation) {
        if let Some(index) = self.sparse.get(&id) {
            let index = *index as usize;

            cfg::debug! { assert_eq!(id, self.entities[index]); }

            unsafe {
                self.column.replace_item(index, data, tick, caller);
            }
        } else {
            // SAFETY: `0 < EntityId < u32::MAX`, so `len < u32::MAX`.
            let last_index = self.entities.len();

            if last_index == self.entities.capacity() {
                self.reserve_one();
            }

            self.entities.push(id);

            unsafe {
                self.column.init_item(last_index, data, tick, caller);
                self.sparse.insert(id, last_index as u32);
            }
        }
    }

    #[inline]
    pub fn get_component(&self, id: EntityId) -> Option<Ptr<'_>> {
        self.sparse.get(&id).map(|&index| {
            let index = index as usize;
            cfg::debug! { assert_eq!(id, self.entities[index]); }

            unsafe { self.column.get_data(index) }
        })
    }

    #[inline]
    pub fn get_component_mut(&mut self, id: EntityId) -> Option<PtrMut<'_>> {
        self.sparse.get(&id).map(|&index| {
            let index = index as usize;
            cfg::debug! { assert_eq!(id, self.entities[index]); }

            unsafe { self.column.get_data_mut(index) }
        })
    }

    #[inline]
    pub fn get_with_ticks(&self, id: EntityId) -> Option<(Ptr<'_>, ComponentTickCells<'_>)> {
        let index = *self.sparse.get(&id)? as usize;
        cfg::debug! { assert_eq!(id, self.entities[index]); }

        unsafe {
            Some((
                self.column.get_data(index),
                ComponentTickCells {
                    added: self.column.get_added_tick(index),
                    changed: self.column.get_changed_tick(index),
                    changed_by: self.column.get_changed_by(index),
                },
            ))
        }
    }

    #[inline]
    pub fn get_added_tick(&self, id: EntityId) -> Option<&UnsafeCell<Tick>> {
        let index = *self.sparse.get(&id)? as usize;
        cfg::debug! { assert_eq!(id, self.entities[index]); }

        unsafe { Some(self.column.get_added_tick(index)) }
    }

    #[inline]
    pub fn get_changed_tick(&self, id: EntityId) -> Option<&UnsafeCell<Tick>> {
        let index = *self.sparse.get(&id)? as usize;
        cfg::debug! { assert_eq!(id, self.entities[index]); }

        unsafe { Some(self.column.get_changed_tick(index)) }
    }

    #[inline]
    pub fn get_changed_by(
        &self,
        _id: EntityId,
    ) -> DebugLocation<Option<&UnsafeCell<&'static Location<'static>>>> {
        cfg::debug! {
            if let Some(index) = self.sparse.get(&_id) {
                let index = *index as usize;
                assert_eq!(_id, self.entities[index]);

                let changed_by = unsafe {
                    self.column.get_changed_by(index)
                };

                return DebugLocation::untranspose(Some(changed_by));
            }
        }

        DebugLocation::new(None)
    }

    #[inline]
    pub fn get_component_ticks(&self, id: EntityId) -> Option<ComponentTicks> {
        let index = *self.sparse.get(&id)? as usize;
        cfg::debug! { assert_eq!(id, self.entities[index]); }

        unsafe {
            Some(ComponentTicks {
                added: self.column.copy_added_tick(index),
                changed: self.column.copy_changed_tick(index),
            })
        }
    }

    #[must_use = "The returned pointer must be used to drop the removed component."]
    pub fn remove_and_forget(&mut self, id: EntityId) -> Option<OwningPtr<'_>> {
        use crate::storage::VecCopyRemove;

        self.sparse.remove(&id).map(|index_u32| {
            let index = index_u32 as usize;
            cfg::debug! { assert_eq!(id, self.entities[index]); }

            let last_index = self.entities.len() - 1;

            if index != last_index {
                unsafe {
                    let swapped_id = self
                        .entities
                        .copy_last_and_return_nonoverlapping(index, last_index);

                    *self.sparse.get_mut(&swapped_id).debug_checked_unwrap() = index_u32;
                    self.column.swap_remove_nonoverlapping(index, last_index)
                }
            } else {
                unsafe {
                    self.entities.set_len(last_index);
                    self.column.get_data_mut(last_index).promote()
                }
            }
        })
    }

    pub fn remove(&mut self, id: EntityId) -> bool {
        use crate::storage::VecCopyRemove;

        self.sparse
            .remove(&id)
            .map(|index_u32| {
                let index = index_u32 as usize;
                cfg::debug! { assert_eq!(id, self.entities[index]); }

                let last_index = self.entities.len() - 1;

                if index != last_index {
                    unsafe {
                        let swapped_id = self
                            .entities
                            .copy_last_and_return_nonoverlapping(index, last_index);

                        *self.sparse.get_mut(&swapped_id).debug_checked_unwrap() = index_u32;
                        self.column
                            .swap_remove_and_drop_nonoverlapping(index, last_index);
                    }
                } else {
                    unsafe {
                        self.entities.set_len(last_index);
                        self.column.drop_last(last_index);
                    }
                }
            })
            .is_some()
    }
}

impl Drop for SparseComponent {
    fn drop(&mut self) {
        let len = self.entity_count();
        let current_capacity = self.capacity();
        self.entities.clear();
        unsafe {
            self.column.dealloc(current_capacity, len);
        }
    }
}

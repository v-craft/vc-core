#![allow(clippy::new_without_default, reason = "internal type")]

use core::alloc::Layout;
use core::num::NonZeroUsize;

use vc_ptr::OwningPtr;

use super::{BlobArray, TickArray};

use crate::{
    borrow::{UntypedMut, UntypedRef, UntypedSliceMut, UntypedSliceRef},
    component::{
        ComponentTicksMut, ComponentTicksRef, ComponentTicksSliceMut, ComponentTicksSliceRef,
    },
    tick::{CheckTicks, Tick},
};

// -----------------------------------------------------------------------------
// Column

/// A type-erased container for storing components.
///
/// A `Column` can only store one type of component.
///
/// When used with `Table` storage, each cell represents one component
/// instance, each column represents one component type, and each row
/// represents one entity.
///
/// # Safety
/// - Users need to manage memory manually.
/// - The length and capacity provided by the caller must be correct.
#[derive(Debug)]
pub(super) struct Column {
    data: BlobArray,
    added: TickArray,
    changed: TickArray,
}

// -----------------------------------------------------------------------------
// Basic methods

impl Column {
    /// # Safety
    /// - `item_layout` is correct item layout.
    /// - `drop_fn` is valid for item type.
    #[inline(always)]
    pub const unsafe fn new(
        item_layout: Layout,
        drop_fn: Option<unsafe fn(OwningPtr<'_>)>,
    ) -> Self {
        Self {
            data: unsafe { BlobArray::new(item_layout, drop_fn) },
            added: TickArray::new(),
            changed: TickArray::new(),
        }
    }

    /// # Safety
    /// - `current_capacity == 0` (not yet allocated).
    #[inline]
    pub unsafe fn alloc(&mut self, new_capacity: NonZeroUsize) {
        unsafe {
            self.data.alloc(new_capacity);
            self.added.alloc(new_capacity);
            self.changed.alloc(new_capacity);
        }
    }

    /// # Safety
    /// - `current_capacity` is correct and not zero.
    /// - `current_capacity <= new_capacity`.
    #[inline]
    pub unsafe fn realloc(&mut self, current_capacity: NonZeroUsize, new_capacity: NonZeroUsize) {
        unsafe {
            self.data.realloc(current_capacity, new_capacity);
            self.added.realloc(current_capacity, new_capacity);
            self.changed.realloc(current_capacity, new_capacity);
        }
    }

    /// # Safety
    /// - `current_capacity` and `current_len` is correct.
    #[inline]
    pub unsafe fn dealloc(&mut self, current_capacity: usize, len: usize) {
        unsafe {
            self.added.dealloc(current_capacity);
            self.changed.dealloc(current_capacity);
            self.data.dealloc(current_capacity, len);
        }
    }

    /// Check the ticks of all components and ensure they are valid.
    ///
    /// # Safety
    /// - `len` is correct.
    #[inline]
    pub unsafe fn check_ticks(&mut self, len: usize, check: CheckTicks) {
        let now = check.tick();

        unsafe {
            Tick::slice_check(self.added.as_mut_slice(len), now);
            Tick::slice_check(self.changed.as_mut_slice(len), now);
        }
    }

    /// # Safety
    /// - `index < current_capacity`.
    /// - `index == current_len`
    /// - `data` point to a valid data.
    #[inline]
    pub unsafe fn init_item(&mut self, index: usize, data: OwningPtr<'_>, tick: Tick) {
        unsafe {
            self.data.init_item(index, data);
            self.added.init_item(index, tick);
            self.changed.init_item(index, tick);
        }
    }

    /// # Safety
    /// - `current_len > 0`
    /// - `index == current_len - 1`
    #[inline]
    #[must_use = "The returned pointer should be used to drop the removed element"]
    pub unsafe fn remove_last(&mut self, last_index: usize) -> OwningPtr<'_> {
        unsafe { self.data.remove_last(last_index) }
    }

    /// # Safety
    /// - `current_len > 0`
    /// - `index == current_len - 1`
    /// - If the data needs to be "dropped", the caller needs to handle
    ///   the returned pointer correctly.
    #[inline]
    pub unsafe fn remove_and_drop_last(&mut self, last_index: usize) {
        unsafe {
            self.data.remove_and_drop_last(last_index);
        }
    }

    /// # Safety
    /// - `current_len >= 2`
    /// - `index < last_index`
    /// - `last_index == current_len - 1`
    /// - If the data needs to be "dropped", the caller needs to handle
    ///   the returned pointer correctly.
    #[cfg_attr(not(any(debug_assertions, feature = "debug")), inline)]
    #[must_use = "The returned pointer should be used to drop the removed element"]
    pub unsafe fn swap_remove_not_last(
        &mut self,
        index: usize,
        last_index: usize,
    ) -> OwningPtr<'_> {
        unsafe {
            let data = self.data.swap_remove_not_last(index, last_index);
            self.added.copy_remove_not_last(index, last_index);
            self.changed.copy_remove_not_last(index, last_index);

            data
        }
    }

    /// # Safety
    /// - `current_len >= 2`
    /// - `index < last_index`
    /// - `last_index == current_len - 1`
    #[cfg_attr(not(any(debug_assertions, feature = "debug")), inline)]
    pub unsafe fn swap_remove_and_drop_not_last(&mut self, index: usize, last_index: usize) {
        unsafe {
            self.data.swap_remove_and_drop_not_last(index, last_index);
            self.added.copy_remove_not_last(index, last_index);
            self.changed.copy_remove_not_last(index, last_index);
        }
    }

    /// Move the last element from other `Component` to the current `Column`.
    ///
    /// # Safety
    /// - `other_last == other_len - 1`
    /// - `index == self.len`
    /// - `self.len < self.capacity`
    #[cfg_attr(not(any(debug_assertions, feature = "debug")), inline)]
    pub unsafe fn init_item_from_last(
        &mut self,
        other: &mut Column,
        other_last: usize,
        index: usize,
    ) {
        debug_assert_eq!(self.data.layout(), other.data.layout());

        unsafe {
            let src_val = other.data.remove_last(other_last);
            self.data.init_item(index, src_val);

            let added_tick = other.added.remove_last(other_last);
            self.added.init_item(index, added_tick);

            let changed_tick = other.changed.remove_last(other_last);
            self.changed.init_item(index, changed_tick);
        }
    }

    /// Move a `Component` from other `Column` to the current `Column`.
    ///
    /// - `dst` : the empty position in the current `Column`.
    /// - `src` : the index of the `Component` to be moved in the other `Component`.
    /// - `other_last_index` : the last index of `other`, used for `swap_remove`.
    ///
    /// # Safety
    /// - `src < other_last_index`
    /// - `other_last_index == other_len - 1`
    /// - `dst == self.len`
    /// - `self.len < self.capacity`
    #[cfg_attr(not(any(debug_assertions, feature = "debug")), inline)]
    pub unsafe fn init_item_from_not_last(
        &mut self,
        other: &mut Column,
        other_last: usize,
        src: usize,
        dst: usize,
    ) {
        debug_assert_eq!(self.data.layout(), other.data.layout());

        unsafe {
            let src_val = other.data.swap_remove_not_last(src, other_last);
            self.data.init_item(dst, src_val);

            let added_tick = other.added.swap_remove_not_last(src, other_last);
            self.added.init_item(dst, added_tick);

            let changed_tick = other.changed.swap_remove_not_last(src, other_last);
            self.changed.init_item(dst, changed_tick);
        }
    }

    #[inline]
    pub unsafe fn get_ref(&self, index: usize, last_run: Tick, this_run: Tick) -> UntypedRef<'_> {
        unsafe {
            UntypedRef {
                value: self.data.get(index),
                ticks: ComponentTicksRef {
                    added: self.added.get(index),
                    changed: self.changed.get(index),
                    last_run,
                    this_run,
                },
            }
        }
    }

    #[inline]
    pub unsafe fn get_mut(
        &mut self,
        index: usize,
        last_run: Tick,
        this_run: Tick,
    ) -> UntypedMut<'_> {
        unsafe {
            UntypedMut {
                value: self.data.get_mut(index),
                ticks: ComponentTicksMut {
                    added: self.added.get_mut(index),
                    changed: self.changed.get_mut(index),
                    last_run,
                    this_run,
                },
            }
        }
    }

    #[inline]
    pub unsafe fn get_slice_ref(
        &self,
        len: usize,
        last_run: Tick,
        this_run: Tick,
    ) -> UntypedSliceRef<'_> {
        unsafe {
            UntypedSliceRef {
                value: self.data.get(0),
                ticks: ComponentTicksSliceRef {
                    added: self.added.as_slice(len),
                    changed: self.changed.as_slice(len),
                    last_run,
                    this_run,
                },
            }
        }
    }

    #[inline]
    pub unsafe fn get_slice_mut(
        &mut self,
        len: usize,
        last_run: Tick,
        this_run: Tick,
    ) -> UntypedSliceMut<'_> {
        unsafe {
            UntypedSliceMut {
                value: self.data.get_mut(0),
                ticks: ComponentTicksSliceMut {
                    added: self.added.as_mut_slice(len),
                    changed: self.changed.as_mut_slice(len),
                    last_run,
                    this_run,
                },
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Optional methods

impl Column {}

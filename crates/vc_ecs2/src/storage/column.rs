#![allow(clippy::new_without_default, reason = "internal type")]
#![allow(unused_variables, reason = "`DebugLocation` is unused in release mod.")]

use core::alloc::Layout;
use core::num::NonZeroUsize;
use core::panic::Location;

use vc_ptr::OwningPtr;

use super::{BlobArray, ThinArray};

use crate::cfg;
use crate::tick::{CheckTicks, Tick};
use crate::utils::DebugLocation;

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
    added_ticks: ThinArray<Tick>,
    changed_ticks: ThinArray<Tick>,
    changed_by: DebugLocation<ThinArray<&'static Location<'static>>>,
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
            added_ticks: ThinArray::new(),
            changed_ticks: ThinArray::new(),
            changed_by: DebugLocation::new(ThinArray::new()),
        }
    }

    /// # Safety
    /// - `current_capacity == 0` (not yet allocated).
    /// - `new_capacity * layout.size <= Isize::MAX`.
    #[inline]
    pub unsafe fn alloc(&mut self, new_capacity: NonZeroUsize) {
        unsafe {
            self.data.alloc(new_capacity);
            self.added_ticks.alloc(new_capacity);
            self.changed_ticks.alloc(new_capacity);
            cfg::debug! {
                self.changed_by.as_mut().map(|cb| cb.alloc(new_capacity));
            }
        }
    }

    /// # Safety
    /// - `current_capacity` is correct and not zero.
    /// - `current_capacity <= new_capacity`.
    /// - `new_capacity * layout.size <= Isize::MAX`.
    #[inline]
    pub unsafe fn realloc(&mut self, current_capacity: NonZeroUsize, new_capacity: NonZeroUsize) {
        unsafe {
            self.data.realloc(current_capacity, new_capacity);
            self.added_ticks.realloc(current_capacity, new_capacity);
            self.changed_ticks.realloc(current_capacity, new_capacity);
            cfg::debug! {
                self.changed_by.as_mut().map(|cb| cb.realloc(current_capacity, new_capacity));
            }
        }
    }

    /// # Safety
    /// - `current_capacity` and `current_len` is correct.
    #[inline]
    pub unsafe fn dealloc(&mut self, current_capacity: usize, len: usize) {
        unsafe {
            self.added_ticks.dealloc(current_capacity);
            self.changed_ticks.dealloc(current_capacity);
            self.data.dealloc(current_capacity, len);
            cfg::debug! {
                self.changed_by.as_mut().map(|cb| cb.dealloc(current_capacity));
            }
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
            Tick::slice_check(self.added_ticks.as_mut_slice(len), now);
            Tick::slice_check(self.changed_ticks.as_mut_slice(len), now);
        }
    }

    /// # Safety
    /// - `index < current_capacity`.
    /// - `index == current_len`
    /// - `data` point to a valid data.
    #[inline]
    pub unsafe fn init_item(
        &mut self,
        index: usize,
        data: OwningPtr<'_>,
        tick: Tick,
        caller: DebugLocation,
    ) {
        unsafe {
            self.data.init_item(index, data);
            self.added_ticks.init_item(index, tick);
            self.changed_ticks.init_item(index, tick);

            cfg::debug! {
                self.changed_by.as_mut()
                    .map(|cb| cb.get_mut(index))
                    .assign(caller);
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Optional methods

impl Column {
    // /// # Safety
    // /// - `current_len` is correct.
    // #[inline]
    // pub unsafe fn clear(&mut self, len: usize) {
    //     unsafe {
    //         self.data.clear(len);
    //     }
    // }

    // /// # Safety
    // /// - `len < current_capacity`.
    // #[inline]
    // #[cfg_attr(any(debug_assertions, feature = "debug"), track_caller)]
    // pub unsafe fn reset_item(&mut self, index: usize) {
    //     unsafe {
    //         self.added_ticks
    //             .init_item(index, Tick::new(0));
    //         self.changed_ticks
    //             .init_item(index, Tick::new(0));
    //         cfg::debug! {
    //             self.changed_by.as_mut().map(move |cb|
    //                 cb.init_item(index, Location::caller())
    //             );
    //         }
    //     }
    // }

    // #[inline(always)]
    // pub fn drop_fn(&self) -> Option<unsafe fn(OwningPtr<'_>)> {
    //     self.data.drop_fn()
    // }

    // /// # Safety
    // /// - `index < current_len`.
    // #[inline(always)]
    // pub unsafe fn get_data_ptr(&self, index: usize) -> Ptr<'_> {
    //     unsafe { self.data.get(index) }
    // }

    // /// # Safety
    // /// - `index < current_len`.
    // #[inline(always)]
    // pub unsafe fn get_data_ptr_mut(&mut self, index: usize) -> PtrMut<'_> {
    //     unsafe { self.data.get_mut(index) }
    // }

    // /// # Safety
    // /// - `index < current_len`.
    // #[inline(always)]
    // pub unsafe fn get_added_tick(&self, index: usize) -> Tick {
    //     unsafe { self.added_ticks.get(index) }
    // }

    // /// # Safety
    // /// - `index < current_len`.
    // #[inline(always)]
    // pub unsafe fn get_added_tick_cell(&self, index: usize) -> &UnsafeCell<Tick> {
    //     unsafe { self.added_ticks.get_cell(index) }
    // }

    // /// # Safety
    // /// - `index < current_len`.
    // #[inline(always)]
    // pub unsafe fn get_changed_tick(&self, index: usize) -> Tick {
    //     unsafe { self.changed_ticks.get(index) }
    // }

    // /// # Safety
    // /// - `index < current_len`.
    // #[inline(always)]
    // pub unsafe fn get_changed_tick_cell(&self, index: usize) -> &UnsafeCell<Tick> {
    //     unsafe { self.changed_ticks.get_cell(index) }
    // }

    // /// # Safety
    // /// - `index < current_len`.
    // #[inline(always)]
    // pub unsafe fn get_changed_by(&self, index: usize) -> DebugLocation {
    //     unsafe { self.changed_by.as_ref().map(|cb| cb.get(index)) }
    // }

    // /// # Safety
    // /// - `index < current_len`.
    // #[inline(always)]
    // pub unsafe fn get_changed_by_cell(
    //     &self,
    //     index: usize,
    // ) -> DebugLocation<&UnsafeCell<&'static Location<'static>>> {
    //     unsafe { self.changed_by.as_ref().map(|cb| cb.get_cell(index)) }
    // }

    // /// # Safety
    // /// - `len <= current_len`.
    // /// - `T` is valid item type.
    // #[inline(always)]
    // pub unsafe fn get_data_cell_slice<T>(&self, len: usize) -> &[UnsafeCell<T>] {
    //     unsafe { self.data.as_slice(len) }
    // }

    // /// # Safety
    // /// - `len <= current_len`.
    // #[inline(always)]
    // pub unsafe fn get_added_ticks_cell_slice(&self, len: usize) -> &[UnsafeCell<Tick>] {
    //     unsafe { self.added_ticks.as_cell_slice(len) }
    // }

    // /// # Safety
    // /// - `len <= current_len`.
    // #[inline(always)]
    // pub unsafe fn get_changed_ticks_cell_slice(&self, len: usize) -> &[UnsafeCell<Tick>] {
    //     unsafe { self.changed_ticks.as_cell_slice(len) }
    // }

    // /// # Safety
    // /// - `len <= current_len`.
    // #[inline(always)]
    // pub unsafe fn get_changed_by_cell_slice(
    //     &self,
    //     len: usize,
    // ) -> DebugLocation<&[UnsafeCell<&'static Location<'static>>]> {
    //     unsafe { self.changed_by.as_ref().map(|cb| cb.as_cell_slice(len)) }
    // }

    // /// # Safety
    // /// - `index < current_capacity`.
    // /// - `index < current_len`
    // /// - `data` point to a valid data.
    // #[inline]
    // pub unsafe fn replace_item(
    //     &mut self,
    //     index: usize,
    //     data: OwningPtr<'_>,
    //     change_tick: Tick,
    //     caller: DebugLocation,
    // ) {
    //     unsafe {
    //         self.data.replace_item(index, data);
    //         self.changed_ticks
    //             .init_item(index, change_tick);

    //         cfg::debug! {
    //             self.changed_by.as_mut()
    //                 .map(|cb| cb.get_mut(index))
    //                 .assign(caller);
    //         }
    //     }
    // }

    // /// # Safety
    // /// - `current_len > 0`
    // /// - `index == current_len - 1`
    // #[inline]
    // #[must_use = "The returned pointer should be used to drop the removed element"]
    // pub unsafe fn remove_last(&mut self, last_index: usize) -> OwningPtr<'_> {
    //     unsafe { self.data.remove_last(last_index) }
    // }

    // /// # Safety
    // /// - `current_len > 0`
    // /// - `index == current_len - 1`
    // /// - If the data needs to be "dropped", the caller needs to handle
    // ///   the returned pointer correctly.
    // #[inline]
    // pub unsafe fn drop_last(&mut self, last_index: usize) {
    //     unsafe {
    //         self.data.drop_last(last_index);
    //     }
    // }

    // /// # Safety
    // /// - `current_len >= 2`
    // /// - `index < last_index`
    // /// - `last_index == current_len - 1`
    // /// - If the data needs to be "dropped", the caller needs to handle
    // ///   the returned pointer correctly.
    // #[cfg_attr(not(any(debug_assertions, feature = "debug")), inline)]
    // #[must_use = "The returned pointer should be used to drop the removed element"]
    // pub unsafe fn swap_remove_nonoverlapping(
    //     &mut self,
    //     index: usize,
    //     last_index: usize,
    // ) -> OwningPtr<'_> {
    //     unsafe {
    //         let data = self.data.swap_remove_nonoverlapping(index, last_index);
    //         self.added_ticks
    //             .copy_remove_nonoverlapping(index, last_index);
    //         self.changed_ticks
    //             .copy_remove_nonoverlapping(index, last_index);

    //         cfg::debug! {
    //             // Use `{ ..; }` to eliminate return values and reduce compilation workload.
    //             self.changed_by.as_mut().map(|cb| {
    //                 cb.copy_remove_nonoverlapping(index, last_index);
    //             });
    //         }

    //         data
    //     }
    // }

    // /// # Safety
    // /// - `current_len >= 2`
    // /// - `index < last_index`
    // /// - `last_index == current_len - 1`
    // #[cfg_attr(not(any(debug_assertions, feature = "debug")), inline)]
    // pub unsafe fn swap_remove_and_drop_nonoverlapping(&mut self, index: usize, last_index: usize) {
    //     unsafe {
    //         self.data
    //             .swap_remove_and_drop_nonoverlapping(index, last_index);
    //         self.added_ticks
    //             .copy_remove_nonoverlapping(index, last_index);
    //         self.changed_ticks
    //             .copy_remove_nonoverlapping(index, last_index);

    //         cfg::debug! {
    //             // Use `{ ..; }` to eliminate return values and reduce compilation workload.
    //             self.changed_by.as_mut().map(|cb| {
    //                 cb.copy_remove_nonoverlapping(index, last_index);
    //             });
    //         }
    //     }
    // }

    // /// Move the last element from other `Component` to the current `Column`.
    // ///
    // /// # Safety
    // /// - `other_last == other_len - 1`
    // /// - `index == self.len`
    // /// - `self.len < self.capacity`
    // #[cfg_attr(not(any(debug_assertions, feature = "debug")), inline)]
    // pub unsafe fn init_item_from_last(
    //     &mut self,
    //     other: &mut Column,
    //     other_last: usize,
    //     index: usize,
    // ) {
    //     debug_assert_ne!(self.data.layout(), other.data.layout());

    //     unsafe {
    //         let src_val = other.data.remove_last(other_last);
    //         self.data.init_item(index, src_val);

    //         let added_tick = other.added_ticks.remove_last(other_last);
    //         self.added_ticks.init_item(index, added_tick);

    //         let changed_tick = other.changed_ticks.remove_last(other_last);
    //         self.changed_ticks.init_item(index, changed_tick);

    //         cfg::debug! {
    //             self.changed_by.as_mut().zip(other.changed_by.as_mut()).map(|(scb, ocb)| {
    //                 let changed_by = ocb.remove_last(other_last);
    //                 scb.init_item(index, changed_by);
    //             });
    //         }
    //     }
    // }

    // /// Move a `Component` from other `Column` to the current `Column`.
    // ///
    // /// - `dst` : the empty position in the current `Column`.
    // /// - `src` : the index of the `Component` to be moved in the other `Component`.
    // /// - `other_last_index` : the last index of `other`, used for `swap_remove`.
    // ///
    // /// # Safety
    // /// - `src < other_last_index`
    // /// - `other_last_index == other_len - 1`
    // /// - `dst == self.len`
    // /// - `self.len < self.capacity`
    // #[cfg_attr(not(any(debug_assertions, feature = "debug")), inline)]
    // pub unsafe fn init_item_from_nonoverlapping(
    //     &mut self,
    //     other: &mut Column,
    //     other_last_index: usize,
    //     src: usize,
    //     dst: usize,
    // ) {
    //     debug_assert_ne!(self.data.layout(), other.data.layout());

    //     unsafe {
    //         let src_val = other.data.swap_remove_nonoverlapping(src, other_last_index);
    //         self.data.init_item(dst, src_val);

    //         let added_tick = other
    //             .added_ticks
    //             .swap_remove_nonoverlapping(src, other_last_index);
    //         self.added_ticks.init_item(dst, added_tick);

    //         let changed_tick = other
    //             .changed_ticks
    //             .swap_remove_nonoverlapping(src, other_last_index);
    //         self.changed_ticks.init_item(dst, changed_tick);

    //         cfg::debug! {
    //             self.changed_by.as_mut().zip(other.changed_by.as_mut()).map(|(scb, ocb)| {
    //                 let changed_by = ocb.swap_remove_nonoverlapping(src, other_last_index);
    //                 scb.init_item(dst, changed_by);
    //             });
    //         }
    //     }
    // }
}

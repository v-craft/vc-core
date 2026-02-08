use core::alloc::Layout;
use core::cell::UnsafeCell;
use core::num::NonZeroUsize;
use core::panic::Location;

use vc_ptr::{OwningPtr, Ptr, PtrMut};

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
pub(crate) struct Column {
    data: BlobArray,
    added_ticks: ThinArray<UnsafeCell<Tick>>,
    changed_ticks: ThinArray<UnsafeCell<Tick>>,
    changed_by: DebugLocation<ThinArray<UnsafeCell<&'static Location<'static>>>>,
}

impl Column {
    /// # Safety
    /// - `item_layout` is correct item layout.
    /// - `drop_fn` is valid for item type.
    #[inline(always)]
    pub const unsafe fn empty(
        item_layout: Layout,
        drop_fn: Option<unsafe fn(OwningPtr<'_>)>,
    ) -> Self {
        Self {
            data: unsafe { BlobArray::empty(item_layout, drop_fn) },
            added_ticks: ThinArray::empty(),
            changed_ticks: ThinArray::empty(),
            changed_by: DebugLocation::new(ThinArray::empty()),
        }
    }

    /// # Safety
    /// - `item_layout` is correct item layout.
    /// - `drop_fn` is valid for item type.
    /// - `capacity * item_layout.size <= isize::MAX`
    pub unsafe fn with_capacity(
        item_layout: Layout,
        drop_fn: Option<unsafe fn(OwningPtr<'_>)>,
        capacity: usize,
    ) -> Self {
        Self {
            data: unsafe { BlobArray::with_capacity(item_layout, drop_fn, capacity) },
            added_ticks: ThinArray::with_capacity(capacity),
            changed_ticks: ThinArray::with_capacity(capacity),
            changed_by: DebugLocation::new_with(|| ThinArray::with_capacity(capacity)),
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

    /// # Safety
    /// - `current_len` is correct.
    #[inline]
    pub unsafe fn clear(&mut self, len: usize) {
        unsafe {
            self.data.clear(len);
        }
    }

    #[inline(always)]
    pub fn drop_fn(&self) -> Option<unsafe fn(OwningPtr<'_>)> {
        self.data.drop_fn()
    }

    /// # Safety
    /// - `index < current_len`.
    #[inline(always)]
    pub unsafe fn get_data(&self, index: usize) -> Ptr<'_> {
        unsafe { self.data.get_item(index) }
    }

    /// # Safety
    /// - `index < current_len`.
    #[inline(always)]
    pub unsafe fn get_data_mut(&mut self, index: usize) -> PtrMut<'_> {
        unsafe { self.data.get_item_mut(index) }
    }

    /// # Safety
    /// - `index < current_len`.
    #[inline(always)]
    pub unsafe fn get_added_tick(&self, index: usize) -> &UnsafeCell<Tick> {
        unsafe { self.added_ticks.get_item(index) }
    }

    /// # Safety
    /// - `index < current_len`.
    #[inline(always)]
    pub unsafe fn copy_added_tick(&self, index: usize) -> Tick {
        unsafe { self.added_ticks.copy_item(index) }
    }

    /// # Safety
    /// - `index < current_len`.
    #[inline(always)]
    pub unsafe fn get_changed_tick(&self, index: usize) -> &UnsafeCell<Tick> {
        unsafe { self.changed_ticks.get_item(index) }
    }

    /// # Safety
    /// - `index < current_len`.
    #[inline(always)]
    pub unsafe fn copy_changed_tick(&self, index: usize) -> Tick {
        unsafe { self.changed_ticks.copy_item(index) }
    }

    /// # Safety
    /// - `index < current_len`.
    #[inline(always)]
    pub unsafe fn get_changed_by(
        &self,
        index: usize,
    ) -> DebugLocation<&UnsafeCell<&'static Location<'static>>> {
        unsafe { self.changed_by.as_ref().map(|cb| cb.get_item(index)) }
    }

    /// # Safety
    /// - `len <= current_len`.
    /// - `T` is valid item type.
    #[inline(always)]
    pub unsafe fn get_data_slice<T>(&self, len: usize) -> &[UnsafeCell<T>] {
        unsafe { self.data.as_slice(len) }
    }

    /// # Safety
    /// - `len <= current_len`.
    #[inline(always)]
    pub unsafe fn get_added_ticks_slice(&self, len: usize) -> &[UnsafeCell<Tick>] {
        unsafe { self.added_ticks.as_slice(len) }
    }

    /// # Safety
    /// - `len <= current_len`.
    #[inline(always)]
    pub unsafe fn get_changed_ticks_slice(&self, len: usize) -> &[UnsafeCell<Tick>] {
        unsafe { self.changed_ticks.as_slice(len) }
    }

    /// # Safety
    /// - `len <= current_len`.
    #[inline(always)]
    pub unsafe fn get_changed_by_slice(
        &self,
        len: usize,
    ) -> DebugLocation<&[UnsafeCell<&'static Location<'static>>]> {
        unsafe { self.changed_by.as_ref().map(|cb| cb.as_slice(len)) }
    }

    /// # Safety
    /// - `len < current_capacity`.
    #[inline]
    #[cfg_attr(any(debug_assertions, feature = "debug"), track_caller)]
    pub unsafe fn reset_item(&mut self, index: usize) {
        unsafe {
            self.added_ticks
                .init_item(index, UnsafeCell::new(Tick::new(0)));
            self.changed_ticks
                .init_item(index, UnsafeCell::new(Tick::new(0)));
            cfg::debug! {
                let caller = Location::caller();
                self.changed_by.as_mut().map(move |cb|
                    cb.init_item(index, UnsafeCell::new(caller))
                );
            }
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
            self.added_ticks.init_item(index, UnsafeCell::new(tick));
            self.changed_ticks.init_item(index, UnsafeCell::new(tick));

            cfg::debug! {
                self.changed_by.as_mut()
                    .map(|cb| cb.get_item_mut(index).get_mut())
                    .assign(caller);
            }
        }
    }

    /// # Safety
    /// - `index < current_capacity`.
    /// - `index < current_len`
    /// - `data` point to a valid data.
    #[inline]
    pub unsafe fn replace_item(
        &mut self,
        index: usize,
        data: OwningPtr<'_>,
        change_tick: Tick,
        caller: DebugLocation,
    ) {
        unsafe {
            self.data.replace_item(index, data);
            self.changed_ticks
                .init_item(index, UnsafeCell::new(change_tick));

            cfg::debug! {
                self.changed_by.as_mut()
                    .map(|cb| cb.get_item_mut(index).get_mut())
                    .assign(caller);
            }
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
    pub unsafe fn drop_last(&mut self, last_index: usize) {
        unsafe {
            self.data.drop_last(last_index);
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
    pub unsafe fn swap_remove_nonoverlapping(
        &mut self,
        index: usize,
        last_index: usize,
    ) -> OwningPtr<'_> {
        unsafe {
            let data = self.data.swap_remove_nonoverlapping(index, last_index);
            self.added_ticks
                .copy_remove_nonoverlapping(index, last_index);
            self.changed_ticks
                .copy_remove_nonoverlapping(index, last_index);

            cfg::debug! {
                // Use `{ ..; }` to eliminate return values and reduce compilation workload.
                self.changed_by.as_mut().map(|cb| {
                    cb.copy_remove_nonoverlapping(index, last_index);
                });
            }

            data
        }
    }

    /// # Safety
    /// - `current_len >= 2`
    /// - `index < last_index`
    /// - `last_index == current_len - 1`
    #[cfg_attr(not(any(debug_assertions, feature = "debug")), inline)]
    pub unsafe fn swap_remove_and_drop_nonoverlapping(&mut self, index: usize, last_index: usize) {
        unsafe {
            self.data
                .swap_remove_and_drop_nonoverlapping(index, last_index);
            self.added_ticks
                .copy_remove_nonoverlapping(index, last_index);
            self.changed_ticks
                .copy_remove_nonoverlapping(index, last_index);

            cfg::debug! {
                // Use `{ ..; }` to eliminate return values and reduce compilation workload.
                self.changed_by.as_mut().map(|cb| {
                    cb.copy_remove_nonoverlapping(index, last_index);
                });
            }
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
        cfg::debug! {
            assert_eq!(self.data.layout(), other.data.layout());
        }

        unsafe {
            let src_val = other.data.remove_last(other_last);
            self.data.init_item(index, src_val);

            let added_tick = other.added_ticks.remove_last(other_last);
            self.added_ticks.init_item(index, added_tick);

            let changed_tick = other.changed_ticks.remove_last(other_last);
            self.changed_ticks.init_item(index, changed_tick);

            cfg::debug! {
                self.changed_by.as_mut().zip(other.changed_by.as_mut()).map(|(scb, ocb)| {
                    let changed_by = ocb.remove_last(other_last);
                    scb.init_item(index, changed_by);
                });
            }
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
    pub unsafe fn init_item_from_nonoverlapping(
        &mut self,
        other: &mut Column,
        other_last_index: usize,
        src: usize,
        dst: usize,
    ) {
        cfg::debug! {
            assert_eq!(self.data.layout(), other.data.layout());
        }

        unsafe {
            let src_val = other.data.swap_remove_nonoverlapping(src, other_last_index);
            self.data.init_item(dst, src_val);

            let added_tick = other
                .added_ticks
                .swap_remove_nonoverlapping(src, other_last_index);
            self.added_ticks.init_item(dst, added_tick);

            let changed_tick = other
                .changed_ticks
                .swap_remove_nonoverlapping(src, other_last_index);
            self.changed_ticks.init_item(dst, changed_tick);

            cfg::debug! {
                self.changed_by.as_mut().zip(other.changed_by.as_mut()).map(|(scb, ocb)| {
                    let changed_by = ocb.swap_remove_nonoverlapping(src, other_last_index);
                    scb.init_item(dst, changed_by);
                });
            }
        }
    }

    /// Check the ticks of all components and ensure they are valid.
    ///
    /// # Safety
    /// - `len` is correct.
    #[inline]
    pub unsafe fn check_ticks(&mut self, len: usize, check: CheckTicks) {
        for i in 0..len {
            unsafe {
                self.added_ticks
                    .get_item_mut(i)
                    .get_mut()
                    .check_age(check.tick());

                self.changed_ticks
                    .get_item_mut(i)
                    .get_mut()
                    .check_age(check.tick());
            }
        }
    }
}

#![allow(clippy::new_without_default, reason = "internal type")]
#![allow(unused, reason = "todo")]

use alloc::alloc as malloc;
use core::alloc::Layout;
use core::cell::UnsafeCell;
use core::num::NonZeroUsize;
use core::ptr::NonNull;
use core::{ptr, slice};

// -----------------------------------------------------------------------------
// ThinArray

/// A thin `Vec` without length and capacity infomation.
///
/// The capacity and length will be stored by the upper-level container.
///
/// This is an internal type with a highly customized API.
/// Some functions have advanced semantics and are meant for specific scenarios only.
///
/// # Safety
/// - `T` must not need drop.
/// - Users need to manage memory manually.
/// - The length and capacity provided by the caller must be correct.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub(super) struct ThinArray<T: Copy> {
    data: NonNull<UnsafeCell<T>>,
}

impl<T: Copy> ThinArray<T> {
    const _STATIC_ASSERT_: () = const {
        assert!(size_of::<T>() == size_of::<UnsafeCell<T>>());
        assert!(align_of::<T>() == align_of::<UnsafeCell<T>>());
    };

    const IS_ZST: bool = size_of::<T>() == 0;

    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            data: NonNull::dangling(),
        }
    }

    /// # Safety
    /// - `current_capacity == 0` (not yet allocated).
    /// - `new_capacity * size_of::<T>() <= Isize::MAX`.
    pub unsafe fn alloc(&mut self, capacity: NonZeroUsize) {
        if !Self::IS_ZST {
            let new_layout = Layout::array::<T>(capacity.get()).unwrap();

            self.data = NonNull::new(unsafe { malloc::alloc(new_layout) })
                .unwrap_or_else(|| malloc::handle_alloc_error(new_layout))
                .cast();
        }
    }

    /// # Safety
    /// - `current_capacity` is correct and not zero.
    /// - `current_capacity <= new_capacity`.
    /// - `new_capacity * size_of::<T>() <= Isize::MAX`.
    pub unsafe fn realloc(&mut self, current_capacity: NonZeroUsize, new_capacity: NonZeroUsize) {
        if !Self::IS_ZST {
            let new_layout = Layout::array::<T>(new_capacity.get()).unwrap();

            self.data = NonNull::new(unsafe {
                malloc::realloc(
                    self.data.as_ptr().cast(),
                    Layout::array::<T>(current_capacity.get()).unwrap_unchecked(),
                    new_layout.size(),
                )
            })
            .unwrap_or_else(|| malloc::handle_alloc_error(new_layout))
            .cast();
        }
    }

    /// # Safety
    /// - `current_capacity` is correct.
    pub unsafe fn dealloc(&mut self, current_capacity: usize) {
        if !Self::IS_ZST && current_capacity != 0 {
            unsafe {
                let layout = Layout::array::<T>(current_capacity).unwrap_unchecked();
                malloc::dealloc(self.data.as_ptr().cast(), layout);
            }
        }
    }

    /// Copy a inner element.
    ///
    /// # Safety
    /// - `index < current_len`
    #[inline(always)]
    pub const unsafe fn get(&self, index: usize) -> T {
        let base_ptr = self.data.as_ptr() as *mut T;
        unsafe { base_ptr.add(index).read() }
    }

    /// # Safety
    /// - `index < current_len`.
    #[inline(always)]
    pub const unsafe fn get_mut(&self, index: usize) -> &mut T {
        let base_ptr = self.data.as_ptr() as *mut T;
        unsafe { &mut *base_ptr.add(index) }
    }

    /// # Safety
    /// - `index < current_len`.
    #[inline(always)]
    pub const unsafe fn get_cell(&self, index: usize) -> &UnsafeCell<T> {
        unsafe { &*self.data.as_ptr().add(index) }
    }

    /// # Safety
    /// - `slice_len <= current_len`.
    #[inline(always)]
    pub const unsafe fn as_cell_slice(&self, slice_len: usize) -> &[UnsafeCell<T>] {
        unsafe { slice::from_raw_parts(self.data.as_ptr(), slice_len) }
    }

    /// # Safety
    /// - `slice_len <= current_len`.
    #[inline(always)]
    pub const unsafe fn as_mut_slice(&self, slice_len: usize) -> &mut [T] {
        let base_ptr = self.data.as_ptr() as *mut T;
        unsafe { slice::from_raw_parts_mut(base_ptr, slice_len) }
    }

    /// # Safety
    /// - `index == current_len`
    /// - `current_len < current_capacity`
    #[inline(always)]
    pub const unsafe fn init_item(&mut self, index: usize, value: T) {
        let base_ptr = self.data.as_ptr() as *mut T;
        unsafe {
            base_ptr.add(index).write(value);
        }
    }

    /// # Safety
    /// - `current_len > 0`
    /// - `last_index == current_len - 1`
    #[inline(always)]
    pub const unsafe fn remove_last(&mut self, last_index: usize) -> T {
        let base_ptr = self.data.as_ptr() as *mut T;
        unsafe { base_ptr.add(last_index).read() }
    }

    /// # Safety
    /// - `index < last_index`
    /// - `current_len > 0`
    /// - `last_index == current_len - 1`
    #[inline(always)]
    pub const unsafe fn swap_remove_nonoverlapping(
        &mut self,
        index: usize,
        last_index: usize,
    ) -> T {
        let base_ptr = self.data.as_ptr() as *mut T;

        unsafe {
            let last = base_ptr.add(last_index);
            let removal = base_ptr.add(index);

            let value = removal.read();
            ptr::copy_nonoverlapping(last, removal, 1);

            value
        }
    }

    /// # Safety
    /// - `index < last_index`
    /// - `current_len > 0`
    /// - `last_index == current_len - 1`
    #[inline(always)]
    pub const unsafe fn copy_remove_nonoverlapping(&mut self, index: usize, last_index: usize) {
        let base_ptr = self.data.as_ptr() as *mut T;

        unsafe {
            let last = base_ptr.add(last_index);
            let removal = base_ptr.add(index);

            ptr::copy_nonoverlapping(last, removal, 1);
        }
    }
}

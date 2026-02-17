#![allow(clippy::new_without_default, reason = "internal type")]

use alloc::alloc as malloc;
use core::alloc::Layout;
use core::num::NonZeroUsize;
use core::ptr::NonNull;
use core::{ptr, slice};

use crate::tick::Tick;

// -----------------------------------------------------------------------------
// TickArray

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub(super) struct TickArray {
    data: NonNull<Tick>,
}

impl TickArray {
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            data: NonNull::dangling(),
        }
    }

    /// # Safety
    /// - `current_capacity == 0` (not yet allocated).
    pub unsafe fn alloc(&mut self, capacity: NonZeroUsize) {
        let new_layout = Layout::array::<Tick>(capacity.get()).unwrap();

        self.data = NonNull::new(unsafe { malloc::alloc(new_layout) })
            .unwrap_or_else(|| malloc::handle_alloc_error(new_layout))
            .cast();
    }

    /// # Safety
    /// - `current_capacity` is correct and not zero.
    /// - `current_capacity <= new_capacity`.
    pub unsafe fn realloc(&mut self, current_capacity: NonZeroUsize, new_capacity: NonZeroUsize) {
        let new_layout = Layout::array::<Tick>(new_capacity.get()).unwrap();

        self.data = NonNull::new(unsafe {
            malloc::realloc(
                self.data.as_ptr().cast(),
                Layout::array::<Tick>(current_capacity.get()).unwrap_unchecked(),
                new_layout.size(),
            )
        })
        .unwrap_or_else(|| malloc::handle_alloc_error(new_layout))
        .cast();
    }

    /// # Safety
    /// - `current_capacity` is correct.
    pub unsafe fn dealloc(&mut self, current_capacity: usize) {
        if current_capacity != 0 {
            unsafe {
                let layout = Layout::array::<Tick>(current_capacity).unwrap_unchecked();
                malloc::dealloc(self.data.as_ptr().cast(), layout);
            }
        }
    }

    /// # Safety
    /// - `index == current_len`
    /// - `current_len < current_capacity`
    #[inline(always)]
    pub const unsafe fn init_item(&mut self, index: usize, value: Tick) {
        unsafe {
            ptr::write(self.data.as_ptr().add(index), value);
        }
    }

    /// Copy a inner element.
    ///
    /// # Safety
    /// - `index < current_len`
    #[inline(always)]
    pub const unsafe fn get(&self, index: usize) -> &Tick {
        unsafe { &*self.data.as_ptr().add(index) }
    }

    /// # Safety
    /// - `index < current_len`.
    #[inline(always)]
    pub const unsafe fn get_mut(&mut self, index: usize) -> &mut Tick {
        unsafe { &mut *self.data.as_ptr().add(index) }
    }

    /// # Safety
    /// - `slice_len <= current_len`.
    #[inline(always)]
    pub const unsafe fn as_slice(&self, slice_len: usize) -> &[Tick] {
        unsafe { slice::from_raw_parts(self.data.as_ptr(), slice_len) }
    }

    /// # Safety
    /// - `slice_len <= current_len`.
    #[inline(always)]
    pub const unsafe fn as_mut_slice(&mut self, slice_len: usize) -> &mut [Tick] {
        unsafe { slice::from_raw_parts_mut(self.data.as_ptr(), slice_len) }
    }

    /// # Safety
    /// - `current_len > 0`
    /// - `last_index == current_len - 1`
    #[inline(always)]
    pub const unsafe fn remove_last(&mut self, last_index: usize) -> Tick {
        unsafe { self.data.as_ptr().add(last_index).read() }
    }

    /// # Safety
    /// - `index < last_index`
    /// - `current_len > 0`
    /// - `last_index == current_len - 1`
    #[inline(always)]
    pub const unsafe fn swap_remove_not_last(&mut self, index: usize, last_index: usize) -> Tick {
        let base_ptr = self.data.as_ptr();
        unsafe {
            let last = base_ptr.add(last_index);
            let removal = base_ptr.add(index);

            let value = ptr::read(removal);
            ptr::copy_nonoverlapping(last, removal, 1);

            value
        }
    }

    /// # Safety
    /// - `index < last_index`
    /// - `current_len > 0`
    /// - `last_index == current_len - 1`
    #[inline(always)]
    pub const unsafe fn copy_remove_not_last(&mut self, index: usize, last_index: usize) {
        let base_ptr = self.data.as_ptr();

        unsafe {
            let last = base_ptr.add(last_index);
            let removal = base_ptr.add(index);

            ptr::copy_nonoverlapping(last, removal, 1);
        }
    }
}

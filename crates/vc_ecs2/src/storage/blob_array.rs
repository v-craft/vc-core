#![allow(clippy::new_without_default, reason = "internal type")]
#![allow(unused, reason = "todo")]

use alloc::alloc as malloc;
use core::alloc::Layout;
use core::num::NonZeroUsize;
use core::ptr::{self, NonNull};

use vc_ptr::{OwningPtr, Ptr, PtrMut};

use super::AbortOnDropFail;

// -----------------------------------------------------------------------------
// BlobArray

/// A type-erased `Vec` without length or capacity information.
///
/// The capacity and length will be stored by the upper-level container.
///
/// This is an internal type with a highly customized API. Some functions
/// have advanced semantics and are meant for specific scenarios only.
///
/// # Safety
/// - The `item_layout` must be valid.
/// - Users need to manage memory manually.
/// - The length and capacity provided by the caller must be correct.
#[derive(Debug)]
pub(super) struct BlobArray {
    item_layout: Layout,
    data: NonNull<u8>,
    drop_fn: Option<unsafe fn(OwningPtr<'_>)>,
}

impl BlobArray {
    /// Return `true` if `layout.size` is `0` .
    #[inline(always)]
    const fn is_zst(&self) -> bool {
        self.item_layout.size() == 0
    }

    /// Return the `Layout` of the item type.
    #[inline(always)]
    pub const fn layout(&self) -> Layout {
        self.item_layout
    }

    /// Return the item type's `drop` function.
    #[inline(always)]
    pub const fn drop_fn(&self) -> Option<unsafe fn(OwningPtr<'_>)> {
        self.drop_fn
    }

    /// # Safety
    /// - `layout` must be valid.
    /// - `drop_fn` must be valid and match the item type.
    #[inline(always)]
    pub const unsafe fn new(
        item_layout: Layout,
        drop_fn: Option<unsafe fn(OwningPtr<'_>)>,
    ) -> Self {
        let align = unsafe { NonZeroUsize::new_unchecked(item_layout.align()) };

        Self {
            item_layout,
            drop_fn,
            data: NonNull::without_provenance(align),
        }
    }

    /// # Safety
    /// - `current_capacity == 0` (not yet allocated).
    /// - `new_capacity * layout.size <= Isize::MAX`.
    pub unsafe fn alloc(&mut self, capacity: NonZeroUsize) {
        if !self.is_zst() {
            let new_layout = array_layout(self.item_layout, capacity.get());

            self.data = NonNull::new(unsafe { malloc::alloc(new_layout) })
                .unwrap_or_else(|| malloc::handle_alloc_error(new_layout));
        }
    }

    /// # Safety
    /// - `current_capacity` is correct and not zero.
    /// - `current_capacity <= new_capacity`.
    /// - `new_capacity * layout.size <= Isize::MAX`.
    pub unsafe fn realloc(&mut self, current_capacity: NonZeroUsize, new_capacity: NonZeroUsize) {
        if !self.is_zst() {
            let new_layout = array_layout(self.item_layout, new_capacity.get());

            self.data = NonNull::new(unsafe {
                malloc::realloc(
                    self.data.as_ptr(),
                    array_layout_unchecked(self.item_layout, current_capacity.get()),
                    new_layout.size(),
                )
            })
            .unwrap_or_else(|| malloc::handle_alloc_error(new_layout));
        }
    }

    /// # Safety
    /// - `current_capacity` and `current_len` is correct.
    pub unsafe fn dealloc(&mut self, current_capacity: usize, len: usize) {
        if current_capacity != 0 {
            unsafe {
                self.clear(len);

                if !self.is_zst() {
                    let layout = array_layout_unchecked(self.item_layout, current_capacity);
                    malloc::dealloc(self.data.as_ptr(), layout);
                }
            }
        }
    }

    /// # Safety
    /// - `len == current_len`.
    #[inline]
    pub unsafe fn clear(&mut self, len: usize) {
        if let Some(drop_fn) = self.drop_fn {
            let size = self.item_layout.size();
            let mut offset: usize = 0;

            let drop_guard = AbortOnDropFail;

            for _ in 0..len {
                unsafe {
                    drop_fn(OwningPtr::new(self.data.byte_add(offset)));
                }
                offset += size;
            }

            ::core::mem::forget(drop_guard);
        }
    }

    /// # Safety
    /// - `index < current_len`.
    #[inline(always)]
    pub const unsafe fn get(&self, index: usize) -> Ptr<'_> {
        let size = self.item_layout.size();
        unsafe { Ptr::new(self.data.byte_add(index * size)) }
    }

    /// # Safety
    /// - `index < current_len`.
    #[inline(always)]
    pub const unsafe fn get_mut(&mut self, index: usize) -> PtrMut<'_> {
        let size = self.item_layout.size();
        unsafe { PtrMut::new(self.data.byte_add(index * size)) }
    }

    /// # Safety
    /// - `slice_len <= current_len` .
    /// - type `T` is correct.
    #[inline(always)]
    pub const unsafe fn as_slice<T>(&self, slice_len: usize) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.data.as_ptr() as *const T, slice_len) }
    }

    /// # Safety
    /// - `index == current_len`
    /// - `current_len < current_capacity`
    /// - `value` point to a valid data
    #[inline(always)]
    pub const unsafe fn init_item(&mut self, index: usize, value: OwningPtr<'_>) {
        let size = self.item_layout.size();
        unsafe {
            let dst = self.data.as_ptr().byte_add(index * size);
            ptr::copy_nonoverlapping::<u8>(value.as_ptr(), dst, size);
        }
    }

    /// # Safety
    /// - `index < current_len`
    /// - `value` point to a valid data
    ///
    /// # Abort
    /// Abort if `drop` item panicked.
    #[inline]
    pub unsafe fn replace_item(&mut self, index: usize, value: OwningPtr<'_>) {
        let size = self.item_layout.size();

        let src = value.as_ptr();
        let dst = unsafe { self.data.byte_add(index * size) };

        if let Some(drop_fn) = self.drop_fn {
            let drop_guard = AbortOnDropFail;

            unsafe {
                drop_fn(OwningPtr::new(dst));
            }

            ::core::mem::forget(drop_guard);
        }

        // Overwriting the previous value.
        unsafe {
            ptr::copy_nonoverlapping::<u8>(src, dst.as_ptr(), size);
        }
    }

    /// # Safety
    /// - `current_len > 0`
    /// - `last_index == current_len - 1`
    /// - If the data needs to be "dropped", the caller needs to handle
    ///   the returned pointer correctly.
    #[inline(always)]
    #[must_use = "The returned pointer should be used to drop the removed element"]
    pub const unsafe fn remove_last(&mut self, last_index: usize) -> OwningPtr<'_> {
        let size = self.item_layout.size();
        unsafe { OwningPtr::new(self.data.byte_add(size * last_index)) }
    }

    /// # Safety
    /// - `current_len > 0`
    /// - `last_index == current_len - 1`
    ///
    /// # Abort
    /// Abort if `drop` item panicked.
    #[inline(always)]
    pub unsafe fn drop_last(&mut self, last_index: usize) {
        if let Some(drop_fn) = self.drop_fn {
            let drop_guard = AbortOnDropFail;

            let size = self.item_layout.size();
            unsafe {
                drop_fn(OwningPtr::new(self.data.byte_add(size * last_index)));
            }

            ::core::mem::forget(drop_guard);
        }
    }

    /// # Safety
    /// - `current_len > 0`
    /// - `last_index == current_len - 1`
    /// - `index < last_index`
    /// - If the data needs to be "dropped", the caller needs to handle
    ///   the returned pointer correctly.
    #[inline(always)]
    #[must_use = "The returned pointer should be used to drop the removed element"]
    pub const unsafe fn swap_remove_nonoverlapping(
        &mut self,
        index: usize,
        last_index: usize,
    ) -> OwningPtr<'_> {
        let size = self.item_layout.size();
        unsafe {
            let item = self.data.as_ptr().byte_add(size * index);
            let last = self.data.byte_add(size * last_index);
            ptr::swap_nonoverlapping::<u8>(item, last.as_ptr(), size);

            OwningPtr::new(last)
        }
    }

    /// # Safety
    /// - `current_len > 0`
    /// - `last_index == current_len - 1`
    /// - `index < last_index`
    ///
    /// # Abort
    /// Abort if `drop` item panicked.
    #[inline]
    pub unsafe fn swap_remove_and_drop_nonoverlapping(&mut self, index: usize, last_index: usize) {
        let drop_fn = self.drop_fn;

        unsafe {
            let value = self.swap_remove_nonoverlapping(index, last_index);
            if let Some(drop_fn) = drop_fn {
                drop_fn(value);
            }
        }
    }
}

// -----------------------------------------------------------------------------
// alloc helper

/// Similar to `Layout::repeat`
///
/// # Panic
/// Panic if `layout.size * n > isize::MAX`
#[inline]
const fn array_layout(layout: Layout, n: usize) -> Layout {
    #[cold]
    #[inline(never)]
    const fn invalid_size() -> ! {
        panic!("invalid size in `Layout::from_size_align`");
    }

    let Some(alloc_size) = layout.size().checked_mul(n) else {
        invalid_size();
    };

    if alloc_size > isize::MAX as usize {
        invalid_size();
    }

    unsafe { Layout::from_size_align_unchecked(alloc_size, layout.align()) }
}

/// # Safety
/// - `layout` is valid
/// - `layout.size * n <= isize::MAX`
#[inline]
const unsafe fn array_layout_unchecked(layout: Layout, n: usize) -> Layout {
    unsafe { Layout::from_size_align_unchecked(layout.size() * n, layout.align()) }
}

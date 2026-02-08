use alloc::alloc as malloc;
use core::alloc::Layout;
use core::cell::UnsafeCell;
use core::num::NonZeroUsize;
use core::ptr::NonNull;

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
#[derive(Debug)]
#[repr(transparent)]
pub(super) struct ThinArray<T> {
    data: NonNull<T>,
}

impl<T> Clone for ThinArray<T> {
    fn clone(&self) -> Self {
        *self
    }
}

/// impl `Copy` to ensure that `DebugLocation<ThinArray<T>>`
/// can be initialized in const function.
impl<T> Copy for ThinArray<T> {}

impl<T: Copy> ThinArray<UnsafeCell<T>> {
    const _STATIC_ASSERT_: () = const {
        assert!(::core::mem::size_of::<T>() == ::core::mem::size_of::<UnsafeCell<T>>());
        assert!(::core::mem::align_of::<T>() == ::core::mem::align_of::<UnsafeCell<T>>());
    };

    /// Create a empty `ThinArray`.
    #[inline(always)]
    pub const fn empty() -> Self {
        Self {
            data: NonNull::dangling(),
        }
    }

    /// Create a `ThinArray` with specific `with_capacity`.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        let mut arr = Self::empty();

        if let Some(capacity) = NonZeroUsize::new(capacity) {
            unsafe {
                arr.alloc(capacity);
            }
        }

        arr
    }

    /// Copy a inner element.
    ///
    /// # Safety
    /// - `index < current_len`
    #[inline(always)]
    pub const unsafe fn copy_item(&self, index: usize) -> T {
        unsafe { core::ptr::read(self.data.as_ptr().add(index) as *const T) }
    }
}

impl<T> ThinArray<T> {
    const IS_ZST: bool = ::core::mem::size_of::<T>() == 0;

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

    /// # Safety
    /// - `index < current_len`.
    #[inline(always)]
    pub const unsafe fn get_item(&self, index: usize) -> &T {
        unsafe { &*self.data.as_ptr().add(index) }
    }

    /// # Safety
    /// - `index < current_len`.
    #[inline(always)]
    pub const unsafe fn get_item_mut(&mut self, index: usize) -> &mut T {
        unsafe { &mut *self.data.as_ptr().add(index) }
    }

    /// # Safety
    /// - `slice_len <= current_len`.
    #[inline(always)]
    pub const unsafe fn as_slice(&self, slice_len: usize) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.data.as_ptr(), slice_len) }
    }

    /// # Safety
    /// - `index == current_len`
    /// - `current_len < current_capacity`
    #[inline(always)]
    pub const unsafe fn init_item(&mut self, index: usize, value: T) {
        unsafe {
            core::ptr::write(self.data.as_ptr().add(index), value);
        }
    }

    /// # Safety
    /// - `current_len > 0`
    /// - `last_index == current_len - 1`
    #[inline(always)]
    pub const unsafe fn remove_last(&mut self, last_index: usize) -> T {
        unsafe { core::ptr::read(self.data.as_ptr().add(last_index)) }
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
        let base_ptr = self.data.as_ptr();

        unsafe {
            let last = base_ptr.add(last_index);
            let removal = base_ptr.add(index);

            let value = core::ptr::read(removal);
            core::ptr::copy_nonoverlapping(last, removal, 1);

            value
        }
    }

    /// # Safety
    /// - `index < last_index`
    /// - `current_len > 0`
    /// - `last_index == current_len - 1`
    #[inline(always)]
    pub const unsafe fn copy_remove_nonoverlapping(&mut self, index: usize, last_index: usize) {
        let base_ptr = self.data.as_ptr();

        unsafe {
            let last = base_ptr.add(last_index);
            let removal = base_ptr.add(index);

            core::ptr::copy_nonoverlapping(last, removal, 1);
        }
    }
}

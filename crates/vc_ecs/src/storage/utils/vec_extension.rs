use alloc::vec::Vec;
use core::ptr;

// -----------------------------------------------------------------------------
// VecSwapRemove

pub(crate) trait VecSwapRemove<T> {
    /// # Safety
    /// - `vec.len() > 0`
    /// - `index < last_index`
    /// - `last_index == vec.len() - 1`
    unsafe fn swap_remove_nonoverlapping(&mut self, index: usize, last_index: usize) -> T;

    /// # Safety
    /// - `vec.len() > 0`
    /// - `last_index == vec.len() - 1`
    unsafe fn remove_last(&mut self, last_index: usize) -> T;
}

impl<T> VecSwapRemove<T> for Vec<T> {
    #[inline(always)]
    unsafe fn swap_remove_nonoverlapping(&mut self, index: usize, last_index: usize) -> T {
        let base_ptr = self.as_mut_ptr();

        unsafe {
            let removal = base_ptr.add(index);
            let last = base_ptr.add(last_index);

            let value = ptr::read(removal);

            ptr::copy_nonoverlapping(last, removal, 1);

            self.set_len(last_index);

            value
        }
    }

    #[inline(always)]
    unsafe fn remove_last(&mut self, last_index: usize) -> T {
        unsafe {
            let value = ptr::read(self.as_ptr().add(last_index));
            self.set_len(last_index);
            value
        }
    }
}

// -----------------------------------------------------------------------------
// VecCopyRemove

pub(crate) trait VecCopyRemove<T: Copy> {
    /// Copy the last element to the specified position and return it,
    /// then reduce the length.
    ///
    /// Note that the returned element is the copied last element,
    /// not the element that was overwritten.
    ///
    /// # Safety
    /// - `vec.len() > 0`
    /// - `index < last_index`
    /// - `last_index == vec.len() - 1`
    unsafe fn copy_last_and_return_nonoverlapping(&mut self, index: usize, last_index: usize) -> T;
}

impl<T: Copy> VecCopyRemove<T> for Vec<T> {
    #[inline(always)]
    unsafe fn copy_last_and_return_nonoverlapping(&mut self, index: usize, last_index: usize) -> T {
        let base_ptr = self.as_mut_ptr();

        unsafe {
            let src = base_ptr.add(last_index);
            let dst = base_ptr.add(index);

            ptr::copy_nonoverlapping(src, dst, 1);

            self.set_len(last_index);

            ptr::read(dst)
        }
    }
}

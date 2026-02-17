use alloc::vec::Vec;
use core::ptr;

// -----------------------------------------------------------------------------
// AbortOnPanic

/// A guard used to terminate a process
/// when memory allocation failure.
pub(super) struct AbortOnPanic;

impl Drop for AbortOnPanic {
    #[cold]
    #[inline(never)]
    fn drop(&mut self) {
        crate::cfg::std! {
            if {
                ::std::eprintln!("Aborting due to allocator error.");
                ::std::process::abort();
            } else {
                panic!("Aborting due to allocator error.");
            }
        }
    }
}

// -----------------------------------------------------------------------------
// AbortOnDropFail

/// A guard used to terminate a process
/// when `drop_fn` panicked.
pub(super) struct AbortOnDropFail;

impl Drop for AbortOnDropFail {
    #[cold]
    #[inline(never)]
    fn drop(&mut self) {
        crate::cfg::std! {
            if {
                ::std::eprintln!("Aborting due to drop_fn panicked.");
                ::std::process::abort();
            } else {
                panic!("Aborting due to drop_fn panicked.");
            }
        }
    }
}

// -----------------------------------------------------------------------------
// VecRemoveExt

pub(super) trait VecRemoveExt<T> {
    unsafe fn swap_remove_not_last(&mut self, index: usize, last_index: usize) -> T;
    unsafe fn remove_last(&mut self, last_index: usize) -> T;
    unsafe fn copy_return_not_last(&mut self, last_index: usize, to: usize) -> T;
}

impl<T: Copy> VecRemoveExt<T> for Vec<T> {
    #[inline(always)]
    unsafe fn swap_remove_not_last(&mut self, index: usize, last_index: usize) -> T {
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
            let last = self.as_ptr().add(last_index);

            let value = ptr::read(last);
            self.set_len(last_index);
            value
        }
    }

    #[inline(always)]
    unsafe fn copy_return_not_last(&mut self, last_index: usize, to: usize) -> T {
        let base_ptr = self.as_mut_ptr();

        unsafe {
            let src = base_ptr.add(last_index);
            let dst = base_ptr.add(to);

            let value = ptr::read(src);
            ptr::write(dst, value);
            self.set_len(last_index);
            value
        }
    }
}

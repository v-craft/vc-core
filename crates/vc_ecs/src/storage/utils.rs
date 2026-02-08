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
// VecRemoveExt

pub(super) trait VecRemoveExt<T> {
    unsafe fn remove_last(&mut self, last_index: usize) -> T;
    unsafe fn move_last_to(&mut self, last_index: usize, to: usize) -> T;
}

impl<T: Copy> VecRemoveExt<T> for Vec<T> {
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
    unsafe fn move_last_to(&mut self, last_index: usize, to: usize) -> T {
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

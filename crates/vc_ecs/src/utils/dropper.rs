// -----------------------------------------------------------------------------
// Dropper

use vc_ptr::OwningPtr;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Dropper {
    pub(crate) func: unsafe fn(OwningPtr<'_>),
}

impl Dropper {
    /// # Safety
    /// - `ptr` point to valid data.
    unsafe fn drop_fn<T>(ptr: OwningPtr<'_>) {
        ptr.debug_assert_aligned::<T>();
        unsafe {
            ptr.drop_as::<T>();
        }
    }

    pub const fn of<T>() -> Option<Dropper> {
        if ::core::mem::needs_drop::<T>() {
            Some(Dropper {
                func: Self::drop_fn::<T>,
            })
        } else {
            None
        }
    }

    #[inline(always)]
    pub(crate) unsafe fn call(self, ptr: OwningPtr<'_>) {
        unsafe { (self.func)(ptr) }
    }
}

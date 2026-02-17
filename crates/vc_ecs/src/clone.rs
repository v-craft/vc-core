use vc_ptr::{Ptr, PtrMut};

// -----------------------------------------------------------------------------
// CloneBehavior

#[derive(Debug, Clone, Copy)]
pub enum CloneBehavior {
    Ignore,
    Refuse,
    Custom(unsafe fn(Ptr<'_>, PtrMut<'_>)),
}

impl CloneBehavior {
    /// # Safety
    /// - `src` and`dst` point to valid data.
    /// - the data of `dst` is uninit, so it's no need to drop.
    unsafe fn clone_via_clone<T: Clone>(src: Ptr<'_>, dst: PtrMut<'_>) {
        src.debug_assert_aligned::<T>();
        dst.debug_assert_aligned::<T>();

        unsafe {
            let val = src.as_ref::<T>();
            let dst = dst.as_ptr() as *mut T;
            core::ptr::write(dst, val.clone());
        }
    }

    /// # Safety
    /// - `src` and`dst` point to valid data.
    unsafe fn clone_via_copy<T: Copy>(src: Ptr<'_>, dst: PtrMut<'_>) {
        src.debug_assert_aligned::<T>();
        dst.debug_assert_aligned::<T>();

        unsafe {
            let src = src.as_ptr() as *const T;
            let dst = dst.as_ptr() as *mut T;
            core::ptr::copy_nonoverlapping(src, dst, 1);
        }
    }

    pub const fn clonable<T: Clone>() -> Self {
        Self::Custom(Self::clone_via_clone::<T>)
    }

    pub const fn copyable<T: Copy>() -> Self {
        Self::Custom(Self::clone_via_copy::<T>)
    }
}

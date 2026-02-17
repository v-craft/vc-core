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

/// A type-erased `Box`.
///
/// # Safety
/// - The `layout` must be valid.
#[derive(Debug)]
pub(super) struct BlobBox {
    layout: Layout,
    data: NonNull<u8>,
    drop_fn: Option<unsafe fn(OwningPtr<'_>)>,
    is_valid: bool,
}

impl Drop for BlobBox {
    fn drop(&mut self) {
        self.drop_data();

        if self.layout.size() != 0 {
            unsafe {
                malloc::dealloc(self.data.as_ptr(), self.layout);
            }
        }
    }
}

impl BlobBox {
    #[inline(always)]
    pub const fn is_valid(&self) -> bool {
        self.is_valid
    }

    /// # Safety
    /// - `layout` must be valid.
    /// - `drop_fn` must be valid and match the item type.
    pub unsafe fn new(layout: Layout, drop_fn: Option<unsafe fn(OwningPtr<'_>)>) -> Self {
        let data = if layout.size() == 0 {
            let align = NonZeroUsize::new(layout.align()).unwrap();
            NonNull::without_provenance(align)
        } else {
            NonNull::new(unsafe { malloc::alloc(layout) })
                .unwrap_or_else(|| malloc::handle_alloc_error(layout))
        };

        Self {
            layout,
            drop_fn,
            data,
            is_valid: false,
        }
    }

    pub fn drop_data(&mut self) {
        if self.is_valid
            && let Some(drop_fn) = self.drop_fn
        {
            let drop_guard = AbortOnDropFail;

            self.is_valid = false;
            unsafe {
                let ptr = OwningPtr::new(self.data);
                drop_fn(ptr);
            }

            ::core::mem::forget(drop_guard);
        }
    }

    /// # Safety
    /// - The pointer must be valid and match the item type.
    #[inline]
    pub unsafe fn set(&mut self, data: OwningPtr<'_>) {
        self.drop_data();

        unsafe {
            ptr::copy_nonoverlapping::<u8>(data.as_ptr(), self.data.as_ptr(), self.layout.size());
        }
        self.is_valid = true;
    }

    /// # Safety
    /// `self.is_valid == true`
    #[inline(always)]
    pub unsafe fn get(&self) -> Ptr<'_> {
        unsafe { Ptr::new(self.data) }
    }

    /// # Safety
    /// `self.is_valid == true`
    #[inline(always)]
    pub unsafe fn get_mut(&mut self) -> PtrMut<'_> {
        unsafe { PtrMut::new(self.data) }
    }
}

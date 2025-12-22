use core::marker::PhantomData;
use core::ptr::NonNull;

/// A slice like `&'a [T]`, without length information for better performance.
///
/// It only has [`get`](Self::get) and [`from_ref`](Self::from_ref) methods,
/// where `get` does not check bounds and is always unsafe.
///
/// # Examples
///
/// ```
/// use vc_ptr::ThinSlicePtr;
///
/// let x = [1, 2, 3, 4];
///
/// let ptr = ThinSlicePtr::from_ref(&x);
///
/// assert_eq!(unsafe{ *ptr.get(2) }, 3);
/// ```
pub struct ThinSlicePtr<'a, T> {
    _marker: PhantomData<&'a [T]>,
    ptr: NonNull<T>,
    #[cfg(debug_assertions)]
    len: usize,
}

impl<T> Clone for ThinSlicePtr<'_, T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ThinSlicePtr<'_, T> {}

impl<'a, T> ThinSlicePtr<'a, T> {
    /// Indexes the slice without doing bounds checks.
    ///
    /// # Safety
    /// `index` must be in-bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use vc_ptr::ThinSlicePtr;
    ///
    /// let x = [1, 2, 3, 4];
    ///
    /// let ptr = ThinSlicePtr::from_ref(&x);
    ///
    /// assert_eq!(unsafe{ *ptr.get(2) }, 3);
    /// ```
    #[cfg_attr(debug_assertions, track_caller)]
    #[cfg_attr(not(debug_assertions), inline(always))]
    pub const unsafe fn get(self, index: usize) -> &'a T {
        // debug_assert! Use if branch to determine whether to execute.
        // Therefore, #[cfg] is needed.
        #[cfg(debug_assertions)]
        assert!(index < self.len, "tried to index out-of-bounds of a slice");

        // SAFETY: `index` is in-bounds so the resulting pointer is valid to deref.
        unsafe { &*self.ptr.as_ptr().add(index) }
    }

    /// Converts a reference to a `ThinSlicePtr` pointer.
    ///
    /// [`From::from`] is not const, but this is.
    #[inline(always)]
    pub const fn from_ref(r: &'a [T]) -> ThinSlicePtr<'a, T> {
        Self {
            _marker: PhantomData,
            ptr: NonNull::from_ref(r).cast(),
            #[cfg(debug_assertions)]
            len: r.len(),
        }
    }
}

impl<'a, T> From<&'a [T]> for ThinSlicePtr<'a, T> {
    #[inline]
    fn from(slice: &'a [T]) -> Self {
        Self::from_ref(slice)
    }
}

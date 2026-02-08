use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::ptr::NonNull;
use core::slice;

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
    /// ```no_run
    /// use vc_ptr::ThinSlicePtr;
    ///
    /// let x = [1, 2, 3, 4];
    ///
    /// let ptr = ThinSlicePtr::from_ref(&x);
    ///
    /// assert_eq!(unsafe{ *ptr.get(2) }, 3);
    /// ```
    #[cfg_attr(not(debug_assertions), inline(always))]
    pub const unsafe fn get(&self, index: usize) -> &'a T {
        // debug_assert! Use if branch to determine whether
        // to execute. Therefore, #[cfg] is needed.
        #[cfg(debug_assertions)]
        assert!(index < self.len, "tried to index out-of-bounds of a slice");

        unsafe { &*self.ptr.as_ptr().add(index) }
    }

    /// Returns a slice without performing bounds checks.
    ///
    /// # Safety
    /// - `len` must be less than or equal to the length of the slice.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vc_ptr::ThinSlicePtr;
    ///
    /// let x = [1, 2, 3, 4];
    /// let ptr = ThinSlicePtr::from_ref(&x);
    ///
    /// assert_eq!(
    ///     unsafe { ptr.as_slice(3) },
    ///     &[1, 2, 3],
    /// );
    /// ```
    #[cfg_attr(not(debug_assertions), inline(always))]
    pub const unsafe fn as_slice(&self, len: usize) -> &'a [T] {
        // debug_assert! Use if branch to determine whether
        // to execute. Therefore, #[cfg] is needed.
        #[cfg(debug_assertions)]
        assert!(len <= self.len, "tried to create an out-of-bounds slice");

        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), len) }
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

impl<'a, T> ThinSlicePtr<'a, UnsafeCell<T>> {
    /// Indexes the slice without doing bounds checks.
    ///
    /// # Safety
    /// - `index` must be less than the length of the slice.
    /// - There must not be any aliases for the lifetime `'a` to the item.
    #[cfg_attr(not(debug_assertions), inline(always))]
    pub const unsafe fn get_mut(&mut self, index: usize) -> &'a mut T {
        // debug_assert! Use if branch to determine whether
        // to execute. Therefore, #[cfg] is needed.
        #[cfg(debug_assertions)]
        assert!(index < self.len, "tried to index out-of-bounds of a slice");

        unsafe { &mut *self.ptr.as_ptr().cast::<T>().add(index) }
    }

    /// Returns a mutable reference of the slice
    ///
    /// # Safety
    /// - `len` must be less than or equal to the length of the slice.
    /// - There must not be any aliases for the lifetime `'a` to the slice.
    #[cfg_attr(not(debug_assertions), inline(always))]
    pub const unsafe fn as_mut_slice(&self, len: usize) -> &'a mut [T] {
        // debug_assert! Use if branch to determine whether
        // to execute. Therefore, #[cfg] is needed.
        #[cfg(debug_assertions)]
        assert!(len <= self.len, "tried to create an out-of-bounds slice");

        unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr().cast::<T>(), len) }
    }

    /// Returns a slice pointer to the underlying type `T`.
    #[inline(always)]
    pub const fn cast(&self) -> ThinSlicePtr<'a, T> {
        ThinSlicePtr {
            _marker: PhantomData,
            ptr: self.ptr.cast(),
            #[cfg(debug_assertions)]
            len: self.len,
        }
    }
}

impl<'a, T> From<&'a [T]> for ThinSlicePtr<'a, T> {
    #[inline]
    fn from(slice: &'a [T]) -> Self {
        Self::from_ref(slice)
    }
}

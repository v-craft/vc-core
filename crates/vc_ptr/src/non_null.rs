use core::fmt;
use core::ptr::NonNull;

/// A read-only `NonNull<T>`.
///
/// # Examples
///
/// ```
/// use vc_ptr::ConstNonNull;
///
/// let x = 10;
///
/// let ptr = ConstNonNull::from_ref(&x);
///
/// assert_eq!(unsafe{ *ptr.as_ref() }, 10);
/// ```
#[repr(transparent)]
pub struct ConstNonNull<T: ?Sized>(NonNull<T>);

impl<T: ?Sized> ConstNonNull<T> {
    /// Create a new `ConstNonNull` or return `None` if `ptr` is null.
    ///
    /// # Examples
    ///
    /// ```
    /// use vc_ptr::ConstNonNull;
    ///
    /// let x = 0u32;
    /// let ptr = ConstNonNull::new(&raw const x).expect("ptr is null!");
    /// ```
    #[inline]
    pub const fn new(ptr: *const T) -> Option<Self> {
        match NonNull::new(ptr.cast_mut()) {
            Some(x) => Some(Self(x)),
            None => None,
        }
    }

    /// Create a new `ConstNonNull` without checking for null.
    ///
    /// # Safety
    ///
    /// `ptr` must be non-null.
    ///
    /// # Examples
    ///
    /// ```
    /// use vc_ptr::ConstNonNull;
    ///
    /// let x = 0u32;
    /// let ptr = unsafe { ConstNonNull::new_unchecked(&raw const x) };
    /// ```
    #[inline(always)]
    pub const unsafe fn new_unchecked(ptr: *const T) -> Self {
        unsafe { Self(NonNull::new_unchecked(ptr.cast_mut())) }
    }

    /// Return an immutable reference to the value.
    ///
    /// # Safety
    ///
    /// When calling this method, you have to ensure that the pointer is
    /// [convertible to a reference](https://doc.rust-lang.org/stable/core/ptr/index.html#pointer-to-reference-conversion).
    ///
    /// # Examples
    ///
    /// ```
    /// use vc_ptr::ConstNonNull;
    ///
    /// let x = 0;
    /// let ptr = ConstNonNull::new(&raw const x).expect("ptr is null!");
    ///
    /// let ref_x = unsafe { ptr.as_ref() };
    /// assert_eq!(*ref_x, 0);
    /// ```
    #[inline(always)]
    pub const unsafe fn as_ref<'a>(&self) -> &'a T {
        // Safety: See `NonNull::as_ref`
        unsafe { self.0.as_ref() }
    }

    /// Acquires the underlying `*const` pointer.
    #[inline(always)]
    pub const fn as_ptr(&self) -> *const T {
        self.0.as_ptr()
    }

    /// Converts a reference to a `ConstNonNull` pointer.
    #[inline(always)]
    pub const fn from_ref(r: &T) -> Self {
        Self(NonNull::from_ref(r))
    }

    /// Converts a mutable reference to a `ConstNonNull` pointer.
    #[inline(always)]
    pub const fn from_mut(r: &mut T) -> Self {
        Self(NonNull::from_mut(r))
    }
}

impl<T: ?Sized> From<NonNull<T>> for ConstNonNull<T> {
    #[inline(always)]
    fn from(value: NonNull<T>) -> Self {
        Self(value)
    }
}

impl<T: ?Sized> Clone for ConstNonNull<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Copy for ConstNonNull<T> {}

impl<T: ?Sized> fmt::Pointer for ConstNonNull<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.as_ptr(), f)
    }
}

impl<T: ?Sized> fmt::Debug for ConstNonNull<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.as_ptr(), f)
    }
}

// -----------------------------------------------------------------------------
// Optional

/*
use core::fmt;

impl<'a, T: ?Sized> From<&'a T> for ConstNonNull<T> {
    #[inline]
    fn from(value: &'a T) -> Self {
        Self(NonNull::from_ref(value))
    }
}

impl<'a, T: ?Sized> From<&'a mut T> for ConstNonNull<T> {
    #[inline]
    fn from(value: &'a mut T) -> Self {
        Self(NonNull::from_mut(value))
    }
}

impl<T: ?Sized> Eq for ConstNonNull<T> {}

impl<T: ?Sized> PartialEq for ConstNonNull<T> {
    #[inline]
    #[expect(ambiguous_wide_pointer_comparisons)]
    fn eq(&self, other: &Self) -> bool {
        self.as_ptr() == other.as_ptr()
    }
}

*/

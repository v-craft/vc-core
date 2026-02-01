//! Fixed-capacity circular array buffer with stack storage.
//!
//! Currently only the most basic APIs are provided.
//!
//! Advanced APIs may be added in future releases
//! based on usage patterns and community feedback.
#![expect(unsafe_code, reason = "original implementation")]

use core::fmt;
use core::mem::MaybeUninit;
use core::ptr;

// -----------------------------------------------------------------------------
// ArrayDeque

/// A ring buffer with fixed capacity, storing data on the stack.
///
/// `ArrayDeque` is a double-ended queue (deque) implemented as a circular buffer
/// with compile-time fixed capacity `N`. All data is stored in an array on the stack,
/// avoiding heap allocations.
///
/// Note that the back operation is faster than front.
///
/// ```
/// use vc_utils::extra::ArrayDeque;
///
/// let mut deque: ArrayDeque<i32, 4> = ArrayDeque::new();
///
/// // Push elements to the back
/// deque.push_back(1).unwrap();
/// deque.push_back(2).unwrap();
///
/// // Push elements to the front
/// deque.push_front(0).unwrap();
///
/// // Access elements
/// assert_eq!(deque.front(), Some(&0));
/// assert_eq!(deque.back(), Some(&2));
///
/// // Pop elements
/// assert_eq!(deque.pop_front(), Some(0));
/// assert_eq!(deque.pop_back(), Some(2));
///
/// // Check capacity constraints
/// deque.push_back(3).unwrap();
/// deque.push_back(4).unwrap();
/// deque.push_back(5).unwrap();
/// assert!(deque.is_full());
/// assert_eq!(deque.push_back(6), Err(6)); // Full, returns the element
/// ```
pub struct ArrayDeque<T, const N: usize> {
    slots: [MaybeUninit<T>; N],
    tail: usize,
    len: usize,
}

impl<T, const N: usize> Drop for ArrayDeque<T, N> {
    fn drop(&mut self) {
        if !core::mem::needs_drop::<T>() || self.len == 0 {
            return;
        }
        self.drop_inner();
    }
}

impl<T, const N: usize> Default for ArrayDeque<T, N> {
    /// Create an empty `ArrayDeque` with uninitialized backing storage.
    ///
    /// This uses `MaybeUninit` to avoid initializing the array elements.
    /// The returned queue has `len == 0` and `tail == 0`.
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> ArrayDeque<T, N> {
    #[inline]
    fn drop_inner(&mut self) {
        if !core::mem::needs_drop::<T>() {
            return;
        }
        if self.len == N {
            unsafe {
                ptr::drop_in_place(ptr::slice_from_raw_parts_mut::<T>(
                    self.slots.as_mut_ptr() as *mut T,
                    N,
                ));
            }
            return;
        }
        let begin = (self.tail + N - self.len) % N;
        if self.tail > begin {
            unsafe {
                ptr::drop_in_place(ptr::slice_from_raw_parts_mut::<T>(
                    self.slots.as_mut_ptr().add(begin) as *mut T,
                    self.len,
                ));
            }
        } else {
            unsafe {
                ptr::drop_in_place(ptr::slice_from_raw_parts_mut::<T>(
                    self.slots.as_mut_ptr() as *mut T,
                    self.tail,
                ));
                ptr::drop_in_place(ptr::slice_from_raw_parts_mut::<T>(
                    self.slots.as_mut_ptr().add(begin) as *mut T,
                    N - begin,
                ));
            }
        }
    }

    /// Removes all elements from the deque.
    ///
    /// This method drops all elements currently in the deque and resets
    /// its internal state to empty. The capacity remains unchanged.
    pub fn clear(&mut self) {
        if self.len > 0 {
            self.drop_inner();
        }
        self.tail = 0;
        self.len = 0;
    }

    /// Creates an empty `ArrayDeque` with uninitialized backing storage.
    ///
    /// # Examples
    ///
    /// ```
    /// use vc_utils::extra::ArrayDeque;
    ///
    /// let deque: ArrayDeque<i32, 10> = ArrayDeque::new();
    /// assert!(deque.is_empty());
    /// assert!(!deque.is_full());
    /// assert_eq!(deque.len(), 0);
    /// ```
    ///
    /// Note that the capacity `0` is valid.
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            slots: unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() },
            tail: 0,
            len: 0,
        }
    }

    /// Returns `true` if the buffer is full (len == capacity).
    ///
    /// When the deque is full, any attempt to push additional elements
    /// will fail, returning the element in an `Err`.
    ///
    /// # Examples
    ///
    /// ```
    /// use vc_utils::extra::ArrayDeque;
    ///
    /// let mut deque: ArrayDeque<i32, 2> = ArrayDeque::new();
    /// assert!(!deque.is_full());
    ///
    /// deque.push_back(1).unwrap();
    /// deque.push_back(2).unwrap();
    /// assert!(deque.is_full());
    ///
    /// // Attempting to push another element will fail
    /// assert!(deque.push_back(3).is_err());
    /// ```
    #[inline(always)]
    pub const fn is_full(&self) -> bool {
        self.len == N
    }

    /// Returns `true` if the buffer is empty (len == 0).
    ///
    /// # Examples
    ///
    /// ```
    /// use vc_utils::extra::ArrayDeque;
    ///
    /// let mut deque: ArrayDeque<i32, 4> = ArrayDeque::new();
    /// assert!(deque.is_empty());
    ///
    /// deque.push_back(1).unwrap();
    /// assert!(!deque.is_empty());
    /// ```
    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the number of elements in the deque.
    ///
    /// # Examples
    ///
    /// ```
    /// use vc_utils::extra::ArrayDeque;
    ///
    /// let mut deque: ArrayDeque<i32, 4> = ArrayDeque::new();
    /// assert_eq!(deque.len(), 0);
    ///
    /// deque.push_back(1).unwrap();
    /// deque.push_back(2).unwrap();
    /// assert_eq!(deque.len(), 2);
    /// ```
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns a reference to the front element, if present.
    ///
    /// This method does not remove the element from the deque.
    ///
    /// # Examples
    ///
    /// ```
    /// use vc_utils::extra::ArrayDeque;
    ///
    /// let mut deque: ArrayDeque<i32, 4> = ArrayDeque::new();
    /// assert_eq!(deque.front(), None);
    ///
    /// deque.push_back(1).unwrap();
    /// deque.push_back(2).unwrap();
    /// assert_eq!(deque.front(), Some(&1));
    ///
    /// deque.push_front(0).unwrap();
    /// assert_eq!(deque.front(), Some(&0));
    /// ```
    pub const fn front(&self) -> Option<&T> {
        if !self.is_empty() {
            let front = (self.tail + N - self.len) % N;
            unsafe { Some(&*self.slots.as_ptr().add(front).cast::<T>()) }
        } else {
            None
        }
    }

    /// Returns a reference to the back element, if present.
    ///
    /// This method does not remove the element from the deque.
    ///
    /// # Examples
    ///
    /// ```
    /// use vc_utils::extra::ArrayDeque;
    ///
    /// let mut deque: ArrayDeque<i32, 4> = ArrayDeque::new();
    /// assert_eq!(deque.back(), None);
    ///
    /// deque.push_back(1).unwrap();
    /// deque.push_back(2).unwrap();
    /// assert_eq!(deque.back(), Some(&2));
    ///
    /// deque.pop_back();
    /// assert_eq!(deque.back(), Some(&1));
    /// ```
    pub const fn back(&self) -> Option<&T> {
        if !self.is_empty() {
            let back = (self.tail + N - 1) % N;
            unsafe { Some(&*self.slots.as_ptr().add(back).cast::<T>()) }
        } else {
            None
        }
    }

    /// Pushes an element to the front of the deque without checking capacity.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `!self.is_full()`.
    /// Calling this method when the deque is full results in undefined behavior.
    #[inline(always)]
    pub const unsafe fn push_front_unchecked(&mut self, element: T) {
        let begin = (self.tail + (N << 1) - self.len - 1) % N;
        unsafe {
            ptr::write(self.slots.as_mut_ptr().add(begin) as *mut T, element);
        }
        self.len += 1;
    }

    /// Pushes an element to the front of the deque.
    ///
    /// Returns `Ok(())` if the element was inserted successfully.
    /// If the deque is full, returns `Err(element)` and leaves the deque unchanged.
    ///
    /// # Examples
    ///
    /// ```
    /// use vc_utils::extra::ArrayDeque;
    ///
    /// let mut deque: ArrayDeque<i32, 2> = ArrayDeque::new();
    ///
    /// assert_eq!(deque.push_front(1), Ok(()));
    /// assert_eq!(deque.push_front(0), Ok(()));
    /// assert_eq!(deque.front(), Some(&0));
    ///
    /// // Deque is now full
    /// assert_eq!(deque.push_front(-1), Err(-1));
    /// assert_eq!(deque.len(), 2); // Length unchanged
    /// ```
    #[inline]
    pub const fn push_front(&mut self, element: T) -> Result<(), T> {
        if !self.is_full() {
            unsafe {
                self.push_front_unchecked(element);
            }
            Ok(())
        } else {
            Err(element)
        }
    }

    /// Pushes an element to the back of the deque without checking capacity.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `!self.is_full()`.
    /// Calling this method when the deque is full results in undefined behavior.
    #[inline(always)]
    pub const unsafe fn push_back_unchecked(&mut self, element: T) {
        unsafe {
            ptr::write(self.slots.as_mut_ptr().add(self.tail) as *mut T, element);
        }
        self.tail = (self.tail + 1) % N;
        self.len += 1;
    }

    /// Pushes an element to the back of the deque.
    ///
    /// Returns `Ok(())` if the element was inserted successfully.
    /// If the deque is full, returns `Err(element)` and leaves the deque unchanged.
    ///
    /// # Examples
    ///
    /// ```
    /// use vc_utils::extra::ArrayDeque;
    ///
    /// let mut deque: ArrayDeque<i32, 2> = ArrayDeque::new();
    ///
    /// assert_eq!(deque.push_back(1), Ok(()));
    /// assert_eq!(deque.push_back(2), Ok(()));
    /// assert_eq!(deque.back(), Some(&2));
    ///
    /// // Deque is now full
    /// assert_eq!(deque.push_back(3), Err(3));
    /// assert_eq!(deque.len(), 2); // Length unchanged
    /// ```
    #[inline]
    pub const fn push_back(&mut self, element: T) -> Result<(), T> {
        if !self.is_full() {
            unsafe {
                self.push_back_unchecked(element);
            }
            Ok(())
        } else {
            Err(element)
        }
    }

    /// Removes and returns the front element of the deque.
    ///
    /// Returns `Some(T)` if the deque is not empty, otherwise returns `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use vc_utils::extra::ArrayDeque;
    ///
    /// let mut deque: ArrayDeque<i32, 4> = ArrayDeque::new();
    /// deque.push_back(1).unwrap();
    /// deque.push_back(2).unwrap();
    ///
    /// assert_eq!(deque.pop_front(), Some(1));
    /// assert_eq!(deque.pop_front(), Some(2));
    /// assert_eq!(deque.pop_front(), None);
    /// ```
    #[inline]
    pub const fn pop_front(&mut self) -> Option<T> {
        if !self.is_empty() {
            let begin = (self.tail + N - self.len) % N;
            let value = unsafe { ptr::read(self.slots.as_mut_ptr().add(begin) as *mut T) };
            self.len -= 1;
            Some(value)
        } else {
            None
        }
    }

    /// Removes and returns the back element of the deque.
    ///
    /// Returns `Some(T)` if the deque is not empty, otherwise returns `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use vc_utils::extra::ArrayDeque;
    ///
    /// let mut deque: ArrayDeque<i32, 4> = ArrayDeque::new();
    /// deque.push_back(1).unwrap();
    /// deque.push_back(2).unwrap();
    ///
    /// assert_eq!(deque.pop_back(), Some(2));
    /// assert_eq!(deque.pop_back(), Some(1));
    /// assert_eq!(deque.pop_back(), None);
    /// ```
    #[inline]
    pub const fn pop_back(&mut self) -> Option<T> {
        if !self.is_empty() {
            self.tail = (self.tail + N - 1) % N;
            let value = unsafe { ptr::read(self.slots.as_mut_ptr().add(self.tail) as *mut T) };
            self.len -= 1;
            Some(value)
        } else {
            None
        }
    }
}

impl<T, const N: usize> fmt::Debug for ArrayDeque<T, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArrayDeque")
            .field("len", &self.len)
            .finish()
    }
}

// -----------------------------------------------------------------------------
// Tests

#[cfg(test)]
mod tests {
    use super::ArrayDeque;

    #[test]
    fn is_sync_send() {
        use core::panic::{RefUnwindSafe, UnwindSafe};

        fn is_send<T: Send>() {}
        fn is_sync<T: Send>() {}
        fn is_unwindsafe<T: UnwindSafe>() {}
        fn is_refunwindsafe<T: RefUnwindSafe>() {}

        is_send::<ArrayDeque<i32, 0>>();
        is_sync::<ArrayDeque<i32, 0>>();
        is_unwindsafe::<ArrayDeque<i32, 0>>();
        is_refunwindsafe::<ArrayDeque<i32, 0>>();
    }
}

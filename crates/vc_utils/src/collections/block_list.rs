//! Provide block based linked-list.
//!
//! Improve cache hit rate through block data.
//!
//! Currently only the most basic APIs are provided.
//!
//! Advanced APIs may be added in future releases
//! based on usage patterns and community feedback.
#![expect(unsafe_code, reason = "original implementation")]

use alloc::boxed::Box;
use core::fmt;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::panic::{RefUnwindSafe, UnwindSafe};
use core::ptr;

use crate::vec::StackVec;

// -----------------------------------------------------------------------------
// Config

const BLOCK_SIZE: usize = 13;

const MAX_IDLE: usize = 4;

// -----------------------------------------------------------------------------
// Block

/// A single queue block.
struct Block<T> {
    /// the index of head.
    ///
    /// For example the buffer is `[0, 0, 1, 1, 0]`
    /// (`0` indicates no data), then this index is `2`.
    head: usize,

    /// the index of tail.
    ///
    /// For example the buffer is `[0, 0, 1, 1, 0]`
    /// (`0` indicates no data), then this index is `4`.
    tail: usize,

    data: [MaybeUninit<T>; BLOCK_SIZE],
    next: *mut Block<T>,
}

impl<T> Block<T> {
    /// Create a empty block.
    #[cold]
    fn new() -> Box<Self> {
        Box::new(
            const {
                Block::<T> {
                    head: 0,
                    tail: 0,
                    // SAFETY: Convert full uninit to internal uninit is safe.
                    data: unsafe {
                        <MaybeUninit<[MaybeUninit<T>; BLOCK_SIZE]>>::uninit().assume_init()
                    },
                    next: ptr::null_mut(),
                }
            },
        )
    }

    #[inline(always)]
    fn reset(&mut self) {
        self.head = 0;
        self.tail = 0;
        self.next = ptr::null_mut();
    }
}

/// Drop remaining initialized elements in a block.
///
/// Only elements in range [head_index, tail_index) are valid.
impl<T> Drop for Block<T> {
    fn drop(&mut self) {
        if self.head < self.tail {
            unsafe {
                ptr::drop_in_place(ptr::slice_from_raw_parts_mut::<T>(
                    self.data.as_mut_ptr().add(self.head) as *mut T,
                    self.tail - self.head,
                ));
            }
        }
    }
}

// -----------------------------------------------------------------------------
// BlockList

/// A queue implemented as a linked list of fixed-size blocks.
///
/// `BlockList` provides an efficient queue implementation that:
///
/// - Allocates memory in fixed-size blocks (16 elements per block)
/// - Recycles fully popped blocks to avoid frequent allocations
/// - Maintains a small pool of idle blocks (up to 4) for reuse
///
/// # Examples
///
/// ```
/// # use vc_utils::collections::BlockList;
/// let mut queue = BlockList::new();
/// assert!(queue.is_empty());
///
/// queue.push_back(1);
/// queue.push_back(2);
///
/// assert_eq!(queue.pop_front(), Some(1));
/// assert_eq!(queue.len(), 1);
///
/// assert_eq!(queue.pop_front(), Some(2));
/// assert_eq!(queue.pop_front(), None);
/// ```
pub struct BlockList<T> {
    head_ptr: *mut Block<T>,
    tail_ptr: *mut Block<T>,
    block_num: usize,
    idle: StackVec<Box<Block<T>>, MAX_IDLE>,
    _marker: PhantomData<T>,
}

unsafe impl<T: Sync> Sync for BlockList<T> {}
unsafe impl<T: Send> Send for BlockList<T> {}
impl<T: UnwindSafe> UnwindSafe for BlockList<T> {}
impl<T: RefUnwindSafe> RefUnwindSafe for BlockList<T> {}

impl<T> Drop for BlockList<T> {
    fn drop(&mut self) {
        let mut ptr = self.head_ptr;
        while !ptr.is_null() {
            unsafe {
                let boxed = Box::from_raw(ptr);
                ptr = (*ptr).next;
                ::core::mem::drop(boxed);
            }
        }
    }
}

impl<T> BlockList<T> {
    #[inline]
    fn get_block(&mut self) -> *mut Block<T> {
        if let Some(mut boxed) = self.idle.pop() {
            boxed.reset();
            Box::leak(boxed)
        } else {
            Box::leak(<Block<T>>::new())
        }
    }

    #[inline]
    fn idle_block(&mut self, ptr: *mut Block<T>) {
        // SAFERT: valid ptr, created through `Box::new`.
        let boxed = unsafe { Box::from_raw(ptr) };
        if !self.idle.is_full() {
            // SAFETY: !is_full()
            unsafe {
                self.idle.push_unchecked(boxed);
            }
        }
    }

    /// Creates an empty `BlockList`.
    ///
    /// This function does not allocate any memory.
    /// The first allocation occurs when the first element is pushed.
    ///
    /// # Examples
    ///
    /// ```
    /// use vc_utils::collections::BlockList;
    ///
    /// let queue: BlockList<i32> = BlockList::new();
    /// assert!(queue.is_empty());
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Self {
            head_ptr: ptr::null_mut(),
            tail_ptr: ptr::null_mut(),
            block_num: 0,
            idle: StackVec::new(),
            _marker: PhantomData,
        }
    }

    /// Create a non-idle block , set head_ptr and tail_ptr.
    ///
    /// # Safety
    ///
    /// Self is uninit (head_ptr and tail_ptr is null).
    #[cold]
    #[inline(never)]
    fn init(&mut self) {
        debug_assert!(self.head_ptr.is_null());
        debug_assert!(self.tail_ptr.is_null());
        debug_assert_eq!(self.block_num, 0);
        let ptr = self.get_block();
        self.head_ptr = ptr;
        self.tail_ptr = ptr;
    }

    /// Appends an element to the back of the queue.
    ///
    /// If the current tail block is full, a new block will be allocated
    /// (or reused from the idle pool) and linked to the queue.
    ///
    /// # Examples
    ///
    /// ```
    /// # use vc_utils::collections::BlockList;
    /// let mut queue = BlockList::new();
    ///
    /// queue.push_back(1);
    /// queue.push_back(2);
    /// assert_eq!(queue.len(), 2);
    /// ```
    pub fn push_back(&mut self, value: T) {
        if self.tail_ptr.is_null() {
            self.init();
        }

        // SAFETY: `tail_ptr` point to valid data.
        let block = unsafe { &mut *self.tail_ptr };

        let index = block.tail;
        debug_assert!(index < BLOCK_SIZE);

        // SAFETY: valid index and pointer
        unsafe {
            ptr::write(block.data.as_mut_ptr().add(index) as *mut T, value);
        }

        block.tail = index + 1;

        if block.tail == BLOCK_SIZE {
            let new_block = self.get_block();
            block.next = new_block;

            self.tail_ptr = new_block;
            self.block_num += 1;
        }
    }

    /// Removes and returns the element from the front of the queue.
    ///
    /// Returns `None` if the queue is empty.
    /// If a block becomes empty after popping, it is moved to the idle pool
    /// for potential reuse (up to `MAX_IDLE` blocks).
    ///
    /// # Examples
    ///
    /// ```
    /// # use vc_utils::collections::BlockList;
    /// let mut queue = BlockList::new();
    ///
    /// queue.push_back(1);
    /// queue.push_back(2);
    /// assert_eq!(queue.pop_front(), Some(1));
    /// assert_eq!(queue.pop_front(), Some(2));
    /// assert_eq!(queue.pop_front(), None);
    /// ```
    pub fn pop_front(&mut self) -> Option<T> {
        if self.head_ptr.is_null() {
            return None;
        }

        // SAFETY: `guard.0` point to valid data.
        let block = unsafe { &mut *self.head_ptr };
        let index = block.head;
        debug_assert!(index < BLOCK_SIZE);
        debug_assert!(index <= block.tail);

        if index == block.tail {
            return None;
        }

        // SAFETY: valid index and pointer
        let value = unsafe { ptr::read(block.data.as_ptr().add(index) as *mut T) };

        block.head = index + 1;

        if block.head == BLOCK_SIZE {
            let old_ptr = block as *mut Block<T>;
            let next_ptr = block.next;
            // index + 1 == BLOCK_SIZE, so tail_index == BLOCK_SIZE.
            // next_ptr must be set by `push` function.
            debug_assert!(!next_ptr.is_null());
            self.head_ptr = next_ptr;
            self.block_num -= 1;
            self.idle_block(old_ptr);
        }
        Some(value)
    }

    /// Returns `true` if the queue contains no elements.
    ///
    /// O(1) time complexity.
    ///
    /// # Examples
    ///
    /// ```
    /// # use vc_utils::collections::BlockList;
    /// let mut queue = BlockList::new();
    ///
    /// assert!(queue.is_empty());
    /// queue.push_back(1);
    /// assert!(!queue.is_empty());
    /// queue.pop_front();
    /// assert!(queue.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        if self.head_ptr.is_null() {
            return true;
        }
        let block = unsafe { &*self.head_ptr };
        block.tail == block.head
    }

    /// Returns the number of elements in the queue.
    ///
    /// O(1) time complexity.
    ///
    /// # Examples
    ///
    /// ```
    /// # use vc_utils::collections::BlockList;
    /// let mut queue = BlockList::new();
    ///
    /// queue.push_back(1);
    /// queue.push_back(2);
    /// queue.push_back(3);
    /// assert_eq!(queue.len(), 3);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        if self.head_ptr.is_null() {
            return 0;
        }
        debug_assert!(!self.tail_ptr.is_null());
        let head_index = unsafe { (*self.head_ptr).head };
        let tail_index = unsafe { (*self.tail_ptr).tail };
        self.block_num * BLOCK_SIZE + tail_index - head_index
    }

    /// Clears the queue, removing all values.
    ///
    /// After calling `clear`, the queue will be empty.
    /// Blocks that become empty are moved to the idle pool for reuse.
    ///
    /// # Examples
    ///
    /// ```
    /// # use vc_utils::collections::BlockList;
    /// let mut queue = BlockList::new();
    ///
    /// queue.push_back(1);
    /// queue.push_back(2);
    ///
    /// queue.clear();
    /// assert!(queue.is_empty());
    /// assert_eq!(queue.len(), 0);
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        loop {
            if self.pop_front().is_none() {
                return;
            }
        }
    }
}

impl<T> Default for BlockList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> fmt::Debug for BlockList<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BlockList")
            .field("len", &self.len())
            .finish()
    }
}

// -----------------------------------------------------------------------------
// Tests

#[cfg(test)]
mod tests {
    use super::BlockList;

    #[test]
    fn is_sync_send() {
        use core::panic::{RefUnwindSafe, UnwindSafe};

        fn is_send<T: Send>() {}
        fn is_sync<T: Send>() {}
        fn is_unwindsafe<T: UnwindSafe>() {}
        fn is_refunwindsafe<T: RefUnwindSafe>() {}

        is_send::<BlockList<i32>>();
        is_sync::<BlockList<i32>>();
        is_unwindsafe::<BlockList<i32>>();
        is_refunwindsafe::<BlockList<i32>>();
    }
}

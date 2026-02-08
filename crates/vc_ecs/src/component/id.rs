#![expect(unsafe_code, reason = "unchecked non-zero is unsafe.")]

use alloc::vec::Vec;
use core::fmt;
use core::hash;
use core::num::NonZeroU32;
use core::sync::atomic::Ordering;

use nonmax::NonMaxU32;
use vc_os::sync::atomic::AtomicU32;

// -----------------------------------------------------------------------------
// ComponentId

/// Unique identifier for a `Component` type.
///
/// Component IDs are only valid for a given World, and are not globally unique.
#[derive(Debug, Clone, Copy, PartialOrd, Ord)]
#[repr(transparent)]
pub struct ComponentId(NonZeroU32);

impl ComponentId {
    const _STATIC_ASSERT_: () = const {
        const VAL: u32 = 2026;
        const ID: ComponentId = unsafe { core::mem::transmute(VAL) };
        assert!(VAL == ID.0.get());
        assert!(VAL == ID.index_u32());
    };

    /// Create a `ComponentId` from index.
    #[inline(always)]
    pub const fn new(index: NonZeroU32) -> Self {
        Self(index)
    }

    /// Create a `ComponentId` from u32.
    ///
    /// # Panic
    /// Panic if `index == 0`.
    #[inline(always)]
    pub const fn from_u32(index: u32) -> Self {
        Self(NonZeroU32::new(index).unwrap())
    }

    /// Convert this component ID to u32.
    #[inline(always)]
    pub const fn index_u32(self) -> u32 {
        unsafe { core::mem::transmute(self) }
    }

    /// Convert this component ID to usize.
    #[inline(always)]
    pub const fn index(self) -> usize {
        self.index_u32() as usize
    }
}

impl PartialEq for ComponentId {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.index_u32() == other.index_u32()
    }
}

impl Eq for ComponentId {}

impl hash::Hash for ComponentId {
    #[inline(always)]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        state.write_u32(self.index_u32());
    }
}

impl fmt::Display for ComponentId {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.index_u32(), f)
    }
}

// -----------------------------------------------------------------------------
// ComponentIndices

/// A two-level index table for mapping Component IDs to secondary indices.
///
/// This is used in `Archetypes` and `SparseSets`.
///
/// Using `NonMaxU32` instead of `u32` reduces memory usage by half.
#[derive(Debug, Default, Clone)]
pub struct ComponentIndices {
    indices: Vec<Option<NonMaxU32>>,
}

impl ComponentIndices {
    /// Creates an empty `ComponentIndices`.
    #[inline]
    pub const fn new() -> Self {
        Self {
            indices: Vec::new(),
        }
    }

    /// Returns `true` if the specified `ComponentId` exists in the index table.
    #[inline]
    pub fn contains(&self, id: ComponentId) -> bool {
        let index = id.index();
        self.indices.get(index).is_some_and(Option::is_some)
    }

    /// Returns the secondary index for the specified `ComponentId`.
    #[inline]
    pub fn get(&self, id: ComponentId) -> Option<NonMaxU32> {
        let index = id.index();
        self.indices.get(index).and_then(|&v| v)
    }

    /// Sets the secondary index for the specified `ComponentId`.
    #[inline]
    pub fn set(&mut self, id: ComponentId, value: NonMaxU32) {
        let index = id.index();
        if index >= self.indices.len() {
            self.indices.resize(index + 1, None);
        }

        #[expect(
            unsafe_code,
            reason = "Index is guaranteed to be in bounds after resize"
        )]
        unsafe {
            *self.indices.get_unchecked_mut(index) = Some(value);
        }
    }

    /// Clears all indices while preserving capacity.
    #[inline]
    pub fn clear(&mut self) {
        self.indices.clear();
    }
}

// -----------------------------------------------------------------------------
// ComponentIdAllocator

/// An allocator for `ComponentId` that starts allocation from `1`.
///
/// # Panics
/// Panics if the allocated ID would exceed or equal `u32::MAX`.
#[derive(Debug)]
pub struct ComponentIdAllocator {
    next: AtomicU32,
}

impl Default for ComponentIdAllocator {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl ComponentIdAllocator {
    #[inline(always)]
    const unsafe fn force_cast(id: u32) -> ComponentId {
        unsafe { core::mem::transmute(id) }
    }

    /// Creates a new `ComponentIdAllocator` that starts allocating IDs from `1`.
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            // SAFETY: IDs start from `1` instead of `0`.
            next: AtomicU32::new(1),
        }
    }

    /// Returns the number of IDs that have been allocated.
    #[inline(always)]
    pub fn count(&self) -> usize {
        self.next.load(Ordering::Relaxed) as usize - 1
    }

    /// Returns the next ID without allocating it.
    ///
    /// This operation is atomic and does not increment the internal counter.
    #[inline]
    pub fn peek(&self) -> ComponentId {
        let next = self.next.load(Ordering::Relaxed);
        unsafe { Self::force_cast(next) }
    }

    /// Allocates and returns the next available ID.
    ///
    /// This operation is atomic and increments the internal counter.
    #[inline]
    pub fn next(&self) -> ComponentId {
        let next = self.next.fetch_add(1, Ordering::Relaxed);
        assert!(next < u32::MAX, "too many components");
        unsafe { Self::force_cast(next) }
    }

    /// Returns the next ID without allocating it.
    ///
    /// This method requires exclusive mutable access, so it doesn't need atomic operations.
    #[inline(always)]
    pub fn peek_mut(&mut self) -> ComponentId {
        unsafe { Self::force_cast(*self.next.get_mut()) }
    }

    /// Allocates and returns the next available ID.
    ///
    /// This method requires exclusive mutable access, so it doesn't need atomic operations.
    #[inline]
    pub fn next_mut(&mut self) -> ComponentId {
        let next = self.next.get_mut();
        assert!(*next < u32::MAX, "too many components");
        let result = unsafe { Self::force_cast(*next) };
        *next += 1;
        result
    }
}

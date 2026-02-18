#![allow(clippy::new_without_default, reason = "internal type")]

use core::fmt::{Debug, Display};
use core::hash::Hash;
use core::num::NonZeroU32;
use core::sync::atomic::Ordering;

use vc_os::sync::atomic::AtomicU32;

// -----------------------------------------------------------------------------
// ComponentId

/// Unique identifier for a `Component` type.
///
/// `ComponentId` is only valid for a given `World`,
/// and is not globally unique.
#[derive(Clone, Copy, PartialOrd, Ord)]
#[repr(transparent)]
pub struct ComponentId(NonZeroU32);

impl ComponentId {
    const _STATIC_ASSERT_: () = const {
        let inner = NonZeroU32::new(123456).unwrap();
        assert!(ComponentId(inner).index_u32() == 123456);
    };

    /// Convert `ComponentId` to u32.
    #[inline(always)]
    pub const fn index_u32(self) -> u32 {
        unsafe { core::mem::transmute(self) }
    }

    /// Convert `ComponentId` to usize.
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

impl Hash for ComponentId {
    #[inline(always)]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        state.write_u32(self.index_u32());
    }
}

impl Debug for ComponentId {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.index_u32(), f)
    }
}

impl Display for ComponentId {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.index_u32(), f)
    }
}

// -----------------------------------------------------------------------------
// CompIdAllocator

/// An allocator for `ComponentId` that starts allocation from `1`.
///
/// # Panics
/// Panics if the allocated ID would exceed or equal `u32::MAX`.
pub struct CompIdAllocator {
    next: AtomicU32,
}

impl CompIdAllocator {
    #[inline(always)]
    const unsafe fn force_cast(id: u32) -> ComponentId {
        unsafe { core::mem::transmute(id) }
    }

    /// Creates a new `ComponentIdAllocator` that starts allocating IDs from `1`.
    #[inline(always)]
    pub(crate) const fn new() -> Self {
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

    pub fn alloc(&self) -> ComponentId {
        let next = self.next.fetch_add(1, Ordering::Relaxed);
        assert!(next < u32::MAX, "too many components");
        unsafe { Self::force_cast(next) }
    }

    pub fn alloc_mut(&mut self) -> ComponentId {
        let next = self.next.get_mut();
        assert!(*next < u32::MAX, "too many components");
        let result = unsafe { Self::force_cast(*next) };
        *next += 1;
        result
    }
}

impl Debug for CompIdAllocator {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CompIdAllocator")
            .field("allocated", &self.count())
            .finish()
    }
}

// -----------------------------------------------------------------------------
// Tests

#[cfg(test)]
mod tests {
    use super::CompIdAllocator;

    #[test]
    fn alloc() {
        let mut allocator = CompIdAllocator::new();
        assert_eq!(allocator.alloc().index_u32(), 1);
        assert_eq!(allocator.alloc().index_u32(), 2);
        assert_eq!(allocator.alloc_mut().index_u32(), 3);
        assert_eq!(allocator.alloc_mut().index_u32(), 4);
        assert_eq!(allocator.alloc().index_u32(), 5);
        assert_eq!(allocator.alloc_mut().index_u32(), 6);
    }
}

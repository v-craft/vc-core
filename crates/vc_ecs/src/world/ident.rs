use core::fmt::{Debug, Display};
use vc_os::sync::atomic::{AtomicU64, Ordering};

// -----------------------------------------------------------------------------
// WorldId

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct WorldId(u64);

impl WorldId {
    pub const fn new(id: u64) -> Self {
        Self(id)
    }
}

impl Debug for WorldId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for WorldId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

// -----------------------------------------------------------------------------
// WorldIdAllocator

#[derive(Debug, Default)]
pub struct WorldIdAllocator {
    next: AtomicU64,
}

impl WorldIdAllocator {
    pub const fn new() -> Self {
        Self {
            next: AtomicU64::new(0),
        }
    }

    pub fn alloc(&self) -> WorldId {
        let next = self.next.fetch_add(1, Ordering::Relaxed);
        assert!(next < u64::MAX, "too many worlds");
        WorldId(next)
    }
}

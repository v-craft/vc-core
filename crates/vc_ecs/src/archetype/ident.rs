use core::fmt::{Debug, Display};
use core::hash::Hash;

use nonmax::NonMaxU32;

// -----------------------------------------------------------------------------
// ArchetypeId

#[derive(Copy, Clone, PartialOrd, Ord)]
#[repr(transparent)]
pub struct ArchetypeId(NonMaxU32);

impl ArchetypeId {
    pub const EMPTY: ArchetypeId = ArchetypeId(NonMaxU32::ZERO);

    #[inline(always)]
    pub(crate) const fn new(index: NonMaxU32) -> Self {
        Self(index)
    }

    #[inline(always)]
    pub const fn index_u32(self) -> u32 {
        self.0.get()
    }

    #[inline(always)]
    pub const fn index(self) -> usize {
        self.0.get() as usize
    }
}

impl Debug for ArchetypeId {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.0.get(), f)
    }
}

impl Display for ArchetypeId {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.0.get(), f)
    }
}

impl Hash for ArchetypeId {
    #[inline(always)]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        // we do not use underlying value here,
        // then `SparseHash` is faster.
        state.write_u32(self.0.get());
    }
}

impl PartialEq for ArchetypeId {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        use core::mem::transmute_copy;
        unsafe { transmute_copy::<Self, u32>(self) == transmute_copy::<Self, u32>(other) }
    }
}

impl Eq for ArchetypeId {}

// -----------------------------------------------------------------------------
// Tests

#[cfg(test)]
mod tests {
    use super::ArchetypeId;
    use nonmax::NonMaxU32;

    #[test]
    fn index() {
        let inner = NonMaxU32::new(123456_u32).unwrap();
        assert_eq!(ArchetypeId::new(inner).index_u32(), 123456_u32);
        assert_eq!(ArchetypeId::new(inner).index(), 123456_usize);
    }

    #[test]
    fn eq() {
        let inner1 = NonMaxU32::new(12345).unwrap();
        let inner2 = NonMaxU32::new(54321).unwrap();
        assert_eq!(ArchetypeId::new(inner1), ArchetypeId::new(inner1));
        assert_eq!(ArchetypeId::new(inner2), ArchetypeId::new(inner2));
        assert_ne!(ArchetypeId::new(inner1), ArchetypeId::new(inner2));
    }
}

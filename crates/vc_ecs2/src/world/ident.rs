use core::fmt::{Debug, Display};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct WorldId(u64);

impl WorldId {
    pub const fn with(id: u64) -> Self {
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

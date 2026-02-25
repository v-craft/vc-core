use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::ptr::NonNull;

use crate::world::World;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WorldMode {
    ReadOnly = 0,
    DataMut = 1,
    FullMut = 2,
}

impl WorldMode {
    pub const fn merge(self, other: Self) -> Self {
        if self as u8 >= other as u8 {
            self
        } else {
            other
        }
    }
}

#[derive(Clone, Copy)]
pub struct UnsafeWorld<'a> {
    world: NonNull<World>,
    _marker: PhantomData<&'a UnsafeCell<World>>,
}

impl<'a> UnsafeWorld<'a> {
    /// # Safety
    /// - Read only.
    /// - The caller ensures concurrency safety
    pub unsafe fn read_only(self) -> &'a World {
        unsafe { &*self.world.as_ptr() }
    }

    /// # Safety
    /// - The caller ensures concurrency safety
    /// - Only the data can be changed:
    ///   - entities/resources cannot be added or deleted
    ///   - Cannot register a new type or allocate any ID.
    pub unsafe fn data_mut(self) -> &'a mut World {
        unsafe { &mut *self.world.as_ptr() }
    }

    /// # Safety
    /// - There are no other borrowings.
    pub unsafe fn full_mut(self) -> &'a mut World {
        unsafe { &mut *self.world.as_ptr() }
    }
}

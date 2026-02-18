use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::ptr::NonNull;

use crate::world::World;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldMode {
    Read = 0,
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

#[derive(Copy, Clone)]
pub struct UnsafeWorld<'a> {
    world: NonNull<World>,
    _marker: PhantomData<&'a UnsafeCell<World>>,
}

impl<'a> UnsafeWorld<'a> {
    pub unsafe fn read(self) -> &'a World {
        unsafe { &*self.world.as_ptr() }
    }

    pub unsafe fn data_mut(self) -> &'a mut World {
        unsafe { &mut *self.world.as_ptr() }
    }

    pub unsafe fn full_mut(self) -> &'a mut World {
        unsafe { &mut *self.world.as_ptr() }
    }
}

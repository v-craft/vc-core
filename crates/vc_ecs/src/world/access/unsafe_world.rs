use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::ptr::NonNull;

use crate::world::World;

#[derive(Clone, Copy)]
pub struct UnsafeWorld<'a> {
    world: NonNull<World>,
    _marker: PhantomData<&'a UnsafeCell<World>>,
}

// SAFETY: `&World` and `&mut World` are both `Send`
unsafe impl Send for UnsafeWorld<'_> {}
// SAFETY: `&World` and `&mut World` are both `Sync`
unsafe impl Sync for UnsafeWorld<'_> {}

// impl<'a> From<&'a World> for UnsafeWorld<'a> {
//     fn from(value: &'a World) -> Self {
//         UnsafeWorld {
//             world: NonNull::from_ref(value),
//             _marker: PhantomData,
//         }
//     }
// }

// impl<'a> From<&'a mut World> for UnsafeWorld<'a> {
//     fn from(value: &'a mut World) -> Self {
//         UnsafeWorld {
//             world: NonNull::from_mut(value),
//             _marker: PhantomData,
//         }
//     }
// }

impl World {
    pub const fn unsafe_world(&self) -> UnsafeWorld<'_> {
        UnsafeWorld {
            world: NonNull::from_ref(self),
            _marker: PhantomData,
        }
    }
}

impl<'a> UnsafeWorld<'a> {
    /// # Safety
    /// - Read only.
    /// - The caller ensures concurrency safety
    #[inline(always)]
    pub const unsafe fn read_only(self) -> &'a World {
        unsafe { &*self.world.as_ptr() }
    }

    /// # Safety
    /// - The caller ensures concurrency safety
    /// - Only the data can be changed:
    ///   - entities/resources cannot be added or deleted
    ///   - Cannot register a new type or allocate any ID.
    #[inline(always)]
    pub const unsafe fn data_mut(self) -> &'a mut World {
        unsafe { &mut *self.world.as_ptr() }
    }

    /// # Safety
    /// - There are no other borrowings.
    #[inline(always)]
    pub const unsafe fn full_mut(self) -> &'a mut World {
        unsafe { &mut *self.world.as_ptr() }
    }

    #[inline(always)]
    pub const fn into_inner(self) -> NonNull<World> {
        self.world
    }
}

use core::any::TypeId;

use vc_ptr::OwningPtr;

use crate::borrow::{ResMut, UntypedMut};
use crate::resource::{Resource, ResourceId};
use crate::tick::Tick;
use crate::world::World;

impl World {
    #[inline(always)]
    pub fn insert_resource<T: Resource>(&mut self, value: T) -> ResMut<'_, T> {
        #[inline(never)]
        fn insert_internal<'a, 'b>(
            this: &'a mut World,
            value: OwningPtr<'b>,
            id: ResourceId,
        ) -> UntypedMut<'a> {
            unsafe {
                let data = this.storages.res_set.get_unchecked_mut(id);
                let tick = Tick::new(*this.this_run.get_mut());
                data.insert(value, tick);
                data.assert_get_mut(this.last_run, tick)
            }
        }

        // let id = self.register_resource::<T>();
        let id = self.resources.register::<T>();
        // self.prepare_resource(id);
        let info = unsafe { self.resources.get_unchecked(id) };
        self.storages.prepare_resource(info);

        vc_ptr::into_owning!(value);
        unsafe { insert_internal(self, value, id).into_res::<T>() }
    }

    #[inline]
    pub fn remove_resource<T: Resource>(&mut self) -> Option<T> {
        if let Some(id) = self.resources.get_id(TypeId::of::<T>())
            && let Some(data) = self.storages.res_set.get_mut(id)
            && let Some(ptr) = unsafe { data.remove() }
        {
            ptr.debug_assert_aligned::<T>();
            Some(unsafe { ptr.read::<T>() })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use core::num::NonZeroU64;

    use crate::resource::Resource;
    use crate::world::{World, WorldId};

    #[derive(Debug, PartialEq, Eq)]
    struct Foo;

    #[derive(Debug, PartialEq, Eq)]
    struct Bar(u64);

    unsafe impl Resource for Foo {
        const IS_SEND: bool = true;
        const MUTABLE: bool = true;
    }

    unsafe impl Resource for Bar {
        const IS_SEND: bool = true;
        const MUTABLE: bool = true;
    }

    #[test]
    fn insert_basic() {
        let world_id = WorldId::new(NonZeroU64::new(1).unwrap());
        let mut world = World::new(world_id);
        assert_eq!(*world.insert_resource(Foo).into_inner(), Foo);
        assert_eq!(*world.insert_resource(Bar(234)).into_inner(), Bar(234));

        // std::eprintln!(
        //     "{:?} | {:?}",
        //     get_res::<Foo>(&world),
        //     get_non_send::<Bar>(&world),
        // );
    }
}

use core::any::TypeId;

use vc_ptr::OwningPtr;

use super::World;
use crate::borrow::{NonSendMut, ResMut, UntypedMut};
use crate::component::{ComponentId, NonSendResource, Resource};
use crate::tick::Tick;

impl World {
    #[inline(always)]
    pub fn insert_resource<T: Resource>(&mut self, value: T) -> ResMut<'_, T> {
        let id = self
            .components
            .register_resource::<T>(&mut self.compid_allocator);

        vc_ptr::into_owning!(value);

        unsafe { self.insert_resouce_internal(value, id).into_res::<T>() }
    }

    #[inline(never)]
    fn insert_resouce_internal(&mut self, value: OwningPtr<'_>, id: ComponentId) -> UntypedMut<'_> {
        let info = unsafe { self.components.get(id) };
        self.storages.resources.prepare(info);

        unsafe {
            let data = self.storages.resources.get_mut(id);
            let tick = Tick::new(*self.now_tick.get_mut());
            data.set_data(value, tick);
            data.get_mut(self.last_change, Tick::new(*self.now_tick.get_mut()))
        }
    }

    pub fn remove_resource<T: Resource>(&mut self) {
        if let Some(cid) = self.components.get_resource_id(TypeId::of::<T>()) {
            let data = unsafe { self.storages.resources.get_mut(cid) };
            data.drop_data();
        }
    }

    #[inline(always)]
    pub fn insert_non_send<T: NonSendResource>(&mut self, value: T) -> NonSendMut<'_, T> {
        let id = self
            .components
            .register_non_send::<T>(&mut self.compid_allocator);

        vc_ptr::into_owning!(value);

        unsafe {
            self.insert_non_send_internal(value, id)
                .into_non_send::<T>()
        }
    }

    #[inline(never)]
    fn insert_non_send_internal(
        &mut self,
        value: OwningPtr<'_>,
        id: ComponentId,
    ) -> UntypedMut<'_> {
        let info = unsafe { self.components.get(id) };
        self.storages.non_sends.prepare(info);

        unsafe {
            let data = self.storages.non_sends.get_mut(id);
            let tick = Tick::new(*self.now_tick.get_mut());
            data.set_data(value, tick);
            data.get_mut(self.last_change, Tick::new(*self.now_tick.get_mut()))
        }
    }

    pub fn remove_non_send<T: NonSendResource>(&mut self) {
        if let Some(cid) = self.components.get_non_send_id(TypeId::of::<T>()) {
            let data = unsafe { self.storages.non_sends.get_mut(cid) };
            data.drop_data();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::component::{NonSendResource, Resource};
    use crate::world::{World, WorldId};

    #[derive(Debug, PartialEq, Eq)]
    struct Foo;

    #[derive(Debug, PartialEq, Eq)]
    struct Bar(u64);

    impl Resource for Foo {}
    impl NonSendResource for Bar {}
    #[test]
    fn insert_basic() {
        let mut world = World::new(WorldId::new(1));
        assert_eq!(*world.insert_resource(Foo).into_inner(), Foo);
        assert_eq!(*world.insert_non_send(Bar(234)).into_inner(), Bar(234));

        // std::eprintln!(
        //     "{:?} | {:?}",
        //     get_res::<Foo>(&world),
        //     get_non_send::<Bar>(&world),
        // );
    }
}

use core::any::TypeId;

use vc_ptr::OwningPtr;

use super::World;
use crate::component::{ComponentId, NonSendResource, Resource};

impl World {
    #[inline(always)]
    pub fn insert_resource<T: Resource>(&mut self, value: T) {
        let id = self
            .components
            .register_resource::<T>(&mut self.compid_allocator);

        vc_ptr::into_owning!(value);

        self.insert_resouce_internal(value, id);
    }

    #[inline(never)]
    fn insert_resouce_internal(&mut self, value: OwningPtr<'_>, id: ComponentId) {
        let info = unsafe { self.components.get(id) };
        self.storages.resources.prepare(info);

        unsafe {
            let data = self.storages.resources.get_mut(id);
            data.set_data(value, self.now);
        }
    }

    pub fn remove_resource<T: Resource>(&mut self) {
        if let Some(cid) = self.components.get_resource_id(TypeId::of::<T>()) {
            let data = unsafe { self.storages.resources.get_mut(cid) };
            data.drop_data();
        }
    }

    #[inline(always)]
    pub fn insert_non_send<T: NonSendResource>(&mut self, value: T) {
        let id = self
            .components
            .register_non_send::<T>(&mut self.compid_allocator);

        vc_ptr::into_owning!(value);

        self.insert_non_send_internal(value, id);
    }

    #[inline(never)]
    fn insert_non_send_internal(&mut self, value: OwningPtr<'_>, id: ComponentId) {
        let info = unsafe { self.components.get(id) };
        self.storages.non_sends.prepare(info);

        unsafe {
            let data = self.storages.non_sends.get_mut(id);
            data.set_data(value, self.now);
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
    use core::any::TypeId;

    use crate::borrow::{NonSend, Res};
    use crate::component::{NonSendResource, Resource};
    use crate::world::{World, WorldId};

    #[derive(Debug, PartialEq, Eq)]
    struct Foo;

    #[derive(Debug, PartialEq, Eq)]
    struct Bar(u64);

    impl Resource for Foo {}
    impl NonSendResource for Bar {}

    fn get_res<T: Resource>(world: &World) -> Res<'_, T> {
        let component_id = world.components.get_resource_id(TypeId::of::<T>()).unwrap();
        unsafe {
            world
                .storages
                .resources
                .get(component_id)
                .get_ref(world.now, world.now)
                .into_res::<T>()
        }
    }

    fn get_non_send<T: NonSendResource>(world: &World) -> NonSend<'_, T> {
        let component_id = world.components.get_non_send_id(TypeId::of::<T>()).unwrap();
        unsafe {
            world
                .storages
                .non_sends
                .get(component_id)
                .get_ref(world.now, world.now)
                .into_non_send::<T>()
        }
    }

    #[test]
    fn insert_basic() {
        let mut world = World::new(WorldId::new(1));
        world.insert_resource(Foo);
        world.insert_non_send(Bar(234));

        assert_eq!(get_res::<Foo>(&world).into_inner(), &Foo);
        assert_eq!(get_non_send::<Bar>(&world).into_inner(), &Bar(234));

        std::eprintln!(
            "{:?} | {:?}",
            get_res::<Foo>(&world),
            get_non_send::<Bar>(&world),
        );
    }
}

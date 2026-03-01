use core::any::TypeId;
use core::sync::atomic::Ordering;

use vc_ptr::{OwningPtr, PtrMut};

use crate::borrow::{NonSyncMut, NonSyncRef, ResMut, ResRef};
use crate::resource::{Resource, ResourceId};
use crate::tick::Tick;
use crate::utils::DebugCheckedUnwrap;
use crate::world::World;

#[inline(never)]
fn insert_internal<'a, 'b>(
    this: &'a mut World,
    value: OwningPtr<'b>,
    id: ResourceId,
) -> PtrMut<'a> {
    unsafe {
        let data = this.storages.res.get_unchecked_mut(id);
        let tick = Tick::new(*this.this_run.get_mut());
        data.insert(value, tick);
        data.get_data_mut().debug_checked_unwrap()
    }
}

impl World {
    pub fn insert_resource<T: Resource + Send>(&mut self, value: T) -> &mut T {
        // let id = self.register_resource::<T>();
        let id = self.resources.register::<T>();
        // self.prepare_resource(id);
        let info = unsafe { self.resources.get_unchecked(id) };
        self.storages.prepare_resource(info);

        vc_ptr::into_owning!(value);
        unsafe { insert_internal(self, value, id).consume::<T>() }
    }

    pub fn remove_resource<T: Resource + Send>(&mut self) -> Option<T> {
        if let Some(id) = self.resources.get_id(TypeId::of::<T>())
            && let Some(data) = self.storages.res.get_mut(id)
            && let Some(ptr) = unsafe { data.remove() }
        {
            ptr.debug_assert_aligned::<T>();
            Some(unsafe { ptr.read::<T>() })
        } else {
            None
        }
    }

    pub fn get_resource<T: Resource + Sync>(&self) -> Option<&T> {
        if let Some(id) = self.resources.get_id(TypeId::of::<T>())
            && let Some(data) = self.storages.res.get(id)
            && let Some(ptr) = data.get_data()
        {
            ptr.debug_assert_aligned::<T>();
            Some(unsafe { ptr.as_ref::<T>() })
        } else {
            None
        }
    }

    pub fn get_resource_ref<T: Resource + Sync>(&self) -> Option<ResRef<'_, T>> {
        if let Some(id) = self.resources.get_id(TypeId::of::<T>())
            && let Some(data) = self.storages.res.get(id)
        {
            let last_run = self.last_run;
            let this_run = Tick::new(self.this_run.load(Ordering::Relaxed));
            let ptr = data.get_ref(last_run, this_run)?;
            Some(unsafe { ptr.into_res::<T>() })
        } else {
            None
        }
    }

    pub fn get_resource_mut<T: Resource + Sync>(&mut self) -> Option<ResMut<'_, T>> {
        if let Some(id) = self.resources.get_id(TypeId::of::<T>())
            && let Some(data) = self.storages.res.get_mut(id)
        {
            let last_run = self.last_run;
            let this_run = Tick::new(*self.this_run.get_mut());
            let ptr = data.get_mut(last_run, this_run)?;
            Some(unsafe { ptr.into_res::<T>() })
        } else {
            None
        }
    }

    pub fn insert_non_send<T: Resource>(&mut self, value: T) -> &mut T {
        assert! {
            self.thread_hash == crate::utils::thread_hash(),
            "!Send Resource can only be inserted/removed on the main thread.",
        }

        // let id = self.register_resource::<T>();
        let id = self.resources.register::<T>();
        // self.prepare_resource(id);
        let info = unsafe { self.resources.get_unchecked(id) };
        self.storages.prepare_resource(info);

        vc_ptr::into_owning!(value);
        unsafe { insert_internal(self, value, id).consume::<T>() }
    }

    pub fn remove_non_send<T: Resource>(&mut self) -> Option<T> {
        assert! {
            self.thread_hash == crate::utils::thread_hash(),
            "!Send Resource can only be inserted/removed on the main thread.",
        }

        if let Some(id) = self.resources.get_id(TypeId::of::<T>())
            && let Some(data) = self.storages.res.get_mut(id)
            && let Some(ptr) = unsafe { data.remove() }
        {
            ptr.debug_assert_aligned::<T>();
            Some(unsafe { ptr.read::<T>() })
        } else {
            None
        }
    }

    pub fn get_non_sync<T: Resource>(&mut self) -> Option<&T> {
        assert! {
            self.thread_hash == crate::utils::thread_hash(),
            "!Sync Resource can only be borrowed on the main thread.",
        }

        if let Some(id) = self.resources.get_id(TypeId::of::<T>())
            && let Some(data) = self.storages.res.get(id)
            && let Some(ptr) = data.get_data()
        {
            ptr.debug_assert_aligned::<T>();
            Some(unsafe { ptr.as_ref::<T>() })
        } else {
            None
        }
    }

    pub fn get_non_sync_ref<T: Resource>(&self) -> Option<NonSyncRef<'_, T>> {
        assert! {
            self.thread_hash == crate::utils::thread_hash(),
            "!Sync Resource can only be borrowed on the main thread.",
        }

        if let Some(id) = self.resources.get_id(TypeId::of::<T>())
            && let Some(data) = self.storages.res.get(id)
        {
            let last_run = self.last_run;
            let this_run = Tick::new(self.this_run.load(Ordering::Relaxed));
            let ptr = data.get_ref(last_run, this_run)?;
            Some(unsafe { ptr.into_non_sync::<T>() })
        } else {
            None
        }
    }

    pub fn get_non_sync_mut<T: Resource>(&mut self) -> Option<NonSyncMut<'_, T>> {
        assert! {
            self.thread_hash == crate::utils::thread_hash(),
            "!Sync Resource can only be borrowed on the main thread.",
        }

        if let Some(id) = self.resources.get_id(TypeId::of::<T>())
            && let Some(data) = self.storages.res.get_mut(id)
        {
            let last_run = self.last_run;
            let this_run = Tick::new(*self.this_run.get_mut());
            let ptr = data.get_mut(last_run, this_run)?;
            Some(unsafe { ptr.into_non_sync::<T>() })
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

    unsafe impl Resource for Foo {}
    unsafe impl Resource for Bar {}

    #[test]
    fn insert_basic() {
        let world_id = WorldId::new(NonZeroU64::new(1).unwrap());
        let mut world = World::new(world_id);
        assert_eq!(*world.insert_resource(Foo), Foo);
        assert_eq!(*world.insert_resource(Bar(234)), Bar(234));
        assert_eq!(world.remove_resource::<Foo>(), Some(Foo));
        assert_eq!(world.get_resource::<Foo>(), None);
        assert_eq!(world.get_resource::<Bar>(), Some(&Bar(234)));

        // std::eprintln!(
        //     "{:?} | {:?}",
        //     get_res::<Foo>(&world),
        //     get_non_send::<Bar>(&world),
        // );
    }
}

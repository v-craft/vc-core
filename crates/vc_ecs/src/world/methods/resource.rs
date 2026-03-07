use core::any::TypeId;
use core::sync::atomic::Ordering;

use vc_ptr::{OwningPtr, PtrMut};

use crate::borrow::{NonSendMut, NonSendRef, ResMut, ResRef};
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
        this.prepare_resource(id);
        let data = this.storages.res.get_unchecked_mut(id);
        let tick = Tick::new(*this.this_run.get_mut());
        data.insert_untyped(value, tick);
        data.get_data_mut().debug_checked_unwrap()
    }
}

impl World {
    /// Inserts or replaces a `Send` resource and returns a mutable reference to it.
    ///
    /// The resource is registered by type on first use. Once inserted, it can be
    /// accessed from systems through [`crate::borrow::Res`],
    /// [`crate::borrow::ResRef`], or [`crate::borrow::ResMut`].
    ///
    /// # Examples
    ///
    /// ```
    /// use vc_ecs::resource::Resource;
    /// use vc_ecs::world::{World, WorldIdAllocator};
    ///
    /// static IDS: WorldIdAllocator = WorldIdAllocator::new();
    ///
    /// #[derive(Debug, PartialEq, Eq)]
    /// struct Counter(u32);
    ///
    /// unsafe impl Resource for Counter {}
    ///
    /// let mut world = World::new(IDS.alloc());
    /// let counter = world.insert_resource(Counter(1));
    /// counter.0 += 1;
    ///
    /// assert_eq!(world.get_resource::<Counter>(), Some(&Counter(2)));
    /// ```
    pub fn insert_resource<T: Resource + Send>(&mut self, value: T) -> &mut T {
        // let id = self.register_resource::<T>();
        let id = self.resources.register::<T>();
        vc_ptr::into_owning!(value);
        unsafe { insert_internal(self, value, id).consume::<T>() }
    }

    /// Removes and returns a `Send` resource if it exists.
    pub fn remove_resource<T: Resource + Send>(&mut self) -> Option<T> {
        if let Some(id) = self.resources.get_id(TypeId::of::<T>())
            && let Some(data) = self.storages.res.get_mut(id)
        {
            unsafe { data.remove() }
        } else {
            None
        }
    }

    /// Drop a `Send` resource if it exists.
    ///
    /// This will be faster than removing, as there is no need to return data.
    pub fn drop_resource<T: Resource + Send>(&mut self) {
        if let Some(id) = self.resources.get_id(TypeId::of::<T>())
            && let Some(data) = self.storages.res.get_mut(id)
        {
            unsafe { data.drop_in_place::<T>() }
        }
    }

    /// Returns a shared reference to a `Send + Sync` resource without change detection.
    ///
    /// This mirrors the behavior of the [`crate::borrow::Res`] system parameter.
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

    /// Returns a shared resource borrow with change detection.
    ///
    /// This mirrors the behavior of the [`crate::borrow::ResRef`] system parameter.
    pub fn get_resource_ref<T: Resource + Sync>(&self) -> Option<ResRef<'_, T>> {
        if let Some(id) = self.resources.get_id(TypeId::of::<T>())
            && let Some(data) = self.storages.res.get(id)
        {
            let last_run = self.last_run;
            let this_run = Tick::new(self.this_run.load(Ordering::Relaxed));
            let ptr = data.get_ref(last_run, this_run)?;
            Some(unsafe { ptr.into_resource::<T>() })
        } else {
            None
        }
    }

    /// Returns an exclusive resource borrow with change detection.
    ///
    /// This mirrors the behavior of the [`crate::borrow::ResMut`] system parameter.
    pub fn get_resource_mut<T: Resource + Send>(&mut self) -> Option<ResMut<'_, T>> {
        if let Some(id) = self.resources.get_id(TypeId::of::<T>())
            && let Some(data) = self.storages.res.get_mut(id)
        {
            let last_run = self.last_run;
            let this_run = Tick::new(*self.this_run.get_mut());
            let ptr = data.get_mut(last_run, this_run)?;
            Some(unsafe { ptr.into_resource::<T>() })
        } else {
            None
        }
    }

    /// Inserts or replaces a main-thread resource and returns a mutable reference to it.
    ///
    /// Unlike [`World::insert_resource`], this accepts `!Sync` values. Access to the
    /// resource is restricted to the thread that created the world.
    ///
    /// # Panics
    ///
    /// Panics if called from a thread other than the world's main thread.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::cell::Cell;
    /// use vc_ecs::resource::Resource;
    /// use vc_ecs::world::{World, WorldIdAllocator};
    ///
    /// static IDS: WorldIdAllocator = WorldIdAllocator::new();
    ///
    /// struct LocalCounter(Cell<u32>);
    ///
    /// unsafe impl Resource for LocalCounter {}
    ///
    /// let mut world = World::new(IDS.alloc());
    /// world.insert_non_send(LocalCounter(Cell::new(3)));
    /// assert_eq!(world.get_non_send::<LocalCounter>().unwrap().0.get(), 3);
    /// ```
    pub fn insert_non_send<T: Resource>(&mut self, value: T) -> &mut T {
        assert! {
            self.thread_hash == crate::utils::thread_hash(),
            "!Send Resource can only be inserted/removed on the main thread.",
        }

        // let id = self.register_resource::<T>();
        let id = self.resources.register::<T>();

        vc_ptr::into_owning!(value);
        unsafe { insert_internal(self, value, id).consume::<T>() }
    }

    /// Removes and returns a main-thread resource if it exists.
    ///
    /// # Panics
    ///
    /// Panics if called from a thread other than the world's main thread.
    pub fn remove_non_send<T: Resource>(&mut self) -> Option<T> {
        assert! {
            self.thread_hash == crate::utils::thread_hash(),
            "!Send Resource can only be inserted/removed on the main thread.",
        }

        if let Some(id) = self.resources.get_id(TypeId::of::<T>())
            && let Some(data) = self.storages.res.get_mut(id)
        {
            unsafe { data.remove() }
        } else {
            None
        }
    }

    /// Drop a resource if it exists.
    ///
    /// This will be faster than removing, as there is no need to return data.
    ///
    /// # Panics
    ///
    /// Panics if called from a thread other than the world's main thread.
    pub fn drop_non_send<T: Resource>(&mut self) {
        assert! {
            self.thread_hash == crate::utils::thread_hash(),
            "!Send Resource can only be inserted/removed on the main thread.",
        }

        if let Some(id) = self.resources.get_id(TypeId::of::<T>())
            && let Some(data) = self.storages.res.get_mut(id)
        {
            unsafe { data.drop_in_place::<T>() }
        }
    }

    /// Returns a shared reference to a main-thread resource without change detection.
    ///
    /// # Panics
    ///
    /// Panics if called from a thread other than the world's main thread.
    pub fn get_non_send<T: Resource>(&mut self) -> Option<&T> {
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

    /// Returns a shared main-thread resource borrow with change detection.
    ///
    /// This mirrors the behavior of the [`crate::borrow::NonSendRef`] system parameter.
    ///
    /// # Panics
    ///
    /// Panics if called from a thread other than the world's main thread.
    pub fn get_non_send_ref<T: Resource>(&self) -> Option<NonSendRef<'_, T>> {
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
            Some(unsafe { ptr.into_non_send::<T>() })
        } else {
            None
        }
    }

    /// Returns an exclusive main-thread resource borrow with change detection.
    ///
    /// This mirrors the behavior of the [`crate::borrow::NonSendMut`] system parameter.
    ///
    /// # Panics
    ///
    /// Panics if called from a thread other than the world's main thread.
    pub fn get_non_send_mut<T: Resource>(&mut self) -> Option<NonSendMut<'_, T>> {
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
            Some(unsafe { ptr.into_non_send::<T>() })
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

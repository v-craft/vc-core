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
    /// accessed from systems through [`Res`], [`ResRef`], or [`ResMut`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use vc_ecs::resource::Resource;
    /// # use vc_ecs::world::{World, WorldIdAllocator};
    /// # let mut world = World::new(WorldIdAllocator::new().alloc());
    /// #[derive(Debug, PartialEq, Eq)]
    /// struct Counter(u64);
    /// unsafe impl Resource for Counter {}
    ///
    /// assert_eq!(*world.insert_resource(Counter(1)), Counter(1));
    /// assert_eq!(*world.insert_resource(Counter(2)), Counter(2));
    /// assert_eq!(world.get_resource::<Counter>(), Some(&Counter(2)));
    /// ```
    ///
    /// [`Res`]: crate::borrow::Res
    pub fn insert_resource<T: Resource + Send>(&mut self, value: T) -> &mut T {
        let id = self.resources.register::<T>();
        vc_ptr::into_owning!(value);
        unsafe { insert_internal(self, value, id).consume::<T>() }
    }

    /// Removes and returns a `Send` resource if it exists.
    ///
    /// # Examples
    ///
    /// ```
    /// # use vc_ecs::resource::Resource;
    /// # use vc_ecs::world::{World, WorldIdAllocator};
    /// # let mut world = World::new(WorldIdAllocator::new().alloc());
    /// #[derive(Debug, PartialEq, Eq)]
    /// struct Foo;
    /// unsafe impl Resource for Foo {}
    ///
    /// world.insert_resource(Foo);
    /// assert_eq!(world.remove_resource::<Foo>(), Some(Foo));
    /// assert_eq!(world.remove_resource::<Foo>(), None);
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```
    /// # use vc_ecs::resource::Resource;
    /// # use vc_ecs::world::{World, WorldIdAllocator};
    /// # let mut world = World::new(WorldIdAllocator::new().alloc());
    /// #[derive(Debug)]
    /// struct Temp;
    /// unsafe impl Resource for Temp {}
    ///
    /// world.insert_resource(Temp);
    /// world.drop_resource::<Temp>();
    /// assert!(world.get_resource::<Temp>().is_none());
    /// ```
    pub fn drop_resource<T: Resource + Send>(&mut self) {
        if let Some(id) = self.resources.get_id(TypeId::of::<T>())
            && let Some(data) = self.storages.res.get_mut(id)
        {
            unsafe { data.drop_in_place::<T>() }
        }
    }

    /// Returns a shared reference to a `Send + Sync` resource without change detection.
    ///
    /// This mirrors the behavior of the [`Res`](crate::borrow::Res) system parameter.
    ///
    /// # Examples
    ///
    /// ```
    /// # use vc_ecs::resource::Resource;
    /// # use vc_ecs::world::{World, WorldIdAllocator};
    /// # let mut world = World::new(WorldIdAllocator::new().alloc());
    /// #[derive(Debug, PartialEq, Eq)]
    /// struct Bar(u64);
    /// unsafe impl Resource for Bar {}
    ///
    /// world.insert_resource(Bar(20));
    /// assert_eq!(world.get_resource::<Bar>(), Some(&Bar(20)));
    /// ```
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
    /// This mirrors the behavior of the [`ResRef`] system parameter.
    ///
    /// # Examples
    ///
    /// ```
    /// # use vc_ecs::resource::Resource;
    /// # use vc_ecs::tick::DetectChanges;
    /// # use vc_ecs::world::{World, WorldIdAllocator};
    /// # let mut world = World::new(WorldIdAllocator::new().alloc());
    /// #[derive(Debug, PartialEq, Eq)]
    /// struct Bar(u64);
    /// unsafe impl Resource for Bar {}
    ///
    /// world.insert_resource(Bar(20));
    /// let res = world.get_resource_ref::<Bar>().unwrap();
    /// assert!(res.is_added());
    /// assert!(res.is_changed());
    /// ```
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
    /// This mirrors the behavior of the [`ResMut`] system parameter.
    ///
    /// # Examples
    ///
    /// ```
    /// # use vc_ecs::resource::Resource;
    /// # use vc_ecs::tick::DetectChanges;
    /// # use vc_ecs::world::{World, WorldIdAllocator};
    /// # let mut world = World::new(WorldIdAllocator::new().alloc());
    /// #[derive(Debug, PartialEq, Eq)]
    /// struct Bar(u64);
    /// unsafe impl Resource for Bar {}
    ///
    /// world.insert_resource(Bar(20));
    /// let mut res = world.get_resource_mut::<Bar>().unwrap();
    /// *res = Bar(50);
    /// assert!(res.is_changed());
    /// ```
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
    /// # Examples
    ///
    /// ```
    /// # use vc_ecs::resource::Resource;
    /// # use vc_ecs::world::{World, WorldIdAllocator};
    /// # let mut world = World::new(WorldIdAllocator::new().alloc());
    /// #[derive(Debug, PartialEq, Eq)]
    /// struct LocalCache(u64);
    /// unsafe impl Resource for LocalCache {}
    ///
    /// world.insert_non_send(LocalCache(1));
    /// assert_eq!(world.get_non_send::<LocalCache>(), Some(&LocalCache(1)));
    /// ```
    ///
    /// # Panics
    /// Panics if called from a thread other than the world's main thread.
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
    /// # Examples
    ///
    /// ```
    /// # use vc_ecs::resource::Resource;
    /// # use vc_ecs::world::{World, WorldIdAllocator};
    /// # let mut world = World::new(WorldIdAllocator::new().alloc());
    /// #[derive(Debug, PartialEq, Eq)]
    /// struct Foo;
    /// unsafe impl Resource for Foo {}
    ///
    /// world.insert_non_send(Foo);
    /// assert_eq!(world.remove_non_send::<Foo>(), Some(Foo));
    /// assert_eq!(world.remove_non_send::<Foo>(), None);
    /// ```
    ///
    /// # Panics
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
    /// # Examples
    ///
    /// ```
    /// # use vc_ecs::resource::Resource;
    /// # use vc_ecs::world::{World, WorldIdAllocator};
    /// # let mut world = World::new(WorldIdAllocator::new().alloc());
    /// #[derive(Debug)]
    /// struct LocalTemp;
    /// unsafe impl Resource for LocalTemp {}
    ///
    /// world.insert_non_send(LocalTemp);
    /// world.drop_non_send::<LocalTemp>();
    /// assert!(world.get_non_send::<LocalTemp>().is_none());
    /// ```
    ///
    /// # Panics
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
    /// # Examples
    ///
    /// ```
    /// # use vc_ecs::resource::Resource;
    /// # use vc_ecs::world::{World, WorldIdAllocator};
    /// # let mut world = World::new(WorldIdAllocator::new().alloc());
    /// #[derive(Debug, PartialEq, Eq)]
    /// struct Bar(u64);
    /// unsafe impl Resource for Bar {}
    ///
    /// world.insert_non_send(Bar(99));
    /// assert_eq!(world.get_non_send::<Bar>(), Some(&Bar(99)));
    /// ```
    ///
    /// # Panics
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
    /// This mirrors the behavior of the [`NonSendRef`] system parameter.
    ///
    /// # Examples
    ///
    /// ```
    /// # use vc_ecs::resource::Resource;
    /// # use vc_ecs::tick::DetectChanges;
    /// # use vc_ecs::world::{World, WorldIdAllocator};
    /// # let mut world = World::new(WorldIdAllocator::new().alloc());
    /// #[derive(Debug, PartialEq, Eq)]
    /// struct Bar(u64);
    /// unsafe impl Resource for Bar {}
    ///
    /// world.insert_non_send(Bar(7));
    /// let res = world.get_non_send_ref::<Bar>().unwrap();
    /// assert!(res.is_added());
    /// assert!(res.is_changed());
    /// ```
    ///
    /// # Panics
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
    /// This mirrors the behavior of the [`NonSendMut`] system parameter.
    ///
    /// # Examples
    ///
    /// ```
    /// # use vc_ecs::resource::Resource;
    /// # use vc_ecs::tick::DetectChanges;
    /// # use vc_ecs::world::{World, WorldIdAllocator};
    /// # let mut world = World::new(WorldIdAllocator::new().alloc());
    /// #[derive(Debug, PartialEq, Eq)]
    /// struct Bar(u64);
    /// unsafe impl Resource for Bar {}
    ///
    /// world.insert_non_send(Bar(7));
    /// let mut res = world.get_non_send_mut::<Bar>().unwrap();
    /// *res = Bar(8);
    /// assert!(res.is_changed());
    /// ```
    ///
    /// # Panics
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
    use alloc::boxed::Box;
    use core::num::NonZeroU64;
    use core::sync::atomic::Ordering;
    use vc_os::sync::atomic::AtomicUsize;

    use crate::resource::Resource;
    use crate::tick::DetectChanges;
    use crate::world::{World, WorldId};

    #[derive(Debug, PartialEq, Eq)]
    struct Foo;

    #[derive(Debug, PartialEq, Eq)]
    struct Bar(u64);

    unsafe impl Resource for Foo {}
    unsafe impl Resource for Bar {}

    fn new_world() -> Box<World> {
        let world_id = WorldId::new(NonZeroU64::new(1).unwrap());
        World::new(world_id)
    }

    #[test]
    fn insert_basic() {
        let mut world = new_world();

        assert_eq!(*world.insert_resource(Foo), Foo);
        assert_eq!(*world.insert_resource(Bar(234)), Bar(234));

        assert_eq!(world.get_resource::<Foo>(), Some(&Foo));
        assert_eq!(world.remove_resource::<Foo>(), Some(Foo));
        assert_eq!(world.get_resource::<Foo>(), None);
        assert_eq!(world.get_non_send::<Foo>(), None);
        assert_eq!(world.remove_non_send::<Foo>(), None);
        assert_eq!(world.remove_resource::<Foo>(), None);

        assert_eq!(world.get_resource::<Bar>(), Some(&Bar(234)));
        assert_eq!(world.remove_non_send::<Bar>(), Some(Bar(234)));
        assert_eq!(world.get_resource::<Bar>(), None);
        assert_eq!(world.get_non_send::<Bar>(), None);
        assert_eq!(world.remove_non_send::<Bar>(), None);
        assert_eq!(world.remove_resource::<Bar>(), None);
    }

    #[test]
    fn insert_replace() {
        let mut world = new_world();

        world.insert_resource(Bar(100));
        assert_eq!(world.get_resource::<Bar>(), Some(&Bar(100)));
        assert_eq!(world.get_non_send::<Bar>(), Some(&Bar(100)));

        world.insert_resource(Bar(200));
        assert_eq!(world.get_resource::<Bar>(), Some(&Bar(200)));
        assert_eq!(world.get_non_send::<Bar>(), Some(&Bar(200)));

        world.insert_non_send(Bar(800));
        assert_eq!(world.get_resource::<Bar>(), Some(&Bar(800)));
        assert_eq!(world.get_non_send::<Bar>(), Some(&Bar(800)));
    }

    #[test]
    fn remove_nonexistent() {
        let mut world = new_world();
        assert!(world.remove_resource::<Foo>().is_none());
        assert!(world.get_resource::<Foo>().is_none());
        assert!(world.get_resource_ref::<Foo>().is_none());
        assert!(world.get_resource_mut::<Foo>().is_none());

        assert!(world.remove_non_send::<Foo>().is_none());
        assert!(world.get_non_send::<Foo>().is_none());
        assert!(world.get_non_send_ref::<Foo>().is_none());
        assert!(world.get_non_send_mut::<Foo>().is_none());
    }

    #[test]
    fn get_ref() {
        let mut world = new_world();
        world.insert_resource(Bar(20));

        let res_ref = world.get_resource_ref::<Bar>().unwrap();
        assert!(res_ref.is_changed());
        assert!(res_ref.is_added());

        world.update_tick();

        let res_ref = world.get_resource_ref::<Bar>().unwrap();
        assert_eq!(*res_ref, Bar(20));
        assert!(!res_ref.is_changed());
        assert!(!res_ref.is_added());

        let res_ref = world.get_non_send_ref::<Bar>().unwrap();
        assert_eq!(*res_ref, Bar(20));
        assert!(!res_ref.is_changed());
        assert!(!res_ref.is_added());
    }

    #[test]
    fn get_mut() {
        let mut world = new_world();
        world.insert_resource(Bar(20));

        let res_mut = world.get_resource_mut::<Bar>().unwrap();
        assert!(res_mut.is_changed());
        assert!(res_mut.is_added());

        world.update_tick();
        let mut res_mut = world.get_resource_mut::<Bar>().unwrap();
        assert_eq!(*res_mut, Bar(20));
        assert!(!res_mut.is_changed());
        assert!(!res_mut.is_added());

        *res_mut = Bar(100);
        assert!(res_mut.is_changed());
        assert!(!res_mut.is_added());

        world.update_tick();
        let mut res_mut = world.get_non_send_mut::<Bar>().unwrap();
        assert_eq!(*res_mut, Bar(100));
        assert!(!res_mut.is_changed());
        assert!(!res_mut.is_added());

        *res_mut = Bar(50);
        assert!(res_mut.is_changed());
        assert!(!res_mut.is_added());

        assert_eq!(world.get_non_send::<Bar>(), Some(&Bar(50)));
        assert_eq!(world.get_resource::<Bar>(), Some(&Bar(50)));
    }

    #[test]
    fn drop_resource() {
        static DROP_COUNTER: AtomicUsize = AtomicUsize::new(0);

        #[derive(Debug, PartialEq, Eq)]
        struct DropTracker(usize);
        unsafe impl Resource for DropTracker {}

        impl Drop for DropTracker {
            fn drop(&mut self) {
                DROP_COUNTER.fetch_add(self.0, Ordering::SeqCst);
            }
        }

        let mut world = new_world();

        // ------------------ Drop ----------------------
        DROP_COUNTER.store(0, Ordering::SeqCst);
        world.insert_resource(DropTracker(5));
        assert_eq!(DROP_COUNTER.load(Ordering::SeqCst), 0);
        world.drop_resource::<DropTracker>();
        assert_eq!(DROP_COUNTER.load(Ordering::SeqCst), 5);
        world.insert_non_send(DropTracker(5));
        assert_eq!(DROP_COUNTER.load(Ordering::SeqCst), 5);
        world.drop_non_send::<DropTracker>();
        assert_eq!(DROP_COUNTER.load(Ordering::SeqCst), 10);

        world.remove_non_send::<DropTracker>();
        world.remove_resource::<DropTracker>();
        world.drop_non_send::<DropTracker>();
        world.drop_resource::<DropTracker>();
        assert_eq!(DROP_COUNTER.load(Ordering::SeqCst), 10);

        // ----------------- Remove  ----------------------
        DROP_COUNTER.store(0, Ordering::SeqCst);

        world.insert_non_send(DropTracker(5));
        assert_eq!(DROP_COUNTER.load(Ordering::SeqCst), 0);
        world.remove_non_send::<DropTracker>();
        assert_eq!(DROP_COUNTER.load(Ordering::SeqCst), 5);

        world.insert_resource(DropTracker(5));
        assert_eq!(DROP_COUNTER.load(Ordering::SeqCst), 5);
        world.remove_resource::<DropTracker>();
        assert_eq!(DROP_COUNTER.load(Ordering::SeqCst), 10);

        world.remove_non_send::<DropTracker>();
        world.remove_resource::<DropTracker>();
        world.drop_non_send::<DropTracker>();
        world.drop_resource::<DropTracker>();
        assert_eq!(DROP_COUNTER.load(Ordering::SeqCst), 10);

        // ---------------- Overwrite ----------------------
        DROP_COUNTER.store(0, Ordering::SeqCst);

        world.insert_non_send(DropTracker(5));
        world.insert_resource(DropTracker(5));
        assert_eq!(DROP_COUNTER.load(Ordering::SeqCst), 5);
        world.drop_non_send::<DropTracker>();
        assert_eq!(DROP_COUNTER.load(Ordering::SeqCst), 10);

        world.remove_non_send::<DropTracker>();
        world.remove_resource::<DropTracker>();
        world.drop_non_send::<DropTracker>();
        world.drop_resource::<DropTracker>();
        assert_eq!(DROP_COUNTER.load(Ordering::SeqCst), 10);

        // ---------------- Overwrite ----------------------
        DROP_COUNTER.store(0, Ordering::SeqCst);

        for _ in 0..10 {
            world.insert_resource(DropTracker(1));
        }
        for _ in 0..10 {
            world.insert_non_send(DropTracker(1));
        }
        assert_eq!(DROP_COUNTER.load(Ordering::SeqCst), 19);

        world.remove_resource::<DropTracker>();
        assert_eq!(DROP_COUNTER.load(Ordering::SeqCst), 20);
    }
}

use super::{ReadOnlySystemParam, SystemParam};
use crate::borrow::{NonSend, NonSendMut, NonSendRef};
use crate::borrow::{Res, ResMut, ResRef};
use crate::resource::{Resource, ResourceId};
use crate::system::AccessTable;
use crate::tick::Tick;
use crate::utils::DebugName;
use crate::world::{UnsafeWorld, World};

// -----------------------------------------------------------------------------
// Resource

#[cold]
#[inline(never)]
fn uninit_resource(name: DebugName) -> ! {
    panic!("Resource {name} is uninitialzed before system run.")
}

// -----------------------------------------------------------------------------
// Res

unsafe impl<T: Resource + Sync> ReadOnlySystemParam for Res<'_, T> {}

unsafe impl<T: Resource + Sync> SystemParam for Res<'_, T> {
    type State = ResourceId;
    type Item<'world, 'state> = Res<'world, T>;
    const NON_SEND: bool = false;
    const EXCLUSIVE: bool = false;

    fn init_state(world: &mut World) -> Self::State {
        world.register_resource::<T>()
    }

    fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
        table.set_reading_res(*state)
    }

    unsafe fn get_param<'w, 's>(
        world: UnsafeWorld<'w>,
        state: &'s mut Self::State,
        _last_run: Tick,
        _this_run: Tick,
    ) -> Self::Item<'w, 's> {
        unsafe {
            let world = world.read_only();
            if let Some(data) = world.storages.res.get(*state)
                && let Some(ptr) = data.get_data()
            {
                ptr.debug_assert_aligned::<T>();
                Res {
                    value: ptr.as_ref(),
                }
            } else {
                uninit_resource(DebugName::type_name::<T>());
            }
        }
    }
}

// -----------------------------------------------------------------------------
// ResRef

unsafe impl<T: Resource + Sync> ReadOnlySystemParam for ResRef<'_, T> {}

unsafe impl<T: Resource + Sync> SystemParam for ResRef<'_, T> {
    type State = ResourceId;
    type Item<'world, 'state> = ResRef<'world, T>;
    const NON_SEND: bool = false;
    const EXCLUSIVE: bool = false;

    fn init_state(world: &mut World) -> Self::State {
        world.register_resource::<T>()
    }

    fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
        table.set_reading_res(*state)
    }

    unsafe fn get_param<'w, 's>(
        world: UnsafeWorld<'w>,
        state: &'s mut Self::State,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Item<'w, 's> {
        unsafe {
            let world = world.read_only();
            if let Some(data) = world.storages.res.get(*state)
                && let Some(untyped) = data.get_ref(last_run, this_run)
            {
                untyped.into_resource::<T>()
            } else {
                uninit_resource(DebugName::type_name::<T>());
            }
        }
    }
}

// -----------------------------------------------------------------------------
// ResMut

unsafe impl<T: Resource + Send> SystemParam for ResMut<'_, T> {
    type State = ResourceId;
    type Item<'world, 'state> = ResMut<'world, T>;
    const NON_SEND: bool = false;
    const EXCLUSIVE: bool = false;

    fn init_state(world: &mut World) -> Self::State {
        world.register_resource::<T>()
    }

    fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
        table.set_writing_res(*state)
    }

    unsafe fn get_param<'w, 's>(
        world: UnsafeWorld<'w>,
        state: &'s mut Self::State,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Item<'w, 's> {
        unsafe {
            let world = world.data_mut();
            if let Some(data) = world.storages.res.get_mut(*state)
                && let Some(untyped) = data.get_mut(last_run, this_run)
            {
                untyped.into_resource::<T>()
            } else {
                uninit_resource(DebugName::type_name::<T>());
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Option<Res>

unsafe impl<T: Resource + Sync> ReadOnlySystemParam for Option<Res<'_, T>> {}

unsafe impl<T: Resource + Sync> SystemParam for Option<Res<'_, T>> {
    type State = ResourceId;
    type Item<'world, 'state> = Option<Res<'world, T>>;
    const NON_SEND: bool = false;
    const EXCLUSIVE: bool = false;

    fn init_state(world: &mut World) -> Self::State {
        world.register_resource::<T>()
    }

    fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
        table.set_reading_res(*state)
    }

    unsafe fn get_param<'w, 's>(
        world: UnsafeWorld<'w>,
        state: &'s mut Self::State,
        _last_run: Tick,
        _this_run: Tick,
    ) -> Self::Item<'w, 's> {
        unsafe {
            let world = world.read_only();
            let data = world.storages.res.get(*state)?;
            let ptr = data.get_data()?;
            ptr.debug_assert_aligned::<T>();
            Some(Res {
                value: ptr.as_ref(),
            })
        }
    }
}

// -----------------------------------------------------------------------------
// Option<ResRef>

unsafe impl<T: Resource + Sync> ReadOnlySystemParam for Option<ResRef<'_, T>> {}

unsafe impl<T: Resource + Sync> SystemParam for Option<ResRef<'_, T>> {
    type State = ResourceId;
    type Item<'world, 'state> = Option<ResRef<'world, T>>;
    const NON_SEND: bool = false;
    const EXCLUSIVE: bool = false;

    fn init_state(world: &mut World) -> Self::State {
        world.register_resource::<T>()
    }

    fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
        table.set_reading_res(*state)
    }

    unsafe fn get_param<'w, 's>(
        world: UnsafeWorld<'w>,
        state: &'s mut Self::State,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Item<'w, 's> {
        unsafe {
            let world = world.read_only();
            let data = world.storages.res.get(*state)?;
            Some(data.get_ref(last_run, this_run)?.into_resource::<T>())
        }
    }
}

// -----------------------------------------------------------------------------
// Option<ResMut>

unsafe impl<T: Resource + Send> SystemParam for Option<ResMut<'_, T>> {
    type State = ResourceId;
    type Item<'world, 'state> = Option<ResMut<'world, T>>;
    const NON_SEND: bool = false;
    const EXCLUSIVE: bool = false;

    fn init_state(world: &mut World) -> Self::State {
        world.register_resource::<T>()
    }

    fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
        table.set_writing_res(*state)
    }

    unsafe fn get_param<'w, 's>(
        world: UnsafeWorld<'w>,
        state: &'s mut Self::State,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Item<'w, 's> {
        unsafe {
            let world = world.data_mut();
            let data = world.storages.res.get_mut(*state)?;
            Some(data.get_mut(last_run, this_run)?.into_resource::<T>())
        }
    }
}

// -----------------------------------------------------------------------------
// NonSend

unsafe impl<T: Resource> ReadOnlySystemParam for NonSend<'_, T> {}

unsafe impl<T: Resource> SystemParam for NonSend<'_, T> {
    type State = ResourceId;
    type Item<'world, 'state> = NonSend<'world, T>;
    // Because the resource is !Sync, we can only borrow it
    // on the main thread. In other words, this system is !Send.
    const NON_SEND: bool = true;
    const EXCLUSIVE: bool = false;

    fn init_state(world: &mut World) -> Self::State {
        world.register_resource::<T>()
    }

    fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
        table.set_reading_res(*state)
    }

    unsafe fn get_param<'w, 's>(
        world: UnsafeWorld<'w>,
        state: &'s mut Self::State,
        _last_run: Tick,
        _this_run: Tick,
    ) -> Self::Item<'w, 's> {
        unsafe {
            let world = world.read_only();
            if let Some(data) = world.storages.res.get(*state)
                && let Some(ptr) = data.get_data()
            {
                ptr.debug_assert_aligned::<T>();
                NonSend {
                    value: ptr.as_ref(),
                }
            } else {
                uninit_resource(DebugName::type_name::<T>());
            }
        }
    }
}

// -----------------------------------------------------------------------------
// NonSendRef

unsafe impl<T: Resource> ReadOnlySystemParam for NonSendRef<'_, T> {}

unsafe impl<T: Resource> SystemParam for NonSendRef<'_, T> {
    type State = ResourceId;
    type Item<'world, 'state> = NonSendRef<'world, T>;
    // Because the resource is !Sync, we can only borrow it
    // on the main thread. In other words, this system is !Send.
    const NON_SEND: bool = true;
    const EXCLUSIVE: bool = false;

    fn init_state(world: &mut World) -> Self::State {
        world.register_resource::<T>()
        // We do not prepare resource here,
        // thereby delaying memory allocation.
    }

    fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
        table.set_reading_res(*state)
    }

    unsafe fn get_param<'w, 's>(
        world: UnsafeWorld<'w>,
        state: &'s mut Self::State,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Item<'w, 's> {
        unsafe {
            let world = world.read_only();
            if let Some(data) = world.storages.res.get(*state)
                && let Some(ptr) = data.get_ref(last_run, this_run)
            {
                ptr.into_non_send::<T>()
            } else {
                uninit_resource(DebugName::type_name::<T>());
            }
        }
    }
}

// -----------------------------------------------------------------------------
// NonSendMut

unsafe impl<T: Resource> SystemParam for NonSendMut<'_, T> {
    type State = ResourceId;
    type Item<'world, 'state> = NonSendMut<'world, T>;
    // Because the resource is !Sync, we can only borrow it
    // on the main thread. In other words, this system is !Send.
    const NON_SEND: bool = true;
    const EXCLUSIVE: bool = false;

    fn init_state(world: &mut World) -> Self::State {
        world.register_resource::<T>()
    }

    fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
        table.set_writing_res(*state)
    }

    unsafe fn get_param<'w, 's>(
        world: UnsafeWorld<'w>,
        state: &'s mut Self::State,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Item<'w, 's> {
        unsafe {
            let world = world.data_mut();
            if let Some(data) = world.storages.res.get_mut(*state)
                && let Some(ptr) = data.get_mut(last_run, this_run)
            {
                ptr.into_non_send::<T>()
            } else {
                uninit_resource(DebugName::type_name::<T>());
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Option<NonSend>

unsafe impl<T: Resource> ReadOnlySystemParam for Option<NonSend<'_, T>> {}

unsafe impl<T: Resource> SystemParam for Option<NonSend<'_, T>> {
    type State = ResourceId;
    type Item<'world, 'state> = Option<NonSend<'world, T>>;
    // Because the resource is !Sync, we can only borrow it
    // on the main thread. In other words, this system is !Send.
    const NON_SEND: bool = true;
    const EXCLUSIVE: bool = false;

    fn init_state(world: &mut World) -> Self::State {
        world.register_resource::<T>()
    }

    fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
        table.set_reading_res(*state)
    }

    unsafe fn get_param<'w, 's>(
        world: UnsafeWorld<'w>,
        state: &'s mut Self::State,
        _last_run: Tick,
        _this_run: Tick,
    ) -> Self::Item<'w, 's> {
        unsafe {
            let world = world.read_only();
            let data = world.storages.res.get(*state)?;
            let ptr = data.get_data()?;
            ptr.debug_assert_aligned::<T>();
            Some(NonSend {
                value: ptr.as_ref(),
            })
        }
    }
}

// -----------------------------------------------------------------------------
// Option<NonSendRef>

unsafe impl<T: Resource> ReadOnlySystemParam for Option<NonSendRef<'_, T>> {}

unsafe impl<T: Resource> SystemParam for Option<NonSendRef<'_, T>> {
    type State = ResourceId;
    type Item<'world, 'state> = Option<NonSendRef<'world, T>>;
    // Because the resource is !Sync, we can only borrow it
    // on the main thread. In other words, this system is !Send.
    const NON_SEND: bool = true;
    const EXCLUSIVE: bool = false;

    fn init_state(world: &mut World) -> Self::State {
        world.register_resource::<T>()
    }

    fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
        table.set_reading_res(*state)
    }

    unsafe fn get_param<'w, 's>(
        world: UnsafeWorld<'w>,
        state: &'s mut Self::State,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Item<'w, 's> {
        unsafe {
            let world = world.read_only();
            let data = world.storages.res.get(*state)?;
            Some(data.get_ref(last_run, this_run)?.into_non_send::<T>())
        }
    }
}

// -----------------------------------------------------------------------------
// Option<NonSendMut>

unsafe impl<T: Resource> SystemParam for Option<NonSendMut<'_, T>> {
    type State = ResourceId;
    type Item<'world, 'state> = Option<NonSendMut<'world, T>>;
    // Because the resource is !Sync, we can only borrow it
    // on the main thread. In other words, this system is !Send.
    const NON_SEND: bool = true;
    const EXCLUSIVE: bool = false;

    fn init_state(world: &mut World) -> Self::State {
        world.register_resource::<T>()
    }

    fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
        table.set_writing_res(*state)
    }

    unsafe fn get_param<'w, 's>(
        world: UnsafeWorld<'w>,
        state: &'s mut Self::State,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Item<'w, 's> {
        unsafe {
            let world = world.data_mut();
            let data = world.storages.res.get_mut(*state)?;
            Some(data.get_mut(last_run, this_run)?.into_non_send::<T>())
        }
    }
}

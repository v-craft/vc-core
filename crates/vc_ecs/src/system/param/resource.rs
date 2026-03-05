use super::{SystemParam, ReadOnlySystemParam};
use crate::borrow::{NonSync, NonSyncMut, NonSyncRef};
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

    unsafe fn init_state(world: &mut World) -> Self::State {
        world.register_resource::<T>()
    }

    unsafe fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
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

    unsafe fn init_state(world: &mut World) -> Self::State {
        world.register_resource::<T>()
    }

    unsafe fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
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
                untyped.into_res::<T>()
            } else {
                uninit_resource(DebugName::type_name::<T>());
            }
        }
    }
}

// -----------------------------------------------------------------------------
// ResMut

unsafe impl<T: Resource + Sync> SystemParam for ResMut<'_, T> {
    type State = ResourceId;
    type Item<'world, 'state> = ResMut<'world, T>;
    const NON_SEND: bool = false;
    const EXCLUSIVE: bool = false;

    unsafe fn init_state(world: &mut World) -> Self::State {
        world.register_resource::<T>()
    }

    unsafe fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
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
                untyped.into_res::<T>()
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

    unsafe fn init_state(world: &mut World) -> Self::State {
        world.register_resource::<T>()
    }

    unsafe fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
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

    unsafe fn init_state(world: &mut World) -> Self::State {
        world.register_resource::<T>()
    }

    unsafe fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
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
            Some(data.get_ref(last_run, this_run)?.into_res::<T>())
        }
    }
}

// -----------------------------------------------------------------------------
// Option<ResMut>

unsafe impl<T: Resource + Sync> SystemParam for Option<ResMut<'_, T>> {
    type State = ResourceId;
    type Item<'world, 'state> = Option<ResMut<'world, T>>;
    const NON_SEND: bool = false;
    const EXCLUSIVE: bool = false;

    unsafe fn init_state(world: &mut World) -> Self::State {
        world.register_resource::<T>()
    }

    unsafe fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
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
            Some(data.get_mut(last_run, this_run)?.into_res::<T>())
        }
    }
}

// -----------------------------------------------------------------------------
// NonSync

unsafe impl<T: Resource> ReadOnlySystemParam for NonSync<'_, T> {}

unsafe impl<T: Resource> SystemParam for NonSync<'_, T> {
    type State = ResourceId;
    type Item<'world, 'state> = NonSync<'world, T>;
    // Because the resource is !Sync, we can only borrow it
    // on the main thread. In other words, this system is !Send. 
    const NON_SEND: bool = true;
    const EXCLUSIVE: bool = false;

    unsafe fn init_state(world: &mut World) -> Self::State {
        world.register_resource::<T>()
    }

    unsafe fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
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
                NonSync {
                    value: ptr.as_ref(),
                }
            } else {
                uninit_resource(DebugName::type_name::<T>());
            }
        }
    }
}

// -----------------------------------------------------------------------------
// NonSyncRef

unsafe impl<T: Resource> ReadOnlySystemParam for NonSyncRef<'_, T> {}

unsafe impl<T: Resource> SystemParam for NonSyncRef<'_, T> {
    type State = ResourceId;
    type Item<'world, 'state> = NonSyncRef<'world, T>;
    // Because the resource is !Sync, we can only borrow it
    // on the main thread. In other words, this system is !Send. 
    const NON_SEND: bool = true;
    const EXCLUSIVE: bool = false;

    unsafe fn init_state(world: &mut World) -> Self::State {
        world.register_resource::<T>()
        // We do not prepare resource here,
        // thereby delaying memory allocation.
    }

    unsafe fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
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
                ptr.into_non_sync::<T>()
            } else {
                uninit_resource(DebugName::type_name::<T>());
            }
        }
    }
}

// -----------------------------------------------------------------------------
// NonSyncMut

unsafe impl<T: Resource> SystemParam for NonSyncMut<'_, T> {
    type State = ResourceId;
    type Item<'world, 'state> = NonSyncMut<'world, T>;
    // Because the resource is !Sync, we can only borrow it
    // on the main thread. In other words, this system is !Send. 
    const NON_SEND: bool = true;
    const EXCLUSIVE: bool = false;

    unsafe fn init_state(world: &mut World) -> Self::State {
        world.register_resource::<T>()
    }

    unsafe fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
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
                ptr.into_non_sync::<T>()
            } else {
                uninit_resource(DebugName::type_name::<T>());
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Option<NonSync>

unsafe impl<T: Resource> ReadOnlySystemParam for Option<NonSync<'_, T>> {}

unsafe impl<T: Resource> SystemParam for Option<NonSync<'_, T>> {
    type State = ResourceId;
    type Item<'world, 'state> = Option<NonSync<'world, T>>;
    // Because the resource is !Sync, we can only borrow it
    // on the main thread. In other words, this system is !Send. 
    const NON_SEND: bool = true;
    const EXCLUSIVE: bool = false;

    unsafe fn init_state(world: &mut World) -> Self::State {
        world.register_resource::<T>()
    }

    unsafe fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
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
            Some(NonSync {
                value: ptr.as_ref(),
            })
        }
    }
}

// -----------------------------------------------------------------------------
// Option<NonSyncRef>

unsafe impl<T: Resource> ReadOnlySystemParam for Option<NonSyncRef<'_, T>> {}

unsafe impl<T: Resource> SystemParam for Option<NonSyncRef<'_, T>> {
    type State = ResourceId;
    type Item<'world, 'state> = Option<NonSyncRef<'world, T>>;
    // Because the resource is !Sync, we can only borrow it
    // on the main thread. In other words, this system is !Send. 
    const NON_SEND: bool = true;
    const EXCLUSIVE: bool = false;

    unsafe fn init_state(world: &mut World) -> Self::State {
        world.register_resource::<T>()
    }

    unsafe fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
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
            Some(data.get_ref(last_run, this_run)?.into_non_sync::<T>())
        }
    }
}

// -----------------------------------------------------------------------------
// Option<NonSyncMut>

unsafe impl<T: Resource> SystemParam for Option<NonSyncMut<'_, T>> {
    type State = ResourceId;
    type Item<'world, 'state> = Option<NonSyncMut<'world, T>>;
    // Because the resource is !Sync, we can only borrow it
    // on the main thread. In other words, this system is !Send. 
    const NON_SEND: bool = true;
    const EXCLUSIVE: bool = false;

    unsafe fn init_state(world: &mut World) -> Self::State {
        world.register_resource::<T>()
    }

    unsafe fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
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
            Some(data.get_mut(last_run, this_run)?.into_non_sync::<T>())
        }
    }
}

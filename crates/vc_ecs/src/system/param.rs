use core::marker::PhantomData;

use crate::borrow::{NonSend, NonSendMut, Res, ResMut};
use crate::component::{ComponentId, NonSendResource, Resource};
use crate::tick::Tick;
use crate::utils::DebugName;
use crate::world::{AccessTable, UnsafeWorld, World, WorldId, WorldMode};

// -----------------------------------------------------------------------------
// SystemParam

pub unsafe trait SystemParam: Sized {
    type State: Send + Sync + 'static;
    type Item<'world, 'state>: SystemParam<State = Self::State>;
    const MODE: WorldMode;
    const NON_SEND: bool;

    fn init_state(world: &mut World) -> Self::State;

    unsafe fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool;

    unsafe fn fetch<'w, 's>(
        world: UnsafeWorld<'w>,
        state: &'s mut Self::State,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Item<'w, 's>;
}

// -----------------------------------------------------------------------------
// Implementation

// ---------------------------------------------------------
// World

unsafe impl SystemParam for &World {
    type State = ();
    type Item<'world, 'state> = &'world World;
    const MODE: WorldMode = WorldMode::Read;
    const NON_SEND: bool = false;

    fn init_state(_world: &mut World) -> Self::State {}

    unsafe fn mark_access(table: &mut AccessTable, _state: &Self::State) -> bool {
        if table.all_readable() {
            unsafe {
                table.set_read_all();
            }
            true
        } else {
            false
        }
    }

    unsafe fn fetch<'w, 's>(
        world: UnsafeWorld<'w>,
        _state: &'s mut Self::State,
        _last_run: Tick,
        _this_run: Tick,
    ) -> Self::Item<'w, 's> {
        unsafe { world.read() }
    }
}

unsafe impl SystemParam for &mut World {
    type State = ();
    type Item<'world, 'state> = &'world mut World;
    const MODE: WorldMode = WorldMode::FullMut;
    const NON_SEND: bool = false;

    fn init_state(_world: &mut World) -> Self::State {}

    unsafe fn mark_access(table: &mut AccessTable, _state: &Self::State) -> bool {
        if table.full_mutable() {
            unsafe {
                table.set_full_mut();
            }
            true
        } else {
            false
        }
    }

    unsafe fn fetch<'w, 's>(
        world: UnsafeWorld<'w>,
        _state: &'s mut Self::State,
        _last_run: Tick,
        _this_run: Tick,
    ) -> Self::Item<'w, 's> {
        unsafe { world.full_mut() }
    }
}

// ---------------------------------------------------------
// Resource

#[cold]
#[inline(never)]
fn invalid_resource<T>(world_id: WorldId) -> ! {
    panic!(
        "Resource<{}> in world({}) is uninitialized.",
        DebugName::type_name::<T>(),
        world_id,
    );
}

unsafe impl<T: Resource> SystemParam for Res<'_, T> {
    type State = ComponentId;
    type Item<'world, 'state> = Res<'world, T>;
    const MODE: WorldMode = WorldMode::Read;
    const NON_SEND: bool = false;

    fn init_state(world: &mut World) -> Self::State {
        world.register_resouce::<T>()
    }

    unsafe fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
        if table.is_readable(*state) {
            unsafe {
                table.set_reading(*state);
            }
            true
        } else {
            false
        }
    }

    unsafe fn fetch<'w, 's>(
        world: UnsafeWorld<'w>,
        state: &'s mut Self::State,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Item<'w, 's> {
        unsafe {
            let world = world.read();
            let data = world.storages.resources.get(*state);
            if !data.is_valid() {
                invalid_resource::<T>(world.id);
            }
            data.get_ref(last_run, this_run).into_res::<T>()
        }
    }
}

unsafe impl<T: Resource> SystemParam for ResMut<'_, T> {
    type State = ComponentId;
    type Item<'world, 'state> = ResMut<'world, T>;
    const MODE: WorldMode = WorldMode::DataMut;
    const NON_SEND: bool = false;

    fn init_state(world: &mut World) -> Self::State {
        world.register_resouce::<T>()
    }

    unsafe fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
        if table.is_writable(*state) {
            unsafe {
                table.set_writing(*state);
            }
            true
        } else {
            false
        }
    }

    unsafe fn fetch<'w, 's>(
        world: UnsafeWorld<'w>,
        state: &'s mut Self::State,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Item<'w, 's> {
        unsafe {
            let world = world.data_mut();
            let data = world.storages.resources.get_mut(*state);
            if !data.is_valid() {
                invalid_resource::<T>(world.id);
            }
            data.get_mut(last_run, this_run).into_res::<T>()
        }
    }
}

unsafe impl<T: Resource> SystemParam for Option<Res<'_, T>> {
    type State = ComponentId;
    type Item<'world, 'state> = Option<Res<'world, T>>;
    const MODE: WorldMode = WorldMode::Read;
    const NON_SEND: bool = false;

    fn init_state(world: &mut World) -> Self::State {
        world.register_resouce::<T>()
    }

    unsafe fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
        if table.is_readable(*state) {
            unsafe {
                table.set_reading(*state);
            }
            true
        } else {
            false
        }
    }

    unsafe fn fetch<'w, 's>(
        world: UnsafeWorld<'w>,
        state: &'s mut Self::State,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Item<'w, 's> {
        unsafe {
            let world = world.read();
            let data = world.storages.resources.get(*state);
            if data.is_valid() {
                Some(data.get_ref(last_run, this_run).into_res::<T>())
            } else {
                None
            }
        }
    }
}

unsafe impl<T: Resource> SystemParam for Option<ResMut<'_, T>> {
    type State = ComponentId;
    type Item<'world, 'state> = Option<ResMut<'world, T>>;
    const MODE: WorldMode = WorldMode::DataMut;
    const NON_SEND: bool = false;

    fn init_state(world: &mut World) -> Self::State {
        world.register_resouce::<T>()
    }

    unsafe fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
        if table.is_writable(*state) {
            unsafe {
                table.set_writing(*state);
            }
            true
        } else {
            false
        }
    }

    unsafe fn fetch<'w, 's>(
        world: UnsafeWorld<'w>,
        state: &'s mut Self::State,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Item<'w, 's> {
        unsafe {
            let world = world.data_mut();
            let data = world.storages.resources.get_mut(*state);
            if data.is_valid() {
                Some(data.get_mut(last_run, this_run).into_res::<T>())
            } else {
                None
            }
        }
    }
}

// ---------------------------------------------------------
// NonSendResource

#[cold]
#[inline(never)]
fn invalid_non_send<T>(world_id: WorldId) -> ! {
    panic!(
        "NonSendResource<{}> in world({}) is uninitialized.",
        DebugName::type_name::<T>(),
        world_id,
    );
}

unsafe impl<T: NonSendResource> SystemParam for NonSend<'_, T> {
    type State = ComponentId;
    type Item<'world, 'state> = NonSend<'world, T>;
    const MODE: WorldMode = WorldMode::Read;
    const NON_SEND: bool = true;

    fn init_state(world: &mut World) -> Self::State {
        world.register_non_send::<T>()
    }

    unsafe fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
        if table.is_readable(*state) {
            unsafe {
                table.set_reading(*state);
            }
            true
        } else {
            false
        }
    }

    unsafe fn fetch<'w, 's>(
        world: UnsafeWorld<'w>,
        state: &'s mut Self::State,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Item<'w, 's> {
        unsafe {
            let world = world.read();
            let data = world.storages.non_sends.get(*state);
            if !data.is_valid() {
                invalid_non_send::<T>(world.id);
            }
            data.get_ref(last_run, this_run).into_non_send::<T>()
        }
    }
}

unsafe impl<T: NonSendResource> SystemParam for NonSendMut<'_, T> {
    type State = ComponentId;
    type Item<'world, 'state> = NonSendMut<'world, T>;
    const MODE: WorldMode = WorldMode::DataMut;
    const NON_SEND: bool = true;

    fn init_state(world: &mut World) -> Self::State {
        world.register_non_send::<T>()
    }

    unsafe fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
        if table.is_writable(*state) {
            unsafe {
                table.set_writing(*state);
            }
            true
        } else {
            false
        }
    }

    unsafe fn fetch<'w, 's>(
        world: UnsafeWorld<'w>,
        state: &'s mut Self::State,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Item<'w, 's> {
        unsafe {
            let world = world.data_mut();
            let data = world.storages.non_sends.get_mut(*state);
            if !data.is_valid() {
                invalid_non_send::<T>(world.id);
            }
            data.get_mut(last_run, this_run).into_non_send::<T>()
        }
    }
}

unsafe impl<T: NonSendResource> SystemParam for Option<NonSend<'_, T>> {
    type State = ComponentId;
    type Item<'world, 'state> = Option<NonSend<'world, T>>;
    const MODE: WorldMode = WorldMode::Read;
    const NON_SEND: bool = false;

    fn init_state(world: &mut World) -> Self::State {
        world.register_non_send::<T>()
    }

    unsafe fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
        if table.is_readable(*state) {
            unsafe {
                table.set_reading(*state);
            }
            true
        } else {
            false
        }
    }

    unsafe fn fetch<'w, 's>(
        world: UnsafeWorld<'w>,
        state: &'s mut Self::State,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Item<'w, 's> {
        unsafe {
            let world = world.read();
            let data = world.storages.non_sends.get(*state);
            if data.is_valid() {
                Some(data.get_ref(last_run, this_run).into_non_send::<T>())
            } else {
                None
            }
        }
    }
}

unsafe impl<T: NonSendResource> SystemParam for Option<NonSendMut<'_, T>> {
    type State = ComponentId;
    type Item<'world, 'state> = Option<NonSendMut<'world, T>>;
    const MODE: WorldMode = WorldMode::DataMut;
    const NON_SEND: bool = false;

    fn init_state(world: &mut World) -> Self::State {
        world.register_non_send::<T>()
    }

    unsafe fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
        if table.is_writable(*state) {
            unsafe {
                table.set_writing(*state);
            }
            true
        } else {
            false
        }
    }

    unsafe fn fetch<'w, 's>(
        world: UnsafeWorld<'w>,
        state: &'s mut Self::State,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Item<'w, 's> {
        unsafe {
            let world = world.data_mut();
            let data = world.storages.non_sends.get_mut(*state);
            if data.is_valid() {
                Some(data.get_mut(last_run, this_run).into_non_send::<T>())
            } else {
                None
            }
        }
    }
}

// ---------------------------------------------------------
// PhantomData

unsafe impl<T: SystemParam> SystemParam for PhantomData<T> {
    type State = ();
    type Item<'world, 'state> = PhantomData<T>;
    const MODE: WorldMode = T::MODE;
    const NON_SEND: bool = T::NON_SEND;

    fn init_state(_world: &mut World) -> Self::State {}

    unsafe fn mark_access(_table: &mut AccessTable, _state: &Self::State) -> bool {
        true
    }

    unsafe fn fetch<'w, 's>(
        _world: UnsafeWorld<'w>,
        _state: &'s mut Self::State,
        _last_run: Tick,
        _this_run: Tick,
    ) -> Self::Item<'w, 's> {
        PhantomData
    }
}

#![allow(clippy::missing_safety_doc, reason = "todo")]

use core::marker::PhantomData;

use super::AccessTable;
use crate::borrow::{Res, ResMut};
use crate::resource::{Resource, ResourceId};
use crate::tick::Tick;
use crate::utils::DebugName;
use crate::world::{UnsafeWorld, World, WorldMode};

// -----------------------------------------------------------------------------
// SystemParam

pub unsafe trait SystemParam: Sized {
    type State: Send + Sync + 'static;
    type Item<'world, 'state>: SystemParam<State = Self::State>;
    const MODE: WorldMode;
    const IS_SEND: bool;

    unsafe fn init_state(world: &mut World) -> Self::State;

    unsafe fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool;

    unsafe fn get_param<'w, 's>(
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
    const MODE: WorldMode = WorldMode::ReadOnly;
    const IS_SEND: bool = true;

    unsafe fn init_state(_world: &mut World) -> Self::State {}

    unsafe fn mark_access(table: &mut AccessTable, _state: &Self::State) -> bool {
        if table.can_world_ref() {
            table.set_world_ref();
            true
        } else {
            false
        }
    }

    unsafe fn get_param<'w, 's>(
        world: UnsafeWorld<'w>,
        _state: &'s mut Self::State,
        _last_run: Tick,
        _this_run: Tick,
    ) -> Self::Item<'w, 's> {
        unsafe { world.read_only() }
    }
}

unsafe impl SystemParam for &mut World {
    type State = ();
    type Item<'world, 'state> = &'world mut World;
    const MODE: WorldMode = WorldMode::FullMut;
    const IS_SEND: bool = true;

    unsafe fn init_state(_world: &mut World) -> Self::State {}

    unsafe fn mark_access(table: &mut AccessTable, _state: &Self::State) -> bool {
        if table.can_world_mut() {
            table.set_world_mut();
            true
        } else {
            false
        }
    }

    unsafe fn get_param<'w, 's>(
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
fn uninit_resource(name: DebugName) -> ! {
    panic!("Resource {name} is uninitialzed before system run.")
}

unsafe impl<T: Resource> SystemParam for Res<'_, T> {
    type State = ResourceId;
    type Item<'world, 'state> = Res<'world, T>;
    const MODE: WorldMode = WorldMode::ReadOnly;
    const IS_SEND: bool = T::IS_SEND;

    unsafe fn init_state(world: &mut World) -> Self::State {
        world.register_resource::<T>()
        // We do not prepare resource here,
        // thereby delaying memory allocation.
    }

    unsafe fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
        if table.can_reading_res(*state) {
            table.set_reading_res(*state);
            true
        } else {
            false
        }
    }

    unsafe fn get_param<'w, 's>(
        world: UnsafeWorld<'w>,
        state: &'s mut Self::State,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Item<'w, 's> {
        unsafe {
            let world = world.read_only();
            let Some(data) = world.storages.res_set.get(*state) else {
                uninit_resource(DebugName::type_name::<T>());
            };
            data.assert_get_ref(last_run, this_run).into_res::<T>()
        }
    }
}

unsafe impl<T: Resource> SystemParam for ResMut<'_, T> {
    type State = ResourceId;
    type Item<'world, 'state> = ResMut<'world, T>;
    const MODE: WorldMode = WorldMode::DataMut;
    const IS_SEND: bool = T::IS_SEND;

    unsafe fn init_state(world: &mut World) -> Self::State {
        world.register_resource::<T>()
        // We do not prepare resource here,
        // thereby delaying memory allocation.
    }

    unsafe fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
        if table.can_writing_res(*state) {
            table.set_writing_res(*state);
            true
        } else {
            false
        }
    }

    unsafe fn get_param<'w, 's>(
        world: UnsafeWorld<'w>,
        state: &'s mut Self::State,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Item<'w, 's> {
        unsafe {
            let world = world.data_mut();
            let Some(data) = world.storages.res_set.get_mut(*state) else {
                uninit_resource(DebugName::type_name::<T>());
            };
            data.assert_get_mut(last_run, this_run).into_res::<T>()
        }
    }
}

unsafe impl<T: Resource> SystemParam for Option<Res<'_, T>> {
    type State = ResourceId;
    type Item<'world, 'state> = Option<Res<'world, T>>;
    const MODE: WorldMode = WorldMode::ReadOnly;
    const IS_SEND: bool = T::IS_SEND;

    unsafe fn init_state(world: &mut World) -> Self::State {
        world.register_resource::<T>()
        // We do not prepare resource here,
        // thereby delaying memory allocation.
    }

    unsafe fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
        if table.can_reading_res(*state) {
            table.set_reading_res(*state);
            true
        } else {
            false
        }
    }

    unsafe fn get_param<'w, 's>(
        world: UnsafeWorld<'w>,
        state: &'s mut Self::State,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Item<'w, 's> {
        unsafe {
            let world = world.read_only();
            let data = world.storages.res_set.get(*state)?;
            Some(data.get_ref(last_run, this_run)?.into_res::<T>())
        }
    }
}

unsafe impl<T: Resource> SystemParam for Option<ResMut<'_, T>> {
    type State = ResourceId;
    type Item<'world, 'state> = Option<ResMut<'world, T>>;
    const MODE: WorldMode = WorldMode::DataMut;
    const IS_SEND: bool = T::IS_SEND;

    unsafe fn init_state(world: &mut World) -> Self::State {
        world.register_resource::<T>()
    }

    unsafe fn mark_access(table: &mut AccessTable, state: &Self::State) -> bool {
        if table.can_writing_res(*state) {
            table.set_writing_res(*state);
            true
        } else {
            false
        }
    }

    unsafe fn get_param<'w, 's>(
        world: UnsafeWorld<'w>,
        state: &'s mut Self::State,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Item<'w, 's> {
        unsafe {
            let world = world.data_mut();
            let data = world.storages.res_set.get_mut(*state)?;
            Some(data.get_mut(last_run, this_run)?.into_res::<T>())
        }
    }
}

// ---------------------------------------------------------
// PhantomData

unsafe impl<T: SystemParam> SystemParam for PhantomData<T> {
    type State = ();
    type Item<'world, 'state> = PhantomData<T>;
    const MODE: WorldMode = T::MODE;
    const IS_SEND: bool = T::IS_SEND;

    unsafe fn init_state(_world: &mut World) -> Self::State {}

    unsafe fn mark_access(_table: &mut AccessTable, _state: &Self::State) -> bool {
        true
    }

    unsafe fn get_param<'w, 's>(
        _world: UnsafeWorld<'w>,
        _state: &'s mut Self::State,
        _last_run: Tick,
        _this_run: Tick,
    ) -> Self::Item<'w, 's> {
        PhantomData
    }
}

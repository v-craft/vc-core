use core::ops::{Deref, DerefMut};

use super::{ReadOnlySystemParam, SystemParam};
use crate::system::AccessTable;
use crate::tick::Tick;
use crate::world::{UnsafeWorld, World};

#[derive(Debug)]
pub struct Local<'s, T: Default + Send + Sync + 'static>(&'s mut T);

impl<'s, T: Default + Send + Sync + 'static> Deref for Local<'s, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'s, T: Default + Send + Sync + 'static> DerefMut for Local<'s, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

unsafe impl<T: Default + Send + Sync + 'static> ReadOnlySystemParam for Local<'_, T> {}

unsafe impl<T: Default + Send + Sync + 'static> SystemParam for Local<'_, T> {
    type State = T;
    type Item<'world, 'state> = Local<'state, T>;

    const NON_SEND: bool = false;
    const EXCLUSIVE: bool = false;

    unsafe fn init_state(_world: &mut World) -> Self::State {
        T::default()
    }

    unsafe fn mark_access(_table: &mut AccessTable, _state: &Self::State) -> bool {
        true
    }

    unsafe fn get_param<'w, 's>(
        _world: UnsafeWorld<'w>,
        state: &'s mut Self::State,
        _last_run: Tick,
        _this_run: Tick,
    ) -> Self::Item<'w, 's> {
        Local(state)
    }
}


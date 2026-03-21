use alloc::boxed::Box;
use core::panic::Location;

use crate::error::EcsError;
use crate::world::World;

pub struct CommandObject {
    location: &'static Location<'static>,
    function: Box<dyn FnOnce(&mut World) -> Result<(), EcsError> + Send + 'static>,
}

impl CommandObject {
    #[track_caller]
    #[inline(always)]
    pub fn new<F>(func: F) -> Self
    where
        F: Send + 'static,
        F: FnOnce(&mut World) -> Result<(), EcsError>,
    {
        Self {
            location: Location::caller(),
            function: Box::new(func),
        }
    }

    pub fn location(&self) -> Location<'static> {
        *self.location
    }

    pub fn run(self, world: &mut World) -> Result<(), EcsError> {
        (self.function)(world)
    }
}

const _STATIC_ASSERT_: () = const {
    const fn is_send<T: Send>() {}
    is_send::<CommandObject>();
};

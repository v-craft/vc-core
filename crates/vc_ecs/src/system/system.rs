#![expect(clippy::module_inception, reason = "For better structure.")]
#![expect(clippy::missing_safety_doc, reason = "TODO")]

use alloc::boxed::Box;
use core::fmt::Debug;

use crate::error::EcsError;
use crate::system::{AccessTable, FunctionSystem, SystemFlags, SystemFunction};
use crate::tick::Tick;
use crate::utils::DebugName;
use crate::world::{UnsafeWorld, World};

use super::SystemInput;

// -----------------------------------------------------------------------------
// System

#[diagnostic::on_unimplemented(message = "`{Self}` is not a system", label = "invalid system")]
pub trait System: Send + Sync + 'static {
    /// The system's input.
    type Input: SystemInput;
    /// The system's output.
    type Output;

    fn name(&self) -> DebugName;

    fn flags(&self) -> SystemFlags;

    fn get_last_run(&self) -> Tick;
    fn set_last_run(&mut self, last_run: Tick);

    fn initialize(&mut self, world: &mut World) -> AccessTable;

    unsafe fn run(
        &mut self,
        input: <Self::Input as SystemInput>::Data<'_>,
        world: UnsafeWorld<'_>,
    ) -> Result<Self::Output, EcsError>;

    #[inline]
    fn is_non_send(&self) -> bool {
        self.flags().intersects(SystemFlags::NON_SEND)
    }

    #[inline]
    fn is_exclusive(&self) -> bool {
        self.flags().intersects(SystemFlags::EXCLUSIVE)
    }
}

pub unsafe trait ReadOnlySystem: System {}

impl<I, O> Debug for dyn System<Input = I, Output = O>
where
    I: SystemInput + 'static,
    O: 'static,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("System")
            .field("name", &self.name())
            .field("non_send", &self.is_non_send())
            .field("exclusive", &self.is_exclusive())
            .finish_non_exhaustive()
    }
}

impl<I, O> Debug for dyn ReadOnlySystem<Input = I, Output = O>
where
    I: SystemInput + 'static,
    O: 'static,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ReadOnlySystem")
            .field("name", &self.name())
            .field("non_send", &self.is_non_send())
            .field("exclusive", &self.is_exclusive())
            .finish_non_exhaustive()
    }
}

// -----------------------------------------------------------------------------
// Alias

pub type BoxedSystem<I, O> = Box<dyn System<Input = I, Output = O>>;
pub type BoxedReadOnlySystem<I, O> = Box<dyn ReadOnlySystem<Input = I, Output = O>>;

// -----------------------------------------------------------------------------
// IntoSystem

pub trait IntoSystem<I: SystemInput, O, Marker>: Sized {
    type System: System<Input = I, Output = O>;

    fn into_system(this: Self) -> Self::System;
}

impl<T: System> IntoSystem<T::Input, T::Output, ()> for T {
    type System = T;
    fn into_system(this: Self) -> Self::System {
        this
    }
}

pub struct FunctionMarker;

impl<T: SystemFunction> IntoSystem<T::Input, T::Output, FunctionMarker> for T {
    type System = FunctionSystem<T::Input, T::Output, T>;
    fn into_system(this: Self) -> Self::System {
        FunctionSystem::new(this)
    }
}

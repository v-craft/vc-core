#![expect(clippy::module_inception, reason = "For better structure.")]

use core::fmt::Debug;

use crate::error::ECSError;
use crate::system::{AccessTable, SystemFlags};
use crate::tick::Tick;
use crate::utils::DebugName;
use crate::world::{UnsafeWorld, World};

use super::SystemInput;

// -----------------------------------------------------------------------------
// System

#[diagnostic::on_unimplemented(message = "`{Self}` is not a system", label = "invalid system")]
pub unsafe trait System: Send + Sync + 'static {
    /// The system's input.
    type In: SystemInput;
    /// The system's output.
    type Out;

    fn name(&self) -> DebugName;

    fn flags(&self) -> SystemFlags;

    fn get_last_run(&self) -> Tick;
    fn set_last_run(&mut self, last_run: Tick);

    fn initialize(&mut self, world: &mut World) -> AccessTable;

    unsafe fn run(
        &mut self,
        input: <Self::In as SystemInput>::Inner<'_>,
        world: UnsafeWorld<'_>,
    ) -> Result<Self::Out, ECSError>;

    #[inline]
    fn is_non_send(&self) -> bool {
        self.flags().intersects(SystemFlags::NON_SEND)
    }

    #[inline]
    fn is_exclusive(&self) -> bool {
        self.flags().intersects(SystemFlags::EXCLUSIVE)
    }
}

impl<In, Out> Debug for dyn System<In = In, Out = Out>
where
    In: SystemInput + 'static,
    Out: 'static,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("System")
            .field("name", &self.name())
            .field("non_send", &self.is_non_send())
            .field("exclusive", &self.is_exclusive())
            .finish_non_exhaustive()
    }
}

// -----------------------------------------------------------------------------
// IntoSystem

pub trait IntoSystem<In: SystemInput, Out, Marker>: Sized {
    type System: System<In = In, Out = Out>;

    fn into_system(this: Self) -> Self::System;
}


impl<T: System> IntoSystem<T::In, T::Out, ()> for T {
    type System = T;
    fn into_system(this: Self) -> Self {
        this
    }
}


#![expect(clippy::module_inception, reason = "For better structure.")]

use crate::error::ECSError;
use crate::system::flag::SystemFlags;
use crate::tick::Tick;
use crate::utils::DebugName;
use crate::world::UnsafeWorld;

use super::SystemInput;

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

    unsafe fn run(
        &mut self,
        input: <Self::In as SystemInput>::Inner<'_>,
        world: UnsafeWorld<'_>,
    ) -> Result<Self::Out, ECSError>;
}

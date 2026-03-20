mod multi;
mod single;

pub use multi::MultiThreadedExecutor;
pub use single::SingleThreadedExecutor;

// -----------------------------------------------------------------------------
// Exports

use super::SystemSchedule;
use crate::error::ErrorHandler;
use crate::world::World;

pub trait SystemExecutor {
    fn kind(&self) -> ExecutorKind;

    fn init(&mut self, schedule: &SystemSchedule);

    fn run(&mut self, schedule: &mut SystemSchedule, world: &mut World, handler: ErrorHandler);
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub enum ExecutorKind {
    #[cfg_attr(any(target_arch = "wasm32", not(feature = "std")), default)]
    SingleThreaded,
    #[cfg_attr(all(not(target_arch = "wasm32"), feature = "std"), default)]
    MultiThreaded,
}

// -----------------------------------------------------------------------------
// MultiThreadExecutor

use crate::resource::Resource;
use crate::utils::Cloner;
use vc_os::sync::Arc;
use vc_task::ScopeExecutor;

#[derive(Clone)]
pub struct MainThreadExecutor(pub Arc<ScopeExecutor<'static>>);

impl Resource for MainThreadExecutor {
    const MUTABLE: bool = false;
    const CLONER: Option<Cloner> = Some(Cloner::clonable::<Self>());
}

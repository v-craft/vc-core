use core::marker::PhantomData;
use core::panic::AssertUnwindSafe;

use crate::cfg;
use crate::error::{EcsError, ErrorContext};
use crate::schedule::{ExecutorKind, SystemExecutor, SystemSchedule};
use crate::world::World;

/// Runs the schedule using a single thread.
///
/// Useful if you're dealing with a single-threaded environment,
/// saving your threads for other things, or just trying minimize overhead.
pub struct SingleThreadedExecutor {
    // Sealed, direct creation is prohibited
    _marker: PhantomData<()>,
}

impl SingleThreadedExecutor {
    pub const fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl Default for SingleThreadedExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemExecutor for SingleThreadedExecutor {
    fn kind(&self) -> ExecutorKind {
        ExecutorKind::SingleThreaded
    }

    fn init(&mut self, schedule: &SystemSchedule) {
        assert_eq!(schedule.keys.len(), schedule.systems.len());
    }

    fn run(
        &mut self,
        schedule: &mut SystemSchedule,
        world: &mut World,
        handler: fn(EcsError, ErrorContext),
    ) {
        let system_count = schedule.systems.len();
        assert_eq!(system_count, schedule.keys.len());

        schedule.systems.iter_mut().for_each(|obj| {
            let system = &mut obj.system;
            let name = system.name();
            let func = AssertUnwindSafe(|| unsafe {
                if let Err(e) = system.run((), world.unsafe_world()) {
                    let ctx = ErrorContext::System {
                        name: name.as_str(),
                        last_run: system.get_last_run(),
                    };
                    handler(e, ctx);
                }
            });

            cfg::std! {
                if {
                    if let Err(payload) = ::std::panic::catch_unwind(func) {
                        ::std::eprintln!("Encountered a panic in system `{}`!", system.name());
                        ::std::panic::resume_unwind(payload);
                    }
                } else {
                    (func)();
                }
            }
        });
    }
}

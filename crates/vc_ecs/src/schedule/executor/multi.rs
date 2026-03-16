use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::any::Any;
use core::panic::AssertUnwindSafe;
use vc_os::sync::{Mutex, PoisonError, SyncUnsafeCell};
use vc_os::utils::ListQueue;
use vc_task::{ComputeTaskPool, Scope, TaskPool};

use super::{MainThreadExecutor, SystemExecutor};

use crate::cfg;
use crate::error::{EcsError, ErrorContext};
use crate::schedule::{ExecutorKind, SystemObject, SystemSchedule};
use crate::system::System;
use crate::world::{UnsafeWorld, World};

// -----------------------------------------------------------------------------
// State

struct ExecutorState {
    incoming: Vec<u32>,
    ready_systems: VecDeque<u32>,
}

/// Runs the schedule using multi-threads.
pub struct MultiThreadedExecutor {
    state: Mutex<ExecutorState>,
    completed: ListQueue<u32>,
    panic_payload: Mutex<Option<Box<dyn Any + Send>>>,
}

#[derive(Copy, Clone)]
struct Context<'scope, 'env, 'sys> {
    world: UnsafeWorld<'env>,
    executor: &'env MultiThreadedExecutor,
    systems: &'sys [SyncUnsafeCell<SystemObject>],
    outgoing: &'sys [Vec<u32>],
    scope: &'scope Scope<'scope, 'env, ()>,
    error_handler: fn(EcsError, ErrorContext),
}

type UnitSystem = Box<dyn System<Input = (), Output = ()>>;

// -----------------------------------------------------------------------------
// Implementation

impl ExecutorState {
    const fn new() -> Self {
        Self {
            incoming: Vec::new(),
            ready_systems: VecDeque::new(),
        }
    }

    fn init(&mut self, schedule: &SystemSchedule) {
        let systen_count = schedule.systems.len();
        self.ready_systems = VecDeque::with_capacity(systen_count >> 2);
        self.incoming = Vec::with_capacity(systen_count + (systen_count >> 3));
    }

    fn reset(&mut self, schedule: &SystemSchedule) {
        // Use `clone_from` to avoid memory reallocation.
        self.incoming.clone_from(&schedule.incoming);
        self.ready_systems.clear();
        self.incoming.iter().enumerate().for_each(|(idx, &num)| {
            if num == 0 {
                self.ready_systems.push_back(idx as u32);
            }
        });
    }
}

impl MultiThreadedExecutor {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(ExecutorState::new()),
            completed: ListQueue::default(),
            panic_payload: Mutex::new(None),
        }
    }
}

impl<'scope, 'env: 'scope, 'sys: 'scope> Context<'scope, 'env, 'sys> {
    fn new(
        world: &'env mut World,
        executor: &'env MultiThreadedExecutor,
        schedule: &'sys mut SystemSchedule,
        scope: &'scope Scope<'scope, 'env, ()>,
        error_handler: fn(EcsError, ErrorContext),
    ) -> Self {
        Self {
            world: world.unsafe_world(),
            executor,
            systems: SyncUnsafeCell::from_mut(schedule.systems.as_mut_slice()).as_slice_of_cells(),
            outgoing: &schedule.outgoing,
            scope,
            error_handler,
        }
    }

    fn push_completed_system(
        &self,
        system_index: u32,
        result: Result<(), Box<dyn Any + Send>>,
        system: &UnitSystem,
    ) {
        // tell the executor that the system finished
        self.executor.completed.push(system_index);
        if let Err(payload) = result {
            cfg::std! {
                ::std::eprintln!("Encountered a panic in system `{}`!", system.name());
            }
            // set the payload to propagate the error
            *self.executor.panic_payload.lock().unwrap() = Some(payload);
        }
        self.tick();
    }

    fn handle_completed_system(&self, state: &mut ExecutorState, system_index: u32) {
        let index = system_index as usize;
        self.outgoing[index].iter().for_each(|&to| {
            let to_index = to as usize;
            let target = &mut state.incoming[to_index];
            *target -= 1;
            if *target == 0 {
                state.ready_systems.push_back(to);
            }
        });
    }

    fn spawn_ready_tasks(&self, state: &mut ExecutorState) {
        state.ready_systems.drain(..).for_each(|index| {
            self.spawn_system_task(index);
        });
    }

    fn spawn_system_task(&self, system_index: u32) {
        let system = &mut unsafe { &mut *self.systems[system_index as usize].get() }.system;
        let non_send = system.is_non_send();
        let name = system.name();
        let context: Context<'scope, 'env, 'sys> = *self;

        let task = async move {
            let func = AssertUnwindSafe(|| unsafe {
                if let Err(e) = system.run((), context.world) {
                    let ctx = ErrorContext::System {
                        name: name.as_str(),
                        last_run: system.get_last_run(),
                    };
                    (context.error_handler)(e, ctx);
                }
            });

            cfg::std! {
                if {
                    let result = ::std::panic::catch_unwind(func);
                    context.push_completed_system(system_index, result, system);
                } else {
                    (f)();
                    context.push_completed_system(system_index, Ok(()), system);
                }
            }
        };

        if non_send {
            vc_utils::cold_path();
            self.scope.spawn_on_external(task);
        } else {
            self.scope.spawn(task);
        }
    }

    fn tick_internal(&self, state: &mut ExecutorState) {
        let completed_queue = &self.executor.completed;
        let mut lock_pop = completed_queue.lock_pop();
        while let Some(system_index) = completed_queue.pop_with_lock(&mut lock_pop) {
            self.handle_completed_system(state, system_index);
        }
        ::core::mem::drop(lock_pop);

        self.spawn_ready_tasks(state);
    }

    fn tick(&self) {
        loop {
            let Ok(mut guard) = self.executor.state.try_lock() else {
                // try_lock failed, there are already other threads doing this.
                return;
            };
            self.tick_internal(&mut guard);
            // Make sure we drop the guard before checking
            // completed.is_empty(), or we could lose events.
            drop(guard);
            // We cannot check `is_empty` before `tick_internal`
            // because the initial tasks without dependencies are
            // in a ready state and not in the queue.
            if self.executor.completed.is_empty() {
                return;
            }
        }
    }
}

impl SystemExecutor for MultiThreadedExecutor {
    fn kind(&self) -> ExecutorKind {
        ExecutorKind::MultiThreaded
    }

    fn init(&mut self, schedule: &SystemSchedule) {
        self.state
            .get_mut()
            .unwrap_or_else(PoisonError::into_inner)
            .init(schedule);
    }

    fn run(
        &mut self,
        schedule: &mut SystemSchedule,
        world: &mut World,
        handler: fn(EcsError, ErrorContext),
    ) {
        if schedule.systems.is_empty() {
            return;
        }

        self.state
            .get_mut()
            .unwrap_or_else(PoisonError::into_inner)
            .reset(schedule);

        let main_thread_executor = world
            .get_resource::<MainThreadExecutor>()
            .map(|e| e.0.clone());
        let external_executor = main_thread_executor.as_deref();

        let task_pool = ComputeTaskPool::get_or_init(TaskPool::default);
        task_pool.scope_with_executor(false, external_executor, |scope| {
            let context = Context::new(world, self, schedule, scope, handler);
            context.tick();
        });

        // check to see if there was a panic
        let payload = self.panic_payload.get_mut().unwrap();
        cfg::std! {
            if {
                if let Some(payload) = payload.take() {
                    ::std::panic::resume_unwind(payload);
                }
            } else {
                assert!(payload.take().is_none());
            }
        }
    }
}

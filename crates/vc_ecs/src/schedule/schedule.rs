use alloc::boxed::Box;
use vc_utils::hash::HashMap;

use crate::error::DefaultErrorHandler;
use crate::schedule::{ExecutorKind, ScheduleLabel};
use crate::schedule::{MultiThreadedExecutor, SingleThreadedExecutor};
use crate::world::World;

use super::{InternedScheduleLabel, SystemExecutor};
use super::{ScheduleGraph, SystemSchedule};

pub struct Schedule {
    label: InternedScheduleLabel,
    graph: ScheduleGraph,
    schedule: SystemSchedule,
    executor: Box<dyn SystemExecutor>,
    executor_initialized: bool,
    is_changed: bool,
}

#[derive(Default)]
pub struct Schedules {
    pub inner: HashMap<InternedScheduleLabel, Schedule>,
}

impl Schedule {
    pub fn new(label: impl ScheduleLabel, kind: ExecutorKind) -> Self {
        Self {
            label: label.intern(),
            graph: ScheduleGraph::default(),
            schedule: SystemSchedule::default(),
            executor: match kind {
                ExecutorKind::SingleThreaded => Box::new(SingleThreadedExecutor::new()),
                ExecutorKind::MultiThreaded => Box::new(MultiThreadedExecutor::new()),
            },
            executor_initialized: false,
            is_changed: false,
        }
    }

    pub fn label(&self) -> InternedScheduleLabel {
        self.label
    }

    pub fn update(&mut self, world: &mut World) {
        if self.is_changed {
            vc_utils::cold_path();
            self.graph.recycle_schedule(&mut self.schedule);
            self.graph.initialize(world);
            self.graph.build_schedule(&mut self.schedule);
            self.is_changed = false;
        }

        if !self.executor_initialized {
            vc_utils::cold_path();
            self.executor.init(&self.schedule);
            self.executor_initialized = true;
        }
    }

    pub fn run(&mut self, world: &mut World) {
        self.update(world);

        if let Some(&handler) = world.get_resource::<DefaultErrorHandler>() {
            self.executor.run(&mut self.schedule, world, handler.0);
        } else {
            vc_utils::cold_path();
            let handler = DefaultErrorHandler::default();
            world.insert_resource(handler);
            self.executor.run(&mut self.schedule, world, handler.0);
        }
    }
}

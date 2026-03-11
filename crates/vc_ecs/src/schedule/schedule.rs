use alloc::boxed::Box;

use crate::error::DefaultErrorHandler;
use crate::schedule::ScheduleLabel;
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

impl Schedule {
    #[expect(unreachable_code, reason = "todo")]
    pub fn new(label: impl ScheduleLabel) -> Self {
        Self {
            label: label.intern(),
            graph: ScheduleGraph::default(),
            schedule: SystemSchedule::default(),
            executor: todo!(),
            executor_initialized: false,
            is_changed: false,
        }
    }

    pub fn label(&self) -> InternedScheduleLabel {
        self.label
    }

    pub fn update_schedule(&mut self, world: &mut World) {
        self.graph.recycle_schedule(&mut self.schedule);
        self.graph.initialize(world);
        self.graph.build_schedule(&mut self.schedule);
    }

    pub fn run(&mut self, world: &mut World) {
        if self.is_changed {
            vc_utils::cold_path();
            self.update_schedule(world);
            self.is_changed = false;
        }

        if !self.executor_initialized {
            vc_utils::cold_path();
            self.executor.init(&self.schedule);
            self.executor_initialized = true;
        }

        if let Some(handler) = world.get_resource::<DefaultErrorHandler>() {
            self.executor.run(&self.schedule, world, handler.0);
        } else {
            vc_utils::cold_path();
            let default_handler = DefaultErrorHandler::default();
            world.insert_resource(default_handler);
            self.executor.run(&self.schedule, world, default_handler.0);
        }
    }
}

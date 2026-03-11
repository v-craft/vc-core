use super::SystemSchedule;
use crate::error::{EcsError, ErrorContext};
use crate::world::World;

pub trait SystemExecutor {
    fn init(&mut self, schedule: &SystemSchedule);
    fn run(
        &mut self,
        schedule: &SystemSchedule,
        world: &mut World,
        handler: fn(EcsError, ErrorContext),
    );
}

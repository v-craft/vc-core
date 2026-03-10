mod graph;
mod label;

pub use graph::*;
pub use label::*;

#[expect(unused, reason = "schedule menu")]
mod temp {
    use alloc::boxed::Box;
    use alloc::vec::Vec;
    use slotmap::{KeyData, SlotMap};
    use vc_utils::hash::{HashMap, HashSet};

    use crate::error::{EcsError, ErrorContext};
    use crate::schedule::{Dag, Direction, GraphNode, InternedScheduleLabel};
    use crate::system::{AccessTable, System};
    use crate::world::World;

    pub struct Schedules {
        inner: HashMap<InternedScheduleLabel, Schedule>,
    }

    pub trait SystemExecutor {
        fn init(&mut self, schedule: &SystemSchedule);
        fn run(
            &mut self,
            schedule: SystemSchedule,
            world: &mut World,
            handler: fn(EcsError, ErrorContext),
        );
    }

    pub struct SystemObject {
        system: Box<dyn System<Input = (), Output = ()>>,
        access: AccessTable,
    }

    slotmap::new_key_type! {
        pub struct SystemKey;
    }

    impl GraphNode for SystemKey {
        type Link = (SystemKey, Direction);
        type Edge = (SystemKey, SystemKey);

        fn name(&self) -> &'static str {
            "system"
        }
    }

    pub struct Systems {
        nodes: SlotMap<SystemKey, Option<SystemObject>>,
        uninit: Vec<SystemKey>,
    }

    pub struct Conflicts {
        exclusive: HashSet<SystemKey>,
        conflicts: HashMap<SystemKey, HashSet<SystemKey>>,
    }

    pub struct ScheduleGraph {
        systems: Systems,
        deps: Dag<SystemKey>,
        conflicts: Conflicts,
        changed: bool,
    }

    pub struct SystemSchedule {
        keys: Vec<SystemKey>,
        systems: Vec<SystemObject>,
        incoming: Vec<usize>,
        outgoing: Vec<Vec<usize>>,
    }

    pub struct Schedule {
        label: InternedScheduleLabel,
        graph: ScheduleGraph,
        schedule: SystemSchedule,
        executor: Box<dyn SystemExecutor>,
        executor_initialized: bool,
    }
}

// -----------------------------------------------------------------------------
// Modules

mod executor;
mod graph;
mod label;
mod schedule;
mod schedules;

// -----------------------------------------------------------------------------
// Exports

pub use executor::{ExecutorKind, MainThreadExecutor, SystemExecutor};
pub use executor::{MultiThreadedExecutor, SingleThreadedExecutor};
pub use graph::{Dag, DagAnalysis, DagGroups, DiGraph, ToposortError, UnGraph};
pub use graph::{DagCrossDependencyError, DagOverlappingGroupError, DagRedundancyError};
pub use graph::{Direction, Graph, GraphNode, SccIterator, SccNodes};
pub use label::{InternedScheduleLabel, ScheduleLabel};
pub use schedule::{Schedule, SystemKey, SystemObject, SystemSchedule};
pub use schedules::Schedules;

// -----------------------------------------------------------------------------
// Tests

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::{Component, ComponentStorage};
    use crate::query::{And, Or, With, Without};
    use crate::system::{FunctionSystem, SystemName};
    use crate::world::{World, WorldIdAllocator};
    use alloc::boxed::Box;
    use alloc::string::String;
    use alloc::vec::Vec;

    #[derive(Debug, PartialEq, Eq)]
    struct Foo;

    #[derive(Debug, PartialEq, Eq)]
    struct Bar(u64);

    #[derive(Debug, PartialEq, Eq)]
    struct Baz(String);

    #[derive(Debug, PartialEq)]
    struct Qux(f32);

    #[derive(Debug, PartialEq, Eq)]
    struct Zaz(i32);

    unsafe impl Component for Foo {}
    unsafe impl Component for Bar {}
    unsafe impl Component for Baz {
        const STORAGE: ComponentStorage = ComponentStorage::Sparse;
    }
    unsafe impl Component for Qux {}
    unsafe impl Component for Zaz {}

    #[derive(Debug, Hash, PartialEq, Eq)]
    pub struct Testing;

    impl ScheduleLabel for Testing {
        fn dyn_clone(&self) -> Box<dyn ScheduleLabel> {
            Box::new(Testing)
        }
    }

    fn alloc_world() -> Box<World> {
        static ALLOCATOR: WorldIdAllocator = WorldIdAllocator::new();
        World::new(ALLOCATOR.alloc())
    }

    #[test]
    fn basic() {
        let mut world = alloc_world();
        let mut schedules = Schedules::new();
        let name = SystemName::new("spawn_entities");
        let system = FunctionSystem::new(spawn_entities, name);

        schedules.insert_system(Testing, name, system);
        schedules.entry(Testing).run(&mut world);

        let query =
            world.query_with::<&Foo, And<(With<Bar>, Without<Baz>, Or<(With<Qux>, With<Zaz>)>)>>();

        assert_eq!(query.into_iter().count(), 1);

        let query = world.query::<&Qux>();
        let qux_values: Vec<f32> = query.into_iter().map(|q| q.0).collect();
        assert!(qux_values.contains(&3.0));
    }

    fn spawn_entities(world: &mut World) -> () {
        world.spawn((Foo, Bar(100), Baz(String::from("a")), Qux(1.0)));
        world.spawn((Foo, Bar(200), Baz(String::from("b"))));
        world.spawn((Foo, Bar(300), Qux(3.0)));
        world.spawn((Foo, Baz(String::from("c")), Qux(4.0)));
        world.spawn((Foo, Zaz(42)));
    }
}

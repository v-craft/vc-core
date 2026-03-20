#![expect(clippy::module_inception, reason = "For better structure.")]

use core::fmt::Debug;

use alloc::boxed::Box;
use alloc::vec::Vec;

use fixedbitset::FixedBitSet;
use slotmap::{SecondaryMap, SlotMap};
use vc_utils::hash::{HashMap, HashSet, NoOpHashMap};

use super::{Dag, SystemKey, SystemObject, UnitSystem};
use super::{ExecutorKind, MultiThreadedExecutor, SingleThreadedExecutor};
use super::{InternedScheduleLabel, ScheduleLabel, SystemExecutor};
use crate::error::DefaultErrorHandler;
use crate::system::SystemName;
use crate::world::World;

// -----------------------------------------------------------------------------
// Schedule

pub struct Schedule {
    label: InternedScheduleLabel,
    allocator: Allocator,
    buffer: SystemBuffer,
    ordering: OrderingGraph,
    conflict: ConflictTable,
    schedule: SystemSchedule,
    executor: Box<dyn SystemExecutor>,
    executor_initialized: bool,
    is_changed: bool,
}

// -----------------------------------------------------------------------------
// Allocator

#[derive(Default)]
struct Allocator {
    slots: SlotMap<SystemKey, SystemName>,
    names: NoOpHashMap<SystemName, SystemKey>,
}

// -----------------------------------------------------------------------------
// SystemBuffer

#[derive(Default)]
struct SystemBuffer {
    nodes: SecondaryMap<SystemKey, Option<SystemObject>>,
    uninit: Vec<SystemKey>,
}

// -----------------------------------------------------------------------------
// OrderingGraph

#[derive(Default)]
struct OrderingGraph {
    ordering: Dag<SystemKey>,
}

// -----------------------------------------------------------------------------
// ConflictTable

#[derive(Default)]
struct ConflictTable {
    exclusive: HashSet<SystemKey>,
    conflicts: HashMap<SystemKey, HashSet<SystemKey>>,
}

// -----------------------------------------------------------------------------
// SystemSchedule

#[derive(Default)]
pub struct SystemSchedule {
    pub keys: Vec<SystemKey>,
    pub systems: Vec<SystemObject>,
    pub incoming: Vec<u16>,
    pub outgoing: Vec<Vec<u16>>,
}

// -----------------------------------------------------------------------------
// Allocator Implementation

impl Allocator {
    fn iter(&self) -> impl ExactSizeIterator<Item = (&SystemName, &SystemKey)> + '_ {
        self.names.iter()
    }

    fn contains(&self, name: SystemName) -> bool {
        self.names.contains_key(&name)
    }

    fn get_key(&self, name: SystemName) -> Option<SystemKey> {
        self.names.get(&name).copied()
    }

    fn get_name(&self, key: SystemKey) -> Option<SystemName> {
        self.slots.get(key).copied()
    }

    fn insert(&mut self, name: SystemName) -> SystemKey {
        self.names.get(&name).copied().unwrap_or_else(|| {
            let key = self.slots.insert(name);
            self.names.insert(name, key);
            key
        })
    }

    fn remove(&mut self, name: SystemName) -> Option<SystemKey> {
        let key = self.names.remove(&name)?;
        let removed = self.slots.remove(key);
        debug_assert_eq!(removed, Some(name));
        Some(key)
    }
}

// -----------------------------------------------------------------------------
// SystemBuffer Implementation

impl SystemBuffer {
    fn insert(&mut self, key: SystemKey, system: UnitSystem) {
        let obj = SystemObject::new_uninit(system);
        self.nodes.insert(key, Some(obj));
        self.uninit.push(key);
    }

    fn remove(&mut self, key: SystemKey) {
        self.nodes.remove(key);

        if let Some(index) = self.uninit.iter().position(|value| *value == key) {
            self.uninit.swap_remove(index);
        }
    }

    fn get_system(&self, key: SystemKey) -> Option<&SystemObject> {
        self.nodes.get(key).and_then(Option::as_ref)
    }

    fn get_system_mut(&mut self, key: SystemKey) -> Option<&mut SystemObject> {
        self.nodes.get_mut(key).and_then(Option::as_mut)
    }

    fn take_system(&mut self, key: SystemKey) -> SystemObject {
        self.nodes.get_mut(key).unwrap().take().unwrap()
    }
}

// -----------------------------------------------------------------------------
// OrderingGraph Implementation

impl OrderingGraph {
    fn insert(&mut self, a: SystemKey, b: SystemKey) {
        self.ordering.insert_edge(a, b);
    }

    fn remove(&mut self, a: SystemKey, b: SystemKey) -> bool {
        self.ordering.remove_edge(a, b)
    }

    fn insert_node(&mut self, key: SystemKey) {
        self.ordering.insert_node(key)
    }

    fn remove_node(&mut self, key: SystemKey) {
        self.ordering.remove_node(key)
    }
}

// -----------------------------------------------------------------------------
// ConflictTable Implementation

impl ConflictTable {
    fn set_exclusive(&mut self, key: SystemKey) {
        self.exclusive.insert(key);
    }

    fn set_conflict(&mut self, a: SystemKey, b: SystemKey) {
        self.conflicts.entry(a).or_default().insert(b);
        self.conflicts.entry(b).or_default().insert(a);
    }

    fn is_exclusive(&self, key: SystemKey) -> bool {
        self.exclusive.contains(&key)
    }

    fn is_conflict(&self, a: SystemKey, b: SystemKey) -> bool {
        self.conflicts.get(&a).is_some_and(|set| set.contains(&b))
    }

    fn remove(&mut self, key: SystemKey) {
        self.exclusive.remove(&key);
        if let Some(a_set) = self.conflicts.remove(&key) {
            a_set.iter().for_each(|b| {
                if let Some(b_set) = self.conflicts.get_mut(b) {
                    b_set.remove(&key);
                }
            });
        }
    }
}

// -----------------------------------------------------------------------------
// Schedule Implementation

impl Debug for Schedule {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Schedule")
            .field("label", &self.label)
            .field("systems", &self.allocator.names.keys())
            .finish()
    }
}

impl Schedule {
    fn init_systems(&mut self, world: &mut World) {
        let buffer = &mut self.buffer;
        let conflict = &mut self.conflict;

        let uninit = core::mem::take(&mut buffer.uninit);

        uninit.iter().for_each(|&key| {
            if let Some(obj) = buffer.get_system_mut(key) {
                obj.access = obj.system.initialize(world);
            } else {
                vc_utils::cold_path();
                unreachable!();
            }
        });

        uninit.iter().for_each(|&a| {
            if let Some(obj) = buffer.get_system(a) {
                if obj.system.is_exclusive() {
                    conflict.set_exclusive(a);
                } else {
                    for (b, v) in buffer.nodes.iter() {
                        if let Some(v) = v
                            && a != b
                            && !obj.access.parallelizable(&v.access)
                        {
                            conflict.set_conflict(a, b);
                        }
                    }
                }
            } else {
                vc_utils::cold_path();
                unreachable!();
            }
        });
    }

    fn recycle_schedule(&mut self) {
        let schedule = &mut self.schedule;
        let buffer = &mut self.buffer;
        schedule.incoming.clear();
        schedule.outgoing.clear();
        schedule
            .keys
            .drain(..)
            .zip(schedule.systems.drain(..))
            .for_each(|(k, v)| {
                *buffer.nodes.get_mut(k).unwrap() = Some(v);
            });
    }

    fn build_schedule(&mut self) {
        let buffer = &mut self.buffer;
        let schedule = &mut self.schedule;
        let conflict = &mut self.conflict;
        let ordering = &mut self.ordering;
        assert!(schedule.keys.is_empty() && schedule.systems.is_empty());
        assert!(schedule.outgoing.is_empty() && schedule.incoming.is_empty());

        let mut dag = transitive_reduction(conflict, ordering);

        schedule.keys.extend(dag.toposort().unwrap());
        let topo: &[SystemKey] = &schedule.keys;

        schedule
            .systems
            .extend(topo.iter().map(|&key| buffer.take_system(key)));
        debug_assert_eq!(schedule.keys.len(), schedule.systems.len());

        schedule.incoming.resize(topo.len(), 0);
        schedule.outgoing.resize(topo.len(), Vec::new());

        let mut indices: HashMap<SystemKey, usize> = HashMap::with_capacity(topo.len());
        topo.iter().enumerate().for_each(|(idx, &key)| {
            indices.insert(key, idx);
        });

        topo.iter().enumerate().for_each(|(idx, &key)| {
            dag.neighbors(key).for_each(|to| {
                let neighbor_index = indices[&to];
                schedule.incoming[neighbor_index] += 1;
                schedule.outgoing[idx].push(neighbor_index as u16);
            });
        });
    }

    fn update(&mut self, world: &mut World) {
        if self.is_changed {
            vc_utils::cold_path();
            // self.recycle_schedule();
            self.init_systems(world);
            self.build_schedule();
            self.is_changed = false;
        }

        if !self.executor_initialized {
            vc_utils::cold_path();
            self.executor.init(&self.schedule);
            self.executor_initialized = true;
        }
    }

    pub fn new(label: impl ScheduleLabel) -> Self {
        Self {
            label: label.intern(),
            executor: match ExecutorKind::default() {
                ExecutorKind::SingleThreaded => Box::new(SingleThreadedExecutor::new()),
                ExecutorKind::MultiThreaded => Box::new(MultiThreadedExecutor::new()),
            },
            executor_initialized: false,
            is_changed: false,
            allocator: Default::default(),
            buffer: Default::default(),
            ordering: Default::default(),
            conflict: Default::default(),
            schedule: Default::default(),
        }
    }

    pub fn label(&self) -> InternedScheduleLabel {
        self.label
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

    pub fn contains(&self, name: SystemName) -> bool {
        self.allocator.contains(name)
    }

    pub fn remove(&mut self, name: SystemName) -> bool {
        let Some(key) = self.allocator.remove(name) else {
            return false;
        };

        if !self.is_changed {
            self.recycle_schedule();
            self.is_changed = true;
        }

        self.buffer.remove(key);
        self.ordering.remove_node(key);
        self.conflict.remove(key);

        true
    }

    pub fn insert(&mut self, name: SystemName, system: UnitSystem) -> bool {
        if !self.is_changed {
            self.recycle_schedule();
            self.is_changed = true;
        }

        if let Some(key) = self.allocator.get_key(name) {
            log::info!("Insert system {} repeatedly.", system.name());
            self.buffer.remove(key);
            self.buffer.insert(key, system);
            let len = self.allocator.names.len();
            assert!(len <= u16::MAX as usize, "too many systems in a schedule");
            false
        } else {
            let key = self.allocator.insert(name);
            self.buffer.insert(key, system);
            self.ordering.insert_node(key);
            let len = self.allocator.names.len();
            assert!(len <= u16::MAX as usize, "too many systems in a schedule");
            true
        }
    }

    pub fn insert_order(&mut self, before: SystemName, after: SystemName) -> bool {
        let Some(a) = self.allocator.get_key(before) else {
            return false;
        };
        let Some(b) = self.allocator.get_key(after) else {
            return false;
        };

        if !self.is_changed {
            self.recycle_schedule();
            self.is_changed = true;
        }

        self.ordering.insert(a, b);

        true
    }

    pub fn remove_order(&mut self, before: SystemName, after: SystemName) -> bool {
        let Some(a) = self.allocator.get_key(before) else {
            return false;
        };
        let Some(b) = self.allocator.get_key(after) else {
            return false;
        };

        if !self.is_changed {
            self.recycle_schedule();
            self.is_changed = true;
        }

        self.ordering.remove(a, b)
    }

    pub fn get_key(&self, name: SystemName) -> Option<SystemKey> {
        self.allocator.get_key(name)
    }

    pub fn get_name(&self, key: SystemKey) -> Option<SystemName> {
        self.allocator.get_name(key)
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = (&SystemName, &SystemKey)> + '_ {
        self.allocator.iter()
    }

    pub fn order_graph(&self) -> &Dag<SystemKey> {
        &self.ordering.ordering
    }
}

fn transitive_reduction(conflict: &ConflictTable, ordering: &mut OrderingGraph) -> Dag<SystemKey> {
    const fn bind_index(row: usize, col: usize) -> usize {
        // 0
        // 1 2
        // 3 4 5
        ((row * (row + 1)) >> 1) + col
    }

    let (topo, graph) = ordering.ordering.toposort_and_graph().unwrap();
    debug_assert!(topo.len() <= u16::MAX as usize);
    if topo.is_empty() {
        return Dag::new();
    }

    let mut exec_dag = graph.clone();
    let mut index_map = HashMap::<SystemKey, usize>::with_capacity(topo.len());
    index_map.extend(topo.iter().enumerate().map(|(idx, &key)| (key, idx)));

    let system_count = topo.len();
    let mut exclusive_systems = FixedBitSet::with_capacity(system_count);
    let matrix_size = system_count * (system_count + 1) / 2;
    let mut transitive_closure = FixedBitSet::with_capacity(matrix_size);
    topo.iter().enumerate().for_each(|(ib, &kb)| {
        let b_is_exclusive = conflict.is_exclusive(kb);
        if b_is_exclusive {
            unsafe {
                exclusive_systems.insert_unchecked(ib);
            }
        }

        exec_dag.neighbors(kb).for_each(|km| {
            let im = *index_map.get(&km).unwrap();
            unsafe {
                transitive_closure.insert_unchecked(bind_index(im, ib));
            }
        });

        topo[0..ib].iter().enumerate().rev().for_each(|(ia, &ka)| {
            let matrix_index = bind_index(ia, ib);
            if unsafe { transitive_closure.contains_unchecked(matrix_index) } {
                return;
            }

            if b_is_exclusive
                || unsafe { exclusive_systems.contains_unchecked(ia) }
                || conflict.is_conflict(ka, kb)
            {
                unsafe {
                    transitive_closure.insert_unchecked(matrix_index);
                }
                let is_unreachable = exec_dag.neighbors(kb).all(|km| {
                    let im = *index_map.get(&km).unwrap();
                    unsafe { !transitive_closure.contains_unchecked(bind_index(im, ib)) }
                });

                if is_unreachable {
                    exec_dag.insert_edge(ka, kb);
                }
            }
        });
    });

    exec_dag.into()
}

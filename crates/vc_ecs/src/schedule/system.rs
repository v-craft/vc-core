use alloc::boxed::Box;
use alloc::vec::Vec;
use fixedbitset::FixedBitSet;
use slotmap::SlotMap;
use vc_utils::hash::{HashMap, HashSet};

use crate::schedule::{Dag, Direction, GraphNode};
use crate::system::{AccessTable, System};
use crate::world::World;

// -----------------------------------------------------------------------------
// SystemKey

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

// -----------------------------------------------------------------------------
// SystemObject

type UnitSystem = Box<dyn System<Input = (), Output = ()>>;

pub struct SystemObject {
    pub system: UnitSystem,
    pub access: AccessTable,
}

impl SystemObject {
    #[inline]
    pub fn new_uninit(system: UnitSystem) -> Self {
        Self {
            system,
            access: AccessTable::new(),
        }
    }
}

// -----------------------------------------------------------------------------
// Systems

#[derive(Default)]
pub struct Systems {
    nodes: SlotMap<SystemKey, Option<SystemObject>>,
    uninit: Vec<SystemKey>,
}

#[derive(Default)]
pub struct Conflicts {
    exclusive: HashSet<SystemKey>,
    conflicts: HashMap<SystemKey, HashSet<SystemKey>>,
}

#[derive(Default)]
pub struct ScheduleGraph {
    systems: Systems,
    deps: Dag<SystemKey>,
    conflicts: Conflicts,
}

#[derive(Default)]
pub struct SystemSchedule {
    pub keys: Vec<SystemKey>,
    pub systems: Vec<SystemObject>,
    pub incoming: Vec<usize>,
    pub outgoing: Vec<Vec<usize>>,
}

// -----------------------------------------------------------------------------
// Implementation

impl Systems {
    pub fn is_initialized(&self) -> bool {
        self.uninit.is_empty()
    }

    pub fn initialize(&mut self, world: &mut World) {
        self.uninit.drain(..).for_each(|key| {
            if let Some(obj) = self.nodes.get_mut(key).and_then(Option::as_mut) {
                obj.access = obj.system.initialize(world);
            } else {
                vc_utils::cold_path();
            }
        });
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn insert(&mut self, system: UnitSystem) -> SystemKey {
        let key = self.nodes.insert(Some(SystemObject::new_uninit(system)));
        self.uninit.push(key);
        key
    }

    pub fn remove(&mut self, key: SystemKey) -> bool {
        let mut found = false;
        if self.nodes.remove(key).is_some() {
            found = true;
        }

        if let Some(index) = self.uninit.iter().position(|value| *value == key) {
            self.uninit.swap_remove(index);
            found = true;
        }

        found
    }

    pub fn get(&self, key: SystemKey) -> Option<&SystemObject> {
        self.nodes.get(key).and_then(Option::as_ref)
    }

    pub fn get_node_mut(&mut self, key: SystemKey) -> Option<&mut Option<SystemObject>> {
        self.nodes.get_mut(key)
    }
}

impl Conflicts {
    pub fn set_exclusive(&mut self, key: SystemKey) {
        self.exclusive.insert(key);
    }

    pub fn set_conflict(&mut self, a: SystemKey, b: SystemKey) {
        self.conflicts.entry(a).or_default().insert(b);
        self.conflicts.entry(b).or_default().insert(a);
    }

    pub fn remove(&mut self, key: SystemKey) {
        self.exclusive.remove(&key);
        if let Some(a_set) = self.conflicts.remove(&key) {
            a_set.iter().for_each(|b| {
                if let Some(b_set) = self.conflicts.get_mut(b) {
                    b_set.remove(&key);
                }
            });
        }
    }

    pub fn is_exclusive(&self, key: SystemKey) -> bool {
        self.exclusive.contains(&key)
    }

    pub fn is_conflict(&self, a: SystemKey, b: SystemKey) -> bool {
        self.conflicts.get(&a).is_some_and(|set| set.contains(&b))
    }
}

impl ScheduleGraph {
    pub fn initialize(&mut self, world: &mut World) {
        let uninit = self.systems.uninit.clone();
        self.systems.initialize(world);

        for a in uninit {
            if let Some(obj) = self.systems.get(a) {
                if obj.system.is_exclusive() {
                    self.conflicts.set_exclusive(a);
                } else {
                    for (b, v) in self.systems.nodes.iter() {
                        if let Some(v) = v
                            && a != b
                            && !obj.access.parallelizable(&v.access)
                        {
                            self.conflicts.set_conflict(a, b);
                        }
                    }
                }
            }
        }
    }

    pub fn set_before(&mut self, a: SystemKey, b: SystemKey) {
        self.deps.insert_edge(a, b);
    }

    pub fn set_after(&mut self, a: SystemKey, b: SystemKey) {
        self.deps.insert_edge(b, a)
    }

    pub fn insert(&mut self, system: UnitSystem) -> SystemKey {
        let key = self.systems.insert(system);
        self.deps.insert_node(key);
        key
    }

    pub fn remove(&mut self, key: SystemKey) -> bool {
        self.deps.remove_node(key);
        self.conflicts.remove(key);
        self.systems.remove(key)
    }

    pub fn recycle_schedule(&mut self, schedule: &mut SystemSchedule) {
        schedule.incoming.clear();
        schedule.outgoing.clear();
        schedule
            .keys
            .drain(..)
            .zip(schedule.systems.drain(..))
            .for_each(|(k, v)| {
                *self.systems.nodes.get_mut(k).unwrap() = Some(v);
            });
    }

    pub fn build_schedule(&mut self, schedule: &mut SystemSchedule) {
        assert!(schedule.keys.is_empty() && schedule.systems.is_empty());
        assert!(schedule.incoming.is_empty() && schedule.outgoing.is_empty());

        let mut exec_dag = self.deps.clone();
        let old_topo = self.deps.toposort().unwrap();

        let mut is_exclusive = FixedBitSet::with_capacity(old_topo.len());
        for (idx, &key) in old_topo.iter().enumerate() {
            if self.conflicts.is_exclusive(key) {
                is_exclusive.insert(idx);
            }
        }

        // `deref_mut` will modify `is_dirty` field, so we cache it.
        let digraph = exec_dag.graph_mut();
        for (idx_a, &key_a) in old_topo.iter().enumerate() {
            let a_is_exclusive = unsafe { is_exclusive.contains_unchecked(idx_a) };
            for (idx_b, &key_b) in old_topo[0..idx_a].iter().enumerate() {
                if a_is_exclusive
                    || unsafe { is_exclusive.contains_unchecked(idx_b) }
                    || self.conflicts.is_conflict(key_a, key_b)
                {
                    digraph.insert_edge(key_a, key_b);
                }
            }
        }

        schedule.keys.extend(exec_dag.toposort().unwrap());
        let topo: &[SystemKey] = &schedule.keys;

        schedule.systems.extend(
            topo.iter()
                .map(|&key| self.systems.get_node_mut(key).unwrap().take().unwrap()),
        );

        schedule.incoming.resize(topo.len(), 0);
        schedule.outgoing.resize(topo.len(), Vec::new());

        let mut key_index: HashMap<SystemKey, usize> = HashMap::with_capacity(topo.len());
        topo.iter().enumerate().for_each(|(idx, &key)| {
            key_index.insert(key, idx);
        });

        let reduced = exec_dag.transitive_reduction(topo, &key_index);
        topo.iter().enumerate().for_each(|(idx, &key)| {
            reduced.neighbors(key).for_each(|to| {
                let neighbor_index = key_index[&to];
                schedule.outgoing[idx].push(neighbor_index);
                schedule.incoming[neighbor_index] += 1;
            });
        });
    }
}

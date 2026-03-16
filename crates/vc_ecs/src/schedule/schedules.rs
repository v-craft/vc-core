use alloc::boxed::Box;

use vc_utils::hash::HashMap;

use super::{InternedScheduleLabel, Schedule, ScheduleLabel};
use crate::resource::Resource;
use crate::system::{IntoSystem, SystemName};

pub struct Schedules {
    mapper: HashMap<InternedScheduleLabel, Schedule>,
}

impl Default for Schedules {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl Resource for Schedules {}

impl Schedules {
    pub const fn new() -> Self {
        Self {
            mapper: HashMap::new(),
        }
    }

    pub fn insert(&mut self, schedule: Schedule) -> Option<Schedule> {
        self.mapper.insert(schedule.label(), schedule)
    }

    /// Removes the schedule corresponding to the `label` from the map, returning it if it existed.
    pub fn remove(&mut self, label: impl ScheduleLabel) -> Option<Schedule> {
        self.mapper.remove(&label.intern())
    }

    /// Return true if the provided label already exist.
    pub fn contains(&self, label: impl ScheduleLabel) -> bool {
        self.mapper.contains_key(&label.intern())
    }

    /// Returns a reference to the schedule associated with `label`, if it exists.
    pub fn get(&self, label: impl ScheduleLabel) -> Option<&Schedule> {
        self.mapper.get(&label.intern())
    }

    /// Returns a mutable reference to the schedule associated with `label`, if it exists.
    pub fn get_mut(&mut self, label: impl ScheduleLabel) -> Option<&mut Schedule> {
        self.mapper.get_mut(&label.intern())
    }

    /// Returns a mutable reference to the schedules associated with `label`,
    /// creating one if it doesn't already exist.
    pub fn entry(&mut self, label: impl ScheduleLabel) -> &mut Schedule {
        self.mapper
            .entry(label.intern())
            .or_insert_with(|| Schedule::new(label))
    }

    /// Returns an iterator over all schedules. Iteration order is undefined.
    pub fn iter(&self) -> impl Iterator<Item = (&dyn ScheduleLabel, &Schedule)> {
        self.mapper
            .iter()
            .map(|(label, schedule)| (&**label, schedule))
    }

    /// Returns an iterator over mutable references to all schedules. Iteration order is undefined.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&dyn ScheduleLabel, &mut Schedule)> {
        self.mapper
            .iter_mut()
            .map(|(label, schedule)| (&**label, schedule))
    }

    pub fn insert_system(
        &mut self,
        label: impl ScheduleLabel,
        name: SystemName,
        system: impl IntoSystem<(), ()>,
    ) -> &mut Self {
        self.entry(label)
            .insert(name, Box::new(IntoSystem::into_system(system)));

        self
    }

    pub fn remove_system(&mut self, label: impl ScheduleLabel, name: SystemName) -> bool {
        self.entry(label).remove(name)
    }
}

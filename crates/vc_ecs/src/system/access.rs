use core::fmt::Debug;

use fixedbitset::FixedBitSet;
use vc_utils::hash::NoOpHashMap;

use super::{FilterData, FilterParam};
use crate::resource::ResourceId;

#[derive(Default)]
pub struct AccessTable {
    world_mut: bool,          // holding `&mut world`
    world_ref: bool,          // holding `&world`
    res_reading: FixedBitSet, // resource reading
    res_writing: FixedBitSet, // resource writing
    filter: NoOpHashMap<FilterParam, FilterData>,
}

// `#[derive(Clone)]` does not generate optimized `clone_from`.
impl Clone for AccessTable {
    fn clone(&self) -> Self {
        Self {
            world_mut: self.world_mut,
            world_ref: self.world_ref,
            res_reading: self.res_reading.clone(),
            res_writing: self.res_writing.clone(),
            filter: self.filter.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.world_mut = source.world_mut;
        self.world_ref = source.world_ref;
        self.res_reading.clone_from(&source.res_reading);
        self.res_writing.clone_from(&source.res_writing);
        self.filter.clone_from(&source.filter);
    }
}

impl Debug for AccessTable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        struct FormattedBitSet<'a>(&'a FixedBitSet);
        impl Debug for FormattedBitSet<'_> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_list().entries(self.0.ones()).finish()
            }
        }

        f.debug_struct("AccessTable")
            .field("world_mut", &self.world_mut)
            .field("world_ref", &self.world_ref)
            .field("res_reading", &FormattedBitSet(&self.res_reading))
            .field("res_writing", &FormattedBitSet(&self.res_writing))
            .finish()
    }
}

impl AccessTable {
    /// Creates an empty [`AccessTable`] collection.
    pub const fn new() -> Self {
        Self {
            world_mut: false,
            world_ref: false,
            res_reading: FixedBitSet::new(),
            res_writing: FixedBitSet::new(),
            filter: NoOpHashMap::new(),
        }
    }

    pub fn can_world_mut(&self) -> bool {
        !self.world_mut
            && !self.world_ref
            && self.res_reading.is_clear()
            && self.res_writing.is_clear()
            && self.filter.is_empty()
    }

    pub fn can_world_ref(&self) -> bool {
        self.world_ref
            || (!self.world_mut
                && self.res_writing.is_clear()
                && self.filter.values().all(FilterData::is_read_only))
    }

    pub fn set_world_mut(&mut self) -> bool {
        if self.can_world_mut() {
            *self = const { Self::new() };
            self.world_mut = true;
            true
        } else {
            false
        }
    }

    pub fn set_world_ref(&mut self) -> bool {
        if self.can_world_ref() {
            if !self.world_ref {
                *self = const { Self::new() };
                self.world_ref = true;
            }
            true
        } else {
            false
        }
    }

    pub fn can_reading_res(&self, id: ResourceId) -> bool {
        self.world_ref || (!self.world_mut && !self.res_writing.contains(id.index()))
    }

    pub fn can_writing_res(&self, id: ResourceId) -> bool {
        !self.world_ref && !self.world_mut && !self.res_reading.contains(id.index())
    }

    pub fn set_reading_res(&mut self, id: ResourceId) -> bool {
        if self.can_reading_res(id) {
            if !self.world_ref {
                self.res_reading.grow_and_insert(id.index());
            }
            true
        } else {
            false
        }
    }

    pub fn set_writing_res(&mut self, id: ResourceId) -> bool {
        if self.can_writing_res(id) {
            let index = id.index();
            self.res_reading.grow_and_insert(index);
            self.res_writing.grow_and_insert(index);
            true
        } else {
            false
        }
    }

    pub fn can_query(&self, data: &FilterData, params: &[FilterParam]) -> bool {
        if self.world_mut {
            return false;
        }
        if self.world_ref {
            return data.is_read_only();
        }
        params.iter().all(|param| {
            self.filter.iter().all(|(k, v)| {
                if k.is_disjoint(param) {
                    true
                } else {
                    data.parallelizable(v)
                }
            })
        })
    }

    pub fn set_query(&mut self, data: &FilterData, params: &[FilterParam]) -> bool {
        if self.can_query(data, params) {
            if !self.world_ref {
                params.iter().for_each(|param| {
                    if let Some(item) = self.filter.get_mut(param) {
                        item.merge(data);
                    } else {
                        self.filter.insert(param.clone(), data.clone());
                    }
                });
            }
            true
        } else {
            false
        }
    }

    pub fn parallelizable(&self, other: &Self) -> bool {
        if self.world_mut || other.world_mut {
            return false;
        }
        if self.world_ref && other.world_ref {
            return true;
        }
        if !self.res_writing.is_disjoint(&other.res_reading)
            || !other.res_writing.is_disjoint(&self.res_reading)
        {
            return false;
        }
        self.filter.iter().all(|(k, v)| {
            other.filter.iter().all(|(x, y)| {
                if k.is_disjoint(x) {
                    true
                } else {
                    v.parallelizable(y)
                }
            })
        })
    }
}

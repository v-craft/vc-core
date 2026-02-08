#![allow(clippy::new_without_default, reason = "internal type")]

use alloc::vec::Vec;
use core::fmt::Debug;

use super::{NoSendData, ResourceData};
use crate::component::ComponentId;
use crate::storage::ComponentIndices;
use crate::tick::{CheckTicks, Tick};

// -----------------------------------------------------------------------------
// Resources

pub struct Resources {
    resources: Vec<ResourceData>,
    ids: Vec<ComponentId>,
    indices: ComponentIndices,
}

impl Debug for Resources {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_map()
            .entries(self.ids.iter().zip(self.resources.iter()))
            .finish()
    }
}

impl Drop for Resources {
    fn drop(&mut self) {
        self.resources.iter_mut().for_each(|data| {
            if data.is_valid() {
                data.drop_data();
            }
        });
    }
}

// -----------------------------------------------------------------------------
// NoSends

pub struct NoSends {
    no_sends: Vec<NoSendData>,
    ids: Vec<ComponentId>,
    indices: ComponentIndices,
}

impl Debug for NoSends {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_map()
            .entries(self.ids.iter().zip(self.no_sends.iter()))
            .finish()
    }
}

impl Drop for NoSends {
    fn drop(&mut self) {
        self.no_sends.iter_mut().for_each(|data| {
            if data.is_valid() {
                data.drop_data();
            }
        });
    }
}

// -----------------------------------------------------------------------------
// Basic methods

impl Resources {
    #[inline]
    pub(crate) const fn new() -> Self {
        Self {
            resources: Vec::new(),
            ids: Vec::new(),
            indices: ComponentIndices::new(),
        }
    }

    #[inline]
    pub fn get(&self, id: ComponentId) -> Option<&ResourceData> {
        self.indices.map(id, |index| unsafe {
            self.resources.get_unchecked(index as usize)
        })
    }

    #[inline]
    pub fn get_mut(&mut self, id: ComponentId) -> Option<&mut ResourceData> {
        self.indices.map(id, |index| unsafe {
            self.resources.get_unchecked_mut(index as usize)
        })
    }

    #[inline]
    pub fn check_ticks(&mut self, check: CheckTicks) {
        let now = check.tick();
        let fall_back = now.relative_to(Tick::MAX_AGE);
        self.resources.iter_mut().for_each(move |data| {
            data.quick_check(now, fall_back);
        });
    }
}

impl NoSends {
    #[inline]
    pub(crate) const fn new() -> Self {
        Self {
            no_sends: Vec::new(),
            ids: Vec::new(),
            indices: ComponentIndices::new(),
        }
    }

    #[inline]
    pub fn get(&self, id: ComponentId) -> Option<&NoSendData> {
        self.indices.map(id, |index| unsafe {
            self.no_sends.get_unchecked(index as usize)
        })
    }

    #[inline]
    pub fn get_mut(&mut self, id: ComponentId) -> Option<&mut NoSendData> {
        self.indices.map(id, |index| unsafe {
            self.no_sends.get_unchecked_mut(index as usize)
        })
    }

    #[inline]
    pub fn check_ticks(&mut self, check: CheckTicks) {
        let now = check.tick();
        let fall_back = now.relative_to(Tick::MAX_AGE);
        self.no_sends.iter_mut().for_each(move |data| {
            data.quick_check(now, fall_back);
        });
    }
}

// -----------------------------------------------------------------------------
// Prepare

use crate::component::ComponentInfo;

impl Resources {
    pub(crate) fn prepare(&mut self, info: &ComponentInfo) {
        let id = info.id();
        if !self.indices.contains(id) {
            unsafe {
                let data = ResourceData::new(info.debug_name(), info.layout(), info.drop_fn());

                let index = self.resources.len() as u32;
                self.resources.push(data);
                self.ids.push(id);
                self.indices.set(id, index);
            }
        }
    }
}

impl NoSends {
    pub(crate) fn prepare(&mut self, info: &ComponentInfo) {
        let id = info.id();
        if !self.indices.contains(id) {
            unsafe {
                let data = NoSendData::new(info.debug_name(), info.layout(), info.drop_fn());

                let index = self.no_sends.len() as u32;
                self.no_sends.push(data);
                self.ids.push(id);
                self.indices.set(id, index);
            }
        }
    }
}

#![allow(clippy::new_without_default, reason = "internal type")]

use alloc::vec::Vec;
use core::fmt::Debug;

use super::{NonSendData, ResourceData};
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

// -----------------------------------------------------------------------------
// NonSends

pub struct NonSends {
    non_sends: Vec<NonSendData>,
    ids: Vec<ComponentId>,
    indices: ComponentIndices,
}

impl Debug for NonSends {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_map()
            .entries(self.ids.iter().zip(self.non_sends.iter()))
            .finish()
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
    pub unsafe fn get(&self, id: ComponentId) -> &ResourceData {
        unsafe {
            let index = self.indices.get_unchecked(id);
            self.resources.get_unchecked(index as usize)
        }
    }

    #[inline]
    pub unsafe fn get_mut(&mut self, id: ComponentId) -> &mut ResourceData {
        unsafe {
            let index = self.indices.get_unchecked(id);
            self.resources.get_unchecked_mut(index as usize)
        }
    }

    pub fn try_get(&self, id: ComponentId) -> Option<&ResourceData> {
        self.indices.map(id, |index| unsafe {
            self.resources.get_unchecked(index as usize)
        })
    }

    pub fn try_get_mut(&self, id: ComponentId) -> Option<&ResourceData> {
        self.indices.map(id, |index| unsafe {
            self.resources.get_unchecked(index as usize)
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

impl NonSends {
    #[inline]
    pub(crate) const fn new() -> Self {
        Self {
            non_sends: Vec::new(),
            ids: Vec::new(),
            indices: ComponentIndices::new(),
        }
    }

    #[inline]
    pub unsafe fn get(&self, id: ComponentId) -> &NonSendData {
        unsafe {
            let index = self.indices.get_unchecked(id);
            self.non_sends.get_unchecked(index as usize)
        }
    }

    #[inline]
    pub unsafe fn get_mut(&mut self, id: ComponentId) -> &mut NonSendData {
        unsafe {
            let index = self.indices.get_unchecked(id);
            self.non_sends.get_unchecked_mut(index as usize)
        }
    }

    pub fn try_get(&self, id: ComponentId) -> Option<&NonSendData> {
        self.indices.map(id, |index| unsafe {
            self.non_sends.get_unchecked(index as usize)
        })
    }

    pub fn try_get_mut(&mut self, id: ComponentId) -> Option<&mut NonSendData> {
        self.indices.map(id, |index| unsafe {
            self.non_sends.get_unchecked_mut(index as usize)
        })
    }

    #[inline]
    pub fn check_ticks(&mut self, check: CheckTicks) {
        let now = check.tick();
        let fall_back = now.relative_to(Tick::MAX_AGE);
        self.non_sends.iter_mut().for_each(move |data| {
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

impl NonSends {
    pub(crate) fn prepare(&mut self, info: &ComponentInfo) {
        let id = info.id();
        if !self.indices.contains(id) {
            unsafe {
                let data = NonSendData::new(info.debug_name(), info.layout(), info.drop_fn());

                let index = self.non_sends.len() as u32;
                self.non_sends.push(data);
                self.ids.push(id);
                self.indices.set(id, index);
            }
        }
    }
}

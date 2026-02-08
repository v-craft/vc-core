#![expect(unsafe_code, reason = "original implementation need unsafe codes.")]

use vc_ptr::OwningPtr;

use super::SparseSet;

use crate::component::ComponentId;
use crate::entity::EntityId;
use crate::storage::SparseComponent;
use crate::tick::{CheckTicks, Tick};

pub struct SparseSets {
    sets: SparseSet<SparseComponent>,
}

impl SparseSets {
    #[inline]
    pub const fn empty() -> Self {
        Self {
            sets: SparseSet::empty(),
        }
    }

    #[inline]
    pub fn component_count(&self) -> usize {
        self.sets.len()
    }

    #[inline]
    pub fn get_raw_index(&self, id: ComponentId) -> Option<u32> {
        self.sets.get_raw_index(id)
    }

    #[inline(always)]
    pub unsafe fn get_unchecked(&self, raw_index: u32) -> &SparseComponent {
        unsafe { self.sets.get_raw(raw_index) }
    }

    #[inline(always)]
    pub unsafe fn get_unchecked_mut(&mut self, raw_index: u32) -> &mut SparseComponent {
        unsafe { self.sets.get_mut_raw(raw_index) }
    }

    #[inline(always)]
    pub unsafe fn init_component(
        &mut self,
        raw_index: u32,
        id: EntityId,
        data: OwningPtr<'_>,
        tick: Tick,
        caller: DebugLocation,
    ) {
        unsafe {
            self.sets
                .get_mut_raw(raw_index)
                .insert(id, data, tick, caller);
        }
    }

    #[inline]
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (ComponentId, &SparseComponent)> {
        self.sets.iter().map(|(&id, data)| (id, data))
    }

    #[inline]
    pub fn iter_mut(
        &mut self,
    ) -> impl ExactSizeIterator<Item = (ComponentId, &mut SparseComponent)> {
        self.sets.iter_mut().map(|(&id, data)| (id, data))
    }

    #[inline]
    pub fn clear_entities(&mut self) {
        for set in self.sets.values_mut() {
            set.dealloc();
        }
    }

    #[inline]
    pub fn check_ticks(&mut self, check: CheckTicks) {
        for set in self.sets.values_mut() {
            set.check_ticks(check);
        }
    }
}

// -----------------------------------------------------------------------------
// Create SparseComponent From ComponentInfo

use crate::component::ComponentInfo;
use crate::utils::DebugLocation;

impl SparseSets {
    #[inline]
    pub fn prepare_component(&mut self, info: &ComponentInfo) {
        if self.sets.get_raw_index(info.id()).is_none() {
            self.sets.insert(info.id(), unsafe {
                SparseComponent::empty(info.layout(), info.drop_fn())
            });
        }
    }

    pub fn get_raw_index_or_insert(&mut self, info: &ComponentInfo) -> u32 {
        if let Some(raw_index) = self.sets.get_raw_index(info.id()) {
            return raw_index;
        };

        self.sets.insert(info.id(), unsafe {
            SparseComponent::with_capacity(info.layout(), info.drop_fn(), 16)
        })
    }
}

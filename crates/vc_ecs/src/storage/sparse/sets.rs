#![allow(clippy::new_without_default, reason = "internal type")]

use alloc::vec::Vec;
use core::fmt::Debug;

use vc_ptr::OwningPtr;
use vc_task::ComputeTaskPool;

use super::SparseComponent;

use crate::component::ComponentId;
use crate::entity::EntityId;
use crate::storage::{ComponentIndices, StorageIndex};
use crate::tick::{CheckTicks, Tick};

// -----------------------------------------------------------------------------
// SparseSets

pub struct SparseSets {
    sets: Vec<SparseComponent>,
    ids: Vec<ComponentId>,
    indices: ComponentIndices,
}

impl Debug for SparseSets {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_map()
            .entries(self.ids.iter().zip(self.sets.iter()))
            .finish()
    }
}

// -----------------------------------------------------------------------------
// Basic methods

impl SparseSets {
    #[inline]
    pub(crate) const fn new() -> Self {
        Self {
            sets: Vec::new(),
            ids: Vec::new(),
            indices: ComponentIndices::new(),
        }
    }

    #[inline(always)]
    pub unsafe fn get(&self, index: StorageIndex) -> &SparseComponent {
        debug_assert!(index.index() < self.sets.len());
        unsafe { self.sets.get_unchecked(index.index()) }
    }

    #[inline(always)]
    pub unsafe fn get_mut(&mut self, index: StorageIndex) -> &mut SparseComponent {
        debug_assert!(index.index() < self.sets.len());
        unsafe { self.sets.get_unchecked_mut(index.index()) }
    }

    #[inline]
    pub unsafe fn init_component(
        &mut self,
        index: StorageIndex,
        id: EntityId,
        data: OwningPtr<'_>,
        tick: Tick,
    ) {
        unsafe {
            self.get_mut(index).init_component(id, data, tick);
        }
    }

    #[inline]
    pub unsafe fn remove_and_drop(&mut self, index: StorageIndex, id: EntityId) {
        unsafe {
            self.get_mut(index).remove_and_drop(id);
        }
    }

    pub fn check_ticks(&mut self, check: CheckTicks) {
        if let Some(task_pool) = ComputeTaskPool::try_get() {
            task_pool.scope(|scope| {
                for table in &mut self.sets {
                    scope.spawn(async move {
                        table.check_ticks(check);
                    });
                }
            });
        } else {
            for table in &mut self.sets {
                table.check_ticks(check);
            }
        }
    }
}

// -----------------------------------------------------------------------------
// register

use crate::component::Components;

impl SparseSets {
    pub(crate) fn register(
        &mut self,
        components: &Components,
        idents: &[ComponentId],
        indices: &mut [StorageIndex],
    ) {
        debug_assert_eq!(idents.len(), indices.len());

        idents
            .iter()
            .zip(indices.iter_mut())
            .for_each(|(&id, index)| {
                if let Some(idx) = self.indices.get(id) {
                    *index = StorageIndex::new(idx);
                } else {
                    unsafe {
                        let idx = self.sets.len() as u32;

                        let info = components.get(id);
                        let value = SparseComponent::new(info.layout(), info.drop_fn());
                        self.sets.push(value);
                        self.ids.push(id);
                        self.indices.set(id, idx);

                        *index = StorageIndex::new(idx);
                    }
                }
            });
    }
}

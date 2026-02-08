#![expect(unsafe_code, reason = "original implementation need unsafe codes.")]

use super::{NoSendResourceData, ResourceData};
use crate::component::ComponentId;
use crate::storage::SparseSet;
use crate::tick::CheckTicks;

// -----------------------------------------------------------------------------
// Resources

pub struct Resources {
    resources: SparseSet<ResourceData>,
}

impl Resources {
    #[inline]
    pub fn len(&self) -> usize {
        self.resources.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (ComponentId, &ResourceData)> {
        self.resources.iter().map(|(id, data)| (*id, data))
    }

    #[inline]
    pub fn get(&self, component_id: ComponentId) -> Option<&ResourceData> {
        self.resources.get(component_id)
    }

    #[inline]
    pub fn get_mut(&mut self, component_id: ComponentId) -> Option<&mut ResourceData> {
        self.resources.get_mut(component_id)
    }

    #[inline]
    pub fn clear(&mut self) {
        self.resources.clear();
    }

    #[inline]
    pub fn check_ticks(&mut self, check: CheckTicks) {
        for info in self.resources.values_mut() {
            info.check_ticks(check);
        }
    }
}

// -----------------------------------------------------------------------------
// NoSendResources

pub struct NoSendResources {
    resources: SparseSet<NoSendResourceData>,
}

impl NoSendResources {
    #[inline]
    pub fn len(&self) -> usize {
        self.resources.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (ComponentId, &NoSendResourceData)> {
        self.resources.iter().map(|(id, data)| (*id, data))
    }

    #[inline]
    pub fn get(&self, component_id: ComponentId) -> Option<&NoSendResourceData> {
        self.resources.get(component_id)
    }

    #[inline]
    pub fn get_mut(&mut self, component_id: ComponentId) -> Option<&mut NoSendResourceData> {
        self.resources.get_mut(component_id)
    }

    #[inline]
    pub fn clear(&mut self) {
        self.resources.clear();
    }

    #[inline]
    pub fn check_ticks(&mut self, check: CheckTicks) {
        for info in self.resources.values_mut() {
            info.check_ticks(check);
        }
    }
}

// -----------------------------------------------------------------------------
// Create ResourceData From Components

use crate::component::Components;
use crate::utils::DebugCheckedUnwrap;

impl Resources {
    pub fn get_data_or_insert(
        &mut self,
        id: ComponentId,
        components: &Components,
    ) -> &mut ResourceData {
        self.resources.get_or_insert_with(id, || {
            let info = unsafe {
                components
                    .infos
                    .get_unchecked(id.index())
                    .as_ref()
                    .debug_checked_unwrap()
            };

            assert!(
                info.is_send_and_sync(),
                "Send + Sync resource {} initialized as non_send.",
                info.debug_name(),
            );

            ResourceData::new(info.debug_name().clone(), info.layout(), info.drop_fn())
        })
    }
}

impl NoSendResources {
    pub fn get_data_or_insert(
        &mut self,
        id: ComponentId,
        components: &Components,
    ) -> &mut NoSendResourceData {
        self.resources.get_or_insert_with(id, || {
            let info = unsafe {
                components
                    .infos
                    .get_unchecked(id.index())
                    .as_ref()
                    .debug_checked_unwrap()
            };

            NoSendResourceData::new(info.debug_name().clone(), info.layout(), info.drop_fn())
        })
    }
}

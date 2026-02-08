use core::any::TypeId;
use core::fmt::Debug;

use alloc::vec::Vec;

use vc_os::sync::Arc;
use vc_utils::extra::TypeIdMap;

use crate::bundle::BundleId;
use crate::component::ComponentId;

// -----------------------------------------------------------------------------
// BundleInfo

#[derive(Debug)]
pub struct BundleInfo {
    pub id: BundleId,
    pub in_table: u32,
    pub components: Arc<[ComponentId]>,
}

// -----------------------------------------------------------------------------
// Bundles

pub struct Bundles {
    pub infos: Vec<BundleInfo>,
    pub indices: TypeIdMap<BundleId>,
}

impl Debug for Bundles {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.infos.iter()).finish()
    }
}

impl Bundles {
    pub(crate) fn new() -> Self {
        let mut val = const {
            Bundles {
                infos: Vec::new(),
                indices: TypeIdMap::new(),
            }
        };

        val.indices.insert(TypeId::of::<()>(), BundleId::EMPTY);
        val.infos.push(BundleInfo {
            id: BundleId::EMPTY,
            in_table: 0,
            components: Arc::new([]),
        });

        val
    }

    /// Returns a reference to an `BundleInfo` depending on the `BundleId`.
    #[inline]
    pub fn get(&self, id: BundleId) -> Option<&BundleInfo> {
        self.infos.get(id.index())
    }

    /// Returns a reference to an `BundleInfo` without doing bounds checking.
    ///
    /// # Safety
    /// Calling this method with an out-of-bounds index is *undefined behavior*.
    #[inline(always)]
    pub unsafe fn get_unchecked(&self, id: BundleId) -> &BundleInfo {
        unsafe { self.infos.get_unchecked(id.index()) }
    }

    #[inline]
    pub fn get_id(&self, id: TypeId) -> Option<BundleId> {
        self.indices.get(&id).copied()
    }
}

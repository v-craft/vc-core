use core::any::TypeId;
use core::fmt::Debug;

use alloc::vec::Vec;

use vc_os::sync::Arc;
use vc_utils::extra::TypeIdMap;

use crate::bundle::BundleId;
use crate::component::ComponentId;

// -----------------------------------------------------------------------------
// BundleInfo

pub struct BundleInfo {
    pub id: BundleId,
    pub in_table: u32,
    pub components: Arc<[ComponentId]>,
}

impl BundleInfo {
    pub fn id(&self) -> BundleId {
        self.id
    }

    pub fn components(&self) -> &[ComponentId] {
        &self.components
    }
}

impl Debug for BundleInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use core::mem::transmute;
        let components = unsafe { transmute::<&[ComponentId], &[u32]>(&self.components) };
        f.debug_struct("BundleInfo")
            .field("id", &self.id.index_u32())
            .field("components", &components)
            .finish()
    }
}

// -----------------------------------------------------------------------------
// Bundles

pub struct Bundles {
    pub(crate) infos: Vec<BundleInfo>,
    pub(crate) indices: TypeIdMap<BundleId>,
}

impl Debug for Bundles {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.infos, f)
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

    pub(crate) unsafe fn register(
        &mut self,
        type_id: TypeId,
        components: Arc<[ComponentId]>,
        in_table: u32,
    ) -> BundleId {
        let index = self.infos.len();
        assert!(index < u32::MAX as usize, "too many bundles");

        let id = BundleId::new(index as u32);
        let info = BundleInfo {
            id,
            components,
            in_table,
        };

        self.infos.push(info);
        self.indices.insert(type_id, id);

        id
    }

    /// # Safety
    /// The target must already exist.
    #[inline]
    pub unsafe fn get(&self, id: BundleId) -> &BundleInfo {
        unsafe { self.infos.get_unchecked(id.index()) }
    }

    /// # Safety
    /// The target must already exist.
    #[inline]
    pub unsafe fn get_mut(&mut self, id: BundleId) -> &mut BundleInfo {
        unsafe { self.infos.get_unchecked_mut(id.index()) }
    }

    #[inline]
    pub fn get_id(&self, id: TypeId) -> Option<BundleId> {
        self.indices.get(&id).copied()
    }
}

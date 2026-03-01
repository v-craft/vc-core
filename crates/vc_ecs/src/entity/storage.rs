use core::fmt::Debug;

use vc_utils::num::NonMaxU32;

use crate::archetype::ArcheId;
use crate::storage::TableId;

#[derive(Clone, Copy)]
pub union StorageId {
    pub table_id: TableId,
    pub arche_id: ArcheId,
}

impl StorageId {
    const _STATIC_ASSERT_: () = const {
        let table = StorageId {
            table_id: TableId::new(12345),
        };
        let arche = StorageId {
            arche_id: ArcheId::new(12345),
        };
        assert!(table.get() == 12345);
        assert!(arche.get() == 12345);
    };

    #[inline(always)]
    const fn get(self) -> u32 {
        unsafe { core::mem::transmute::<StorageId, NonMaxU32>(self).get() }
    }

    #[inline(always)]
    const fn bits(self) -> u32 {
        unsafe { core::mem::transmute::<StorageId, u32>(self) }
    }
}

impl PartialEq for StorageId {
    fn eq(&self, other: &Self) -> bool {
        self.bits() == other.bits()
    }
}

impl Eq for StorageId {}

impl PartialOrd for StorageId {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl Ord for StorageId {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        Ord::cmp(&self.get(), &other.get())
    }
}

impl Debug for StorageId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.get(), f)
    }
}

use core::ptr::NonNull;

use vc_ptr::ConstNonNull;

use crate::archetype::Archetype;
use crate::bundle::BundleInfo;
use crate::storage::Table;
use crate::tick::Tick;
use crate::world::UnsafeWorldCell;

pub(crate) struct BundleSpawner<'w> {
    world: UnsafeWorldCell<'w>,
    bundle_info: ConstNonNull<BundleInfo>,
    table: NonNull<Table>,
    archetype: NonNull<Archetype>,
    change_tick: Tick,
}

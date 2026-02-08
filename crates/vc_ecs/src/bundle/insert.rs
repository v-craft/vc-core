use core::ptr::NonNull;

use vc_ptr::ConstNonNull;

use crate::archetype::{Archetype, ArchetypeInsertedBundle, ArchetypeMoveType};
use crate::bundle::BundleInfo;
use crate::storage::Table;
use crate::tick::Tick;
use crate::world::UnsafeWorldCell;

pub(crate) struct BundleInserter<'w> {
    world: UnsafeWorldCell<'w>,
    bundle_info: ConstNonNull<BundleInfo>,
    archetype_after_insert: ConstNonNull<ArchetypeInsertedBundle>,
    table: NonNull<Table>,
    archetype: NonNull<Archetype>,
    archetype_move_type: ArchetypeMoveType,
    change_tick: Tick,
}

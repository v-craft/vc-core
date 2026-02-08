use core::ptr::NonNull;

use vc_ptr::ConstNonNull;

use crate::archetype::Archetype;
use crate::bundle::BundleInfo;
use crate::relationship::RelationshipHookMode;
use crate::storage::Table;
use crate::world::UnsafeWorldCell;

pub(crate) struct BundleRemover<'w> {
    world: UnsafeWorldCell<'w>,
    bundle_info: ConstNonNull<BundleInfo>,
    old_and_new_table: Option<(NonNull<Table>, NonNull<Table>)>,
    old_archetype: NonNull<Archetype>,
    new_archetype: NonNull<Archetype>,
    relationship_hook_mode: RelationshipHookMode,
}

#![expect(unsafe_code)]

// -----------------------------------------------------------------------------
// Modes

mod bundle;
mod id;
mod info;
mod insert;
mod remove;
mod spawn;
mod status;

// -----------------------------------------------------------------------------
// Internal

pub(crate) use insert::BundleInserter;
pub(crate) use remove::BundleRemover;
pub(crate) use spawn::BundleSpawner;

// -----------------------------------------------------------------------------
// Exports

pub use bundle::{Bundle, DynamicBundle};
pub use id::BundleId;
pub use info::BundleInfo;
pub use status::{BundleComponentStatus, ComponentStatus, SpawnBundleStatus};

// -----------------------------------------------------------------------------
// Bundle

use alloc::boxed::Box;
use alloc::vec::Vec;

use vc_utils::extra::TypeIdMap;
use vc_utils::hash::HashMap;

use crate::component::ComponentId;
use crate::storage::StorageType;

pub struct Bundles {
    bundle_infos: Vec<BundleInfo>,
    bundle_ids: TypeIdMap<BundleId>,
    /// Cache bundles, which contains both explicit and required components of [`Bundle`]
    contributed_bundle_ids: TypeIdMap<BundleId>,
    dynamic_bundle_ids: HashMap<Box<[ComponentId]>, BundleId>,
    dynamic_bundle_storages: HashMap<BundleId, Vec<StorageType>>,
    /// Cache optimized dynamic [`BundleId`] with single component
    dynamic_component_bundle_ids: HashMap<ComponentId, BundleId>,
    dynamic_component_storages: HashMap<BundleId, StorageType>,
}

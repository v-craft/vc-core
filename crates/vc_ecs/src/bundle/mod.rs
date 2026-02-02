#![expect(unsafe_code)]

// -----------------------------------------------------------------------------
// Modes

mod bundle;
mod id;
mod info;
mod status;

// -----------------------------------------------------------------------------
// Exports

pub use bundle::{Bundle, DynamicBundle};
pub use id::BundleId;
pub use info::{BundleInfo, InsertMode};
pub use status::{BundleComponentStatus, ComponentStatus, SpawnBundleStatus};

// -----------------------------------------------------------------------------
// Bundle

//! Bundle - Combination of component data.

// -----------------------------------------------------------------------------
// Modules

mod ident;
mod impls;
mod info;

// -----------------------------------------------------------------------------
// Exports

pub use vc_ecs_derive::Bundle;

pub use ident::BundleId;
pub use impls::Bundle;
pub use info::{BundleInfo, Bundles};

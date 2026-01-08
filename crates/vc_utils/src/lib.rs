#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![no_std]

// -----------------------------------------------------------------------------
// Compilation config

/// Some macros used for compilation control.
pub mod cfg {
    vc_cfg::define_alias! {
        #[cfg(feature = "rayon")] => rayon,
    }
}

// -----------------------------------------------------------------------------
// No STD Support

extern crate alloc;

// -----------------------------------------------------------------------------
// Modules

mod default;
mod range_invoke;
mod typeid_map;

pub mod collections;
pub mod hash;
pub mod vec;

// -----------------------------------------------------------------------------
// Top-level exports

pub use default::default;
pub use typeid_map::TypeIdMap;

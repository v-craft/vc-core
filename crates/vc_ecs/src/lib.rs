#![cfg_attr(docsrs, expect(internal_features, reason = "needed for fake_variadic"))]
#![cfg_attr(docsrs, feature(doc_cfg, rustdoc_internals))]
#![expect(unsafe_code, reason = "ECS requires underlying operation")]
#![no_std]

// -----------------------------------------------------------------------------
// Compilation config

/// Some macros used for compilation control.
pub mod cfg {
    vc_cfg::define_alias! {
        #[cfg(feature = "std")] => std,
        #[cfg(any(feature = "debug", debug_assertions))] => debug,
    }
}

// -----------------------------------------------------------------------------
// no_std support

crate::cfg::std! { extern crate std; }

extern crate alloc;

// -----------------------------------------------------------------------------
// Modules

pub mod borrow;
pub mod bundle;
pub mod tick;
pub mod utils;

pub mod component;
pub mod resource;
pub mod storage;

pub mod archetype;
pub mod entity;

pub mod query;
pub mod system;

pub mod world;

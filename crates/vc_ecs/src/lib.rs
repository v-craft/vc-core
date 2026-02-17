#![cfg_attr(docsrs, expect(internal_features, reason = "needed for fake_variadic"))]
#![cfg_attr(docsrs, feature(doc_cfg, rustdoc_internals))]
#![expect(unsafe_code, reason = "ECS requires underlying operation")]
#![no_std]
#![allow(clippy::missing_safety_doc, reason = "todo")]

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

pub mod utils;

pub mod archetype;
pub mod borrow;
pub mod bundle;
pub mod change_detection;
pub mod clone;
pub mod component;
pub mod entity;
pub mod storage;
pub mod tick;
pub mod world;

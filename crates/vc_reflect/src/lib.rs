#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, expect(internal_features, reason = "needed for fake_variadic"))]
#![cfg_attr(docsrs, feature(doc_cfg, rustdoc_internals))]
#![no_std]

// -----------------------------------------------------------------------------
// Compilation config

/// Some macros used for compilation control.
pub mod cfg {
    vc_cfg::define_alias! {
        #[cfg(feature = "std")] => std,
        #[cfg(feature = "auto_register")] => auto_register,
        #[cfg(feature = "reflect_docs")] => reflect_docs,
        #[cfg(all(debug_assertions, feature = "debug"))] => debug,
    }
}

// -----------------------------------------------------------------------------
// Extern Self

// Usually, we need to use `crate` in the crate itself and use `vc_reflect` in doc testing.
// But `macro_utils::Manifest` can only choose one, so we must have an
// `extern self` to ensure `vc_reflect` can be used as an alias for `crate`.
extern crate self as vc_reflect;

// -----------------------------------------------------------------------------
// no_std support

crate::cfg::std! {
    extern crate std;
}

extern crate alloc;

// -----------------------------------------------------------------------------
// Modules

mod reflection;

pub mod access;
pub mod impls;
pub mod info;
pub mod ops;
pub mod registry;
pub mod serde;

// -----------------------------------------------------------------------------
// Top-Level exports

pub mod __macro_exports;

pub use reflection::{FromReflect, Reflect, reflect_hasher};
pub use vc_reflect_derive as derive;

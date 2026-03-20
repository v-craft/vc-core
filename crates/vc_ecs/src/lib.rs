//! TODO: Supplementary documentation.
//!
//! - [`ComponentStorage`] : How components are stored and accessed.
//! - [`Archetype`] : How component queries are performed efficiently.
//! - [`Schedule`] : How systems are executed with maximum concurrency.
//! - ......
//!
//! [`ComponentStorage`]: crate::component::ComponentStorage
//! [`Archetype`]: crate::archetype::Archetype
//! [`Schedule`]: crate::schedule::Schedule

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
// Extern Self

// Usually, we need to use `crate` in the crate itself and use `vc_ecs` in doc testing.
// But `macro_utils::Manifest` can only choose one, so we must have an
// `extern self` to ensure `vc_ecs` can be used as an alias for `crate`.
extern crate self as vc_ecs;

// -----------------------------------------------------------------------------
// no_std support

crate::cfg::std! { extern crate std; }

extern crate alloc;

// -----------------------------------------------------------------------------
// Modules

pub use vc_ecs_derive as derive;

pub mod borrow;
pub mod bundle;
pub mod error;
pub mod tick;
pub mod utils;

pub mod component;
pub mod resource;
pub mod storage;

pub mod archetype;
pub mod entity;

pub mod label;
pub mod query;
pub mod schedule;
pub mod system;

pub mod world;

pub mod __macro_exports;

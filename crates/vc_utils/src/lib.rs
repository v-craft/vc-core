#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![no_std]

// -----------------------------------------------------------------------------
// No STD Support

extern crate alloc;

// -----------------------------------------------------------------------------
// Modules

mod default;
mod range_invoke;
mod unsafe_deref;

pub mod extra;
pub mod hash;
pub mod index;
pub mod num;

pub mod vec;

// -----------------------------------------------------------------------------
// Top-level exports

pub use default::default;
pub use unsafe_deref::UnsafeCellDeref;

// An alternative to `core::hint::cold_path`,
// used for optimizing branch prediction.
#[cold]
#[inline(always)]
pub const fn cold_path() {}

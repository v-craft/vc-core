//! Provide some tools for proc-macro crates.
#![allow(clippy::std_instead_of_core, reason = "proc-macro lib")]
#![allow(clippy::std_instead_of_alloc, reason = "proc-macro lib")]

extern crate proc_macro;

mod manifest;
pub use manifest::Manifest;

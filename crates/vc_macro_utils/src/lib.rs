//! Provide some tools for proc-macro crates.
#![allow(clippy::std_instead_of_alloc, reason = "proc-macro crate")]

extern crate proc_macro;

// -----------------------------------------------------------------------------
// Modules

mod manifest;

pub mod full_path;

// -----------------------------------------------------------------------------
// Modules

pub use manifest::Manifest;

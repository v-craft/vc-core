//! Temporal quantification
//!
//! This module provides a cross-platform alternative to the standard library's `time` module.
//! - In `web` environments, it re-exports `web_time` crate's implementation.
//! - In `std` environments, it directly re-exports the standard library's contents.
//! - In `no_std` environments, different fallback implementations are used based on the situation.<br>
//!   (See [fallback](__fallback) module for no_std support)
//!
//! We strive to ensure that fallback implementations maintain the same API as the standard library
//! (only stable APIs). But some newer APIs may not be immediately available;
//! please submit an Issue in the [repository](https://github.com/VoidCraft-Engine/vc-core) for such cases.
//!
//! See the [standard library](https://doc.rust-lang.org/std/time) for further details.

pub use core::time::{Duration, TryFromFloatSecsError};
pub use time_impl::{Instant, SystemTime, SystemTimeError};

crate::cfg::switch! {
    crate::cfg::web => {
        use ::web_time as time_impl;
    }
    crate::cfg::std => {
        use ::std::time as time_impl;
    }
    _ => {
        mod __fallback;
        use __fallback as time_impl;
    }
}

// -----------------------------------------------------------------------------
// Tests and Docs

#[cfg(any(test, docsrs, feature = "docsrs_dev"))]
crate::cfg::switch! {
    crate::cfg::web => {
        pub mod __fallback;
    }
    crate::cfg::std => {
        pub mod __fallback;
    }
    _ => {}
}

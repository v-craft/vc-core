//! Provide `sleep` function for all platforms.
//!
//! - In `std` environments, it directly re-exports `std::thread::sleep`.
//! - In non-`std` environments, implementations based on spin locks are used.

pub use thread_impl::sleep;

crate::cfg::switch! {
    crate::cfg::std => {
        use std::thread as thread_impl;
    }
    _ => {
        mod __fallback;
        use __fallback as thread_impl;
    }
}

/// Returns an estimate of the default amount of parallelism a program should use.
///
/// It's similar to [`std::thread::available_parallelism`],
/// but when this function fails or in the no_std environment, it directly returns `1`.
///
/// We ensure that `result > 0` .
pub fn available_parallelism() -> usize {
    crate::cfg::switch! {
        crate::cfg::std => {
            std::thread::available_parallelism()
                .map(core::num::NonZero::<usize>::get)
                .unwrap_or(1)
        }
        _ => {
            1
        }
    }
}

// -----------------------------------------------------------------------------
// Tests and Docs

#[cfg(any(test, docsrs, feature = "docsrs_dev"))]
crate::cfg::switch! {
    crate::cfg::std => {
        pub mod __fallback;
    }
    _ => {}
}

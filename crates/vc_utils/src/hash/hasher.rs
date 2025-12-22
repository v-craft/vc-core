//! Provide `FixedHasher` and `NoOpHasher`.
//!
//! `FixedHasher` based on `foldhash` crate,
//! Provide stable hash results through a fixed hash seed.
//!
//! `NoOpHasher` directly use u64 or bit data as hash values.

use core::fmt::Debug;
use core::hash::{BuildHasher, Hasher};

use foldhash::fast::{FixedState, FoldHasher};

// -----------------------------------------------------------------------------
// FixedHasher

/// A fixed hash seed.
const FIXED_HASH_STATE: FixedState = FixedState::with_seed(0x95EE04C4F326B271);

/// A fixed hasher provided hash results that only related on the input.
///
/// A type alias for [`foldhash::fast::FoldHasher`] .
///
/// Which can be created through [`FixedHashState::build_hasher`].
pub type FixedHasher = FoldHasher<'static>;

/// Fixed Hash State based upon a random but fixed seed.
///
/// Based on `foldhash`, but changed the fixed seed.
///
/// # Examples
///
/// ```
/// use core::hash::{Hash, Hasher, BuildHasher};
/// use vc_utils::hash::FixedHashState;
///
/// let mut hasher = FixedHashState.build_hasher();
/// 3.hash(&mut hasher);
/// let result = hasher.finish();
///
/// println!("Hash Result {result}"); // Fixed Result
/// ```
#[derive(Copy, Clone, Default, Debug)]
pub struct FixedHashState;

impl BuildHasher for FixedHashState {
    type Hasher = FixedHasher;

    #[inline(always)]
    fn build_hasher(&self) -> Self::Hasher {
        FIXED_HASH_STATE.build_hasher()
    }
}

// -----------------------------------------------------------------------------
// NoOpHasher

/// A no-op hash that directly pass value through `u64`.
///
/// Which can be created through [`NoOpHashState::build_hasher`].
#[derive(Copy, Clone, Default, Debug)]
pub struct NoOpHasher {
    hash: u64,
}

impl Hasher for NoOpHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.hash
    }

    fn write(&mut self, bytes: &[u8]) {
        // Usually recommended to use `write_u64` directly
        for byte in bytes.iter().rev() {
            // rotate left ensure that `write_u32(10)` is eq to `write_u64(10)`.
            self.hash = self.hash.rotate_left(8).wrapping_add(*byte as u64);
        }
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.hash = i;
    }
}

/// A fixed hasher without any additional operations.
///
/// Only storing one `u64` and assigning values directly by `writa_u64`.
///
/// Other method will call `write`, which will add the input bytes in reverse
/// order to `u64`, and make it rotate left. Ensure that the results of
/// `write_u64(1234)` and `write_i32(1234)` are the same **if only called once**.
///
/// # Examples
///
/// ```
/// use core::hash::{Hash, Hasher, BuildHasher};
/// use vc_utils::hash::NoOpHashState;
///
/// let mut hasher = NoOpHashState.build_hasher();
/// 3.hash(&mut hasher);
/// let result = hasher.finish();
///
/// assert_eq!(result, 3_u64);
/// ```
#[derive(Copy, Clone, Default, Debug)]
pub struct NoOpHashState;

impl BuildHasher for NoOpHashState {
    type Hasher = NoOpHasher;

    #[inline(always)]
    fn build_hasher(&self) -> Self::Hasher {
        NoOpHasher { hash: 0 }
    }
}

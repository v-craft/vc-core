#![expect(unsafe_code, reason = "original implementation")]

pub mod array;
pub mod fast;
pub mod small;

mod utils;

pub use array::ArrayVec;
pub use fast::FastVec;
pub use small::SmallVec;

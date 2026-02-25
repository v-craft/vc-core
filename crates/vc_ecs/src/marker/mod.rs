//! This module provides a mechanism for creating type-level boolean flags
//! using marker traits and associated constants.
//!
//! The pattern allows for compile-time trait selection based on boolean
//! properties, which is useful for generic programming and type system
//! metaprogramming.

mod seal {
    pub trait Marker {}
}

use seal::Marker;

/// A trait that indicates whether a type represents a mutable concept.
pub trait IsMutable: Marker + 'static {
    const VALUE: bool;
}

/// A trait that indicates whether a type represents a sendable concept.
pub trait IsSend: Marker + 'static {
    const VALUE: bool;
}

/// A type representing the affirmative or "true" boolean value at the type level.
pub struct Yes;

/// A type representing the negative or "false" boolean value at the type level.
pub struct No;

impl Marker for Yes {}
impl Marker for No {}

impl IsMutable for Yes {
    const VALUE: bool = true;
}

impl IsMutable for No {
    const VALUE: bool = false;
}

impl IsSend for Yes {
    const VALUE: bool = true;
}

impl IsSend for No {
    const VALUE: bool = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yes_and_no() {
        assert_eq!(<Yes as IsMutable>::VALUE, true);
        assert_eq!(<Yes as IsSend>::VALUE, true);
        assert_eq!(<No as IsMutable>::VALUE, false);
        assert_eq!(<No as IsSend>::VALUE, false);
    }
}

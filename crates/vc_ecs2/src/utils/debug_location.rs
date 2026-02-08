#![allow(
    unused_variables,
    reason = "The function params may not be used in release mode."
)]

use core::cell::UnsafeCell;
use core::fmt;
use core::hash::Hash;
use core::ops::{Deref, DerefMut};
use core::panic::Location;

#[cfg(not(any(debug_assertions, feature = "debug")))]
use core::marker::PhantomData;

use crate::cfg;

// -----------------------------------------------------------------------------
// DebugLocation

/// A value that contains a `T` if the `debug` feature is enabled,
/// and is a ZST if it is not.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct DebugLocation<T: ?Sized = &'static Location<'static>>(
    #[cfg(any(debug_assertions, feature = "debug"))] T,
    #[cfg(not(any(debug_assertions, feature = "debug")))] PhantomData<T>,
);

// -----------------------------------------------------------------------------
// Traits

impl<T: fmt::Display> fmt::Display for DebugLocation<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        cfg::debug! { self.0.fmt(f)?; }
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// Implementation

impl DebugLocation {
    /// Returns the source location of the caller of this function.
    ///
    /// If that function's caller is annotated then its call location will be returned,
    /// and so on up the stack to the first call within a non-tracked function body.
    #[inline(always)]
    #[cfg_attr(any(debug_assertions, feature = "debug"), track_caller)]
    pub const fn caller() -> Self {
        cfg::debug! {
            if {  Self(Location::caller()) }
            else { Self(PhantomData) }
        }
    }
}

impl<T> DebugLocation<T> {
    /// Constructs a new `DebugLocation` that wraps the given value.
    #[inline(always)]
    pub const fn new(value: T) -> Self
    where
        T: Copy,
    {
        cfg::debug! {
            if { DebugLocation(value) }
            else { DebugLocation(PhantomData) }
        }
    }

    /// Constructs a new `DebugLocation` that wraps the result of the given closure.
    #[inline(always)]
    pub fn new_with(f: impl FnOnce() -> T) -> Self {
        cfg::debug! {
            if { Self(f()) } else { Self(PhantomData) }
        }
    }

    /// Maps an `DebugLocation<T> `to `DebugLocation<U>` by applying a function to a contained value.
    #[inline]
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> DebugLocation<U> {
        cfg::debug! {
            if { DebugLocation(f(self.0)) }
            else { DebugLocation(PhantomData) }
        }
    }

    /// Converts a pair `DebugLocation` to a tuple.
    #[inline]
    pub fn zip<U>(self, other: DebugLocation<U>) -> DebugLocation<(T, U)> {
        cfg::debug! {
            if { DebugLocation((self.0, other.0)) }
            else { DebugLocation(PhantomData) }
        }
    }

    /// Returns the contained value or a default.
    ///
    /// - If the `debug` feature is enabled, this always returns the contained value.
    /// - If it is disabled, this always returns `T::Default()`.
    #[inline(always)]
    pub fn unwrap_or_default(self) -> T
    where
        T: Default,
    {
        self.into_option().unwrap_or_default()
    }

    /// Converts an `MaybeLocation` to an [`Option`] to allow run-time branching.
    ///
    /// - If the `debug` feature is enabled, this always returns `Some`.
    /// - If it is disabled, this always returns `None`.
    #[inline(always)]
    pub fn into_option(self) -> Option<T> {
        cfg::debug! {
            if { Some(self.0) } else { None }
        }
    }
}

impl<T> DebugLocation<Option<T>> {
    /// Transposes an [`Option`] of `MaybeLocation` into a `DebugLocation` of [`Option`].
    #[inline]
    pub fn untranspose(value: Option<DebugLocation<T>>) -> Self {
        cfg::debug! {
            if { Self(value.map(|value| value.0)) }
            else { DebugLocation(PhantomData) }
        }
    }

    /// Transposes a `DebugLocation` of an [`Option`] into an [`Option`] of a `MaybeLocation`.
    #[inline]
    pub fn transpose(self) -> Option<DebugLocation<T>> {
        cfg::debug! {
            if { self.0.map(|v|DebugLocation(v)) }
            else { Some(DebugLocation(PhantomData)) }
        }
    }
}

impl<T> DebugLocation<&T> {
    /// Maps an `DebugLocation<&T>` to an `DebugLocation<T>` by copying the contents.
    #[inline(always)]
    pub const fn copied(&self) -> DebugLocation<T>
    where
        T: Copy,
    {
        cfg::debug! {
            if { DebugLocation(*self.0) }
            else { DebugLocation(PhantomData) }
        }
    }
}

impl<T> DebugLocation<&mut T> {
    /// Maps an `DebugLocation<&mut T>` to an `DebugLocation<T>` by copying the contents.
    #[inline(always)]
    pub const fn copied(&self) -> DebugLocation<T>
    where
        T: Copy,
    {
        cfg::debug! {
            if { DebugLocation(*self.0) }
            else { DebugLocation(PhantomData) }
        }
    }

    /// Assigns the contents of an `DebugLocation<T>` to an `DebugLocation<&mut T>`.
    #[inline(always)]
    pub fn assign(&mut self, value: DebugLocation<T>) {
        cfg::debug! {
            *self.0 = value.0;
        }
    }
}

impl<T: ?Sized> DebugLocation<T> {
    /// Converts from `&DebugLocation<T>` to `DebugLocation<&T>`.
    #[inline(always)]
    pub const fn as_ref(&self) -> DebugLocation<&T> {
        cfg::debug! {
            if { DebugLocation(&self.0) }
            else { DebugLocation(PhantomData) }
        }
    }

    /// Converts from `&mut DebugLocation<T>` to `DebugLocation<&mut T>`.
    #[inline(always)]
    pub const fn as_mut(&mut self) -> DebugLocation<&mut T> {
        cfg::debug! {
            if { DebugLocation(&mut self.0) }
            else { DebugLocation(PhantomData) }
        }
    }

    /// Converts from `&DebugLocation<T>` to `DebugLocation<&T::Target>`.
    #[inline(always)]
    pub fn as_deref(&self) -> DebugLocation<&T::Target>
    where
        T: Deref,
    {
        cfg::debug! {
            if {  DebugLocation(&*self.0) }
            else { DebugLocation(PhantomData) }
        }
    }

    /// Converts from `&mut DebugLocation<T>` to `DebugLocation<&mut T::Target>`.
    #[inline(always)]
    pub fn as_deref_mut(&mut self) -> DebugLocation<&mut T::Target>
    where
        T: DerefMut,
    {
        cfg::debug! {
            if {  DebugLocation(&mut *self.0) }
            else { DebugLocation(PhantomData) }
        }
    }
}

impl<T: ?Sized> DebugLocation<UnsafeCell<T>> {
    /// Converts from `DebugLocation<UnsafeCell<T>>` to `DebugLocation<&T>`.
    #[inline(always)]
    pub fn get_inner(&self) -> DebugLocation<T>
    where
        T: Sized + Copy,
    {
        cfg::debug! {
            if {  DebugLocation(unsafe { *self.0.get() }) }
            else { DebugLocation(PhantomData) }
        }
    }

    /// Converts from `DebugLocation<UnsafeCell<T>>` to `DebugLocation<&T>`.
    #[inline(always)]
    pub fn get_ref(&self) -> DebugLocation<&T> {
        cfg::debug! {
            if {  DebugLocation(unsafe { &*self.0.get() }) }
            else { DebugLocation(PhantomData) }
        }
    }

    /// Converts from `DebugLocation<UnsafeCell<T>>` to `DebugLocation<&mut T>`.
    #[inline(always)]
    pub fn get_mut(&mut self) -> DebugLocation<&mut T> {
        cfg::debug! {
            if {  DebugLocation(self.0.get_mut()) }
            else { DebugLocation(PhantomData) }
        }
    }
}

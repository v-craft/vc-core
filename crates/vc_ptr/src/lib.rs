//! This crate provides lightweight pointer wrappers used by the ECS module.
//!
//! The goal is to avoid moving large values between stack frames by passing
//! references or pointers instead, while adding lifetimes and optional alignment
//! checks to improve safety over raw pointers.
//!
//! **ConstNonNull**
//!
//! [`ConstNonNull<T>`] is similar to [`NonNull<T>`](core::ptr::NonNull): a non-null
//! pointer that cannot be used to obtain mutable references directly.
//!
//! **ThinSlice** and **ThinSliceMut**
//!
//! [`ThinSlice`] and [`ThinSliceMut`] is a thin slice pointer that stores only a
//! data pointer (no length), making it smaller. Access through it is unsafe because
//! bounds checks are not available.
//!
//! **Ptr** and **PtrMut**
//!
//! [`Ptr<'a>`] and [`PtrMut<'a>`] are type-erased `&T` and `&mut T` equivalents.
//! Compared to raw pointers, they add a lifetime and optional alignment checks to
//! better approximate the safety of references.
//!
//! **OwningPtr**
//!
//! [`OwningPtr<'a>`] is an “ownership pointer” that can consume the pointee via
//! [`drop_as`](OwningPtr::drop_as) or read out ownership via [`read`](OwningPtr::read).
//! If the value is neither read nor dropped, it may leak.
//!
//! `OwningPtr` does **not** manage allocation; it typically points to stack values
//! or data managed by other containers.
#![expect(unsafe_code, reason = "Raw pointers are inherently unsafe.")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![no_std]

// -----------------------------------------------------------------------------
// Modules

mod non_null;
mod thin_slice;
mod type_erased;

// -----------------------------------------------------------------------------
// Top-level exports

pub use non_null::ConstNonNull;
pub use thin_slice::{ThinSlice, ThinSliceMut};
pub use type_erased::{OwningPtr, Ptr, PtrMut};

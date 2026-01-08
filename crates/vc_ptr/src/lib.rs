//! This crate implements several pointer wrappers on top of Rust’s standard pointer types
//! that will be used frequently in the ECS module.
//!
//! **`ConstNonNull`**:
//!
//! [`ConstNonNull<T>`] is similar to [`NonNull<T>`](core::ptr::NonNull),
//! a non-null pointer that cannot be used to obtain mutable references directly.
//!
//! **`ThinSlicePtr`**:
//!
//! [`ThinSlicePtr`] is a thin slice pointer that does not store length (only a pointer), making it lighter.
//! Access through it is unsafe because bounds checks are not available;
//! in debug mod it may retain length info to help debugging.
//!
//! **`Ptr` and `PtrMut`**:
//!
//! [`Ptr<'a>`] and [`PtrMut<'a>`] are like type-erased `&T` and `&mut T`,
//! Compared to raw pointers they add a lifetime and optional alignment checks to approach the safety of references.
//!
//! **`OwningPtr`**:
//!
//! [`OwningPtr<'a>`] is an “ownership pointer”, it can be used to consume the pointee
//! by [`drop_as`](OwningPtr::drop_as) or readout ownership by [`read`](OwningPtr::read).
//!
//! If the pointer does not read or drop value, it may cause a memory leak.
//!
//! It does **not** manage memory of the pointee(so typically points to stack
//! values or objects managed by other containers).
#![expect(unsafe_code, reason = "Raw pointers are inherently unsafe.")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![no_std]

// -----------------------------------------------------------------------------
// Modules

mod non_null;
mod thin_slice;
mod type_erased;

/* mod moving; */

// -----------------------------------------------------------------------------
// Top-level exports

pub use non_null::ConstNonNull;
pub use thin_slice::ThinSlicePtr;
pub use type_erased::{OwningPtr, Ptr, PtrMut};

/* pub use moving::MovingPtr; */

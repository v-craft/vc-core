//! Provide type registry for non-object infomation querying.
//!
//! ## Menu
//!
//! - [`TypeTrait`]: A trait representing a capability supported by a type.
//! - [`FromType`]: A trait provide a function to crate a `TypeTrait` from a type.
//! - [`TypeMeta`]: A container including a [`TypeInfo`] and a [`TypeTrait`] table.
//! - [`GetTypeMeta`]: A trait provide a function to crate a `TypeMeta` from a type.
//! - [`TypeRegistry`]: A container for storaging and operating `TypeMeta`s.
//! - TypeTraits:
//!     - [`TypeTraitDefault`]: Provide [`Default`] capability for reflecion type.
//!     - [`TypeTraitFromPtr`]: Convert ptr to reflection reference.
//!     - [`TypeTraitFromReflect`]: Provide [`FromReflect`] support for deserialization.
//!     - [`TypeTraitSerialize`]: Provide serialization support for reflection type.
//!     - [`TypeTraitDeserialize`]: Provide deserialization support for reflection type.
//! - [`reflect_trait`]: a attribute macro, which generate a `Reflect{trait_name}` struct, can be used as [`TypeTrait`].
//!
//! ## auto_register
//!
//! See [`TypeRegistry::auto_register`] .
//!
//! We use [`inventory`] crate to implement static registration,
//! not all platforms support it (although major platforms do).
//!
//! The good news is that if it is not supported,
//! this function will directly return false without causing any errors.
//!
//! ### auto_register type menu
//!
//! - `()` `bool` `char` `f32` `f64`
//! - `i8` `i16` `i32` `i64` `i128` `isize`
//! - `u8` `u16` `u32` `u64` `u128` `usize`
//! - `core::num::NonZero`: I8-I128 U8-U128 Isize Usize
//! - `Atomic`: Bool I8-I64 U8-U64 Isize Usize (without Ptr)
//! - `String` `&'static str` `Cow<'static, str>`
//! - `core::any::TypeId`
//! - `core::time::Duration`
//! - `&'static core::panic::Location<'static>`
//! - `vc_os::time::Instant`
//! - "std" feature:
//!   `OsString` `PathBuf` `Cow<'static, Path>` `&'static Path`
//!
//! [`reflect_trait`]: crate::derive::reflect_trait
//! [`FromReflect`]: crate::FromReflect
//! [`TypeInfo`]: crate::info::TypeInfo

// -----------------------------------------------------------------------------
// Modules

mod from_type;
mod traits;
mod type_meta;
mod type_registry;
mod type_trait;

// -----------------------------------------------------------------------------
// Exports

pub use from_type::FromType;
pub use traits::TypeTraitDefault;
pub use traits::{TypeTraitDeserialize, TypeTraitSerialize};
pub use traits::{TypeTraitFromPtr, TypeTraitFromReflect};
pub use type_meta::{GetTypeMeta, TypeMeta};
pub use type_registry::{TypeRegistry, TypeRegistryArc};
pub use type_trait::TypeTrait;

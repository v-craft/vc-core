//! Provide compile-time type infomation implementations.
//!
//! ## Menu
//!
//! - [`TypeId`]: `core::any::TypeId`, a 16 Bytes value representing a single type, but it's uncertain before running.
//!
//! - [`TypePath`]: A trait for obtaining type names, without prefix `::`.
//!     - [`type_path`](TypePath::type_path): Full name, a fixed and unique identifier for the type.
//!     - [`type_name`](TypePath::type_name): The name without module path, may be duplicated.
//!     - [`type_ident`](TypePath::type_ident): The name without generics and module path.
//!     - [`module_path`](TypePath::module_path): optional module path(e.g. "vc_reflect::info").
//!
//! - [`DynamicTypePath`]: Provide dynamic dispatch for `TypePath`.
//!
//! - [`TypePathTable`]: A struct, storaging 4 function pointer for a single type's `TypePath` implementation.
//!
//! - [`Type`]: A struct contains a `TypeId` and a `TypePathTable`, 48 Bytes.
//!
//! - [`CustomAttributes`]: A attribute container, just like `Map<TypeId, Box<dyn Any>>`.
//!
//! - [`Generics`]: A list of `GenericInfo`, representing instantiated generics infomations.
//!     - [`GenericInfo`]: A enum of `TypeParamInfo` and `ConstParamInfo`, 72 Bytes.
//!     - [`TypeParamInfo`]: Type generic infomation, including param name, `Type` and optional default `Type`.
//!     - [`ConstParamInfo`]: Const generic infomation, including param name, `Type` and const param value.
//!
//! - [`TypeInfo`]: A enum representing compile-time type infomations, the inner is one of following:
//!     - Note: The following types all contain Self's `Type` and generic information.
//!     - [`ArrayInfo`]: For array(e.g. `[i32;5]`) infomation, including array capacity and item type info .
//!     - [`ListInfo`]: For list-like(e.g. `Vec<i32>`) infomation, including item type info.
//!     - [`TupleInfo`]: For tuple(e.g. `(i32, f32)`) infomation, including items(fields) type info.
//!     - [`StructInfo`]: For struct(e.g. `A{..}`)  infomation, including field names, fields type info and custom attrirbutes.
//!     - [`TupleStructInfo`]: For tuple-struct(e.g. `A(..)`) infomation, including fields type info and custom attrirbutes.
//!     - [`EnumInfo`]: For enum infomation, including variants infomation and custom attrirbutes.
//!     - [`MapInfo`]: For map-like(e.g. `HashMap<K, V>`) infomation, including key type info and value type info.
//!     - [`SetInfo`]: For set-like(e.g. `HashSet<T>`) infomation, including value type info.
//!     - [`OpaqueInfo`]: For Internal invisible types(e.g. `struct A;`, `String`), including custom attrirbutes.
//!
//! - [`VariantInfo`]: A enum representing a enum variant infomation, the inner is one of following:
//!     - Note: The following types all contain Self's variant name and custom attributes,
//!     - [`StructVariantInfo`]: Similiar to `StructInfo`, but without generic info.
//!     - [`TupleVariantInfo`]: Similiar to `TupleInfo`,
//!     - [`UnitVariantInfo`]: No more content.
//!
//! - Field Info:
//!     - [`NamedField`]: For struct's field, including field name, field type info and custom attributes.
//!     - [`UnnamedField`]: For tuple(or tuple-struct)'s field, including field index, field type info and custom attributes.
//!
//! - Kind:
//!     - [`ReflectKind`]: representing reflect type kind, for example `Struct`, `Array`, `Opaque` .
//!     - [`VariantKind`]: representing enum variant kind, one of `Struct`, `Tuple` and `Unit`.
//!
//! - [`Typed`]: A trait for obtaining `TypeInfo` data.
//!
//! - [`DynamicTyped`]: Provide dynamic dispatch for `Typed`.
//!
//! [`TypeId`]: core::any::TypeId

// -----------------------------------------------------------------------------
// Modules

mod array_info;
mod attributes;
mod const_param_data;
mod docs_macro;
mod enum_info;
mod field_info;
mod generics;
mod list_info;
mod map_info;
mod opaque_info;
mod set_info;
mod struct_info;
mod tuple_info;
mod tuple_struct_info;
mod type_info;
mod type_path;
mod typed;
mod variant_info;

// -----------------------------------------------------------------------------
// Internal API

use attributes::{impl_custom_attributes_fn, impl_with_custom_attributes};
use docs_macro::impl_docs_fn;
use generics::impl_generic_fn;

pub(crate) use type_path::impl_type_fn;

// -----------------------------------------------------------------------------
// Exports

pub use array_info::ArrayInfo;
pub use attributes::CustomAttributes;
pub use const_param_data::ConstParamData;
pub use enum_info::EnumInfo;
pub use field_info::{NamedField, UnnamedField};
pub use generics::{ConstParamInfo, GenericInfo, Generics, TypeParamInfo};
pub use list_info::ListInfo;
pub use map_info::MapInfo;
pub use opaque_info::OpaqueInfo;
pub use set_info::SetInfo;
pub use struct_info::StructInfo;
pub use tuple_info::TupleInfo;
pub use tuple_struct_info::TupleStructInfo;
pub use type_info::{ReflectKind, ReflectKindError, TypeInfo};
pub use type_path::{DynamicTypePath, Type, TypePath, TypePathTable};
pub use typed::{DynamicTyped, Typed};
pub use variant_info::{StructVariantInfo, TupleVariantInfo, UnitVariantInfo};
pub use variant_info::{VariantInfo, VariantKind, VariantKindError};

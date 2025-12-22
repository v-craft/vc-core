//! Containers for static storage of type information.
//!
//! This is usually used to implement [`Typed`](crate::info::Typed);
//!
//! ## NonGenericTypeCell
//!
//! For non generic types, provide [`NonGenericTypeInfoCell`] for storing [`TypeInfo`]
//!
//! Internally, there is an [`OnceLock<T>`], almost no additional expenses.
//!
//! There is no `NonGenericTypePathCell` because it can be replaced by a static string literal.
//!
//! ## GenericTypeCell
//!
//! For non generic types, provide the following containers:
//! - [`GenericTypeInfoCell`]: Storage [`TypeInfo`]
//! - [`GenericTypePathCell`]: Storage [`String`]
//!
//! If the type is generic, the `static CELL` inside the function may be shared by different types.
//! Therefore, the inner of this container is a [`TypeIdMap<T>`] wrapped in [`RwLock`].
//!
//! ## Examples
//!
//! See [`NonGenericTypeInfoCell`], [`GenericTypeInfoCell`] and [`GenericTypePathCell`].
//!

use crate::info::TypeInfo;
use alloc::{boxed::Box, string::String};
use core::any::{Any, TypeId};
use vc_os::sync::{OnceLock, PoisonError, RwLock};
use vc_utils::TypeIdMap;

mod sealed {
    use super::TypeInfo;
    use alloc::string::String;
    pub trait TypedProperty: 'static {}

    impl TypedProperty for String {}
    impl TypedProperty for TypeInfo {}
}

use sealed::TypedProperty;

/// Container for static storage of non-generic type information.
///
/// Provide [`NonGenericTypeInfoCell`] for storing [`TypeInfo`].
///
/// Internally, there is an [`OnceLock<T>`], almost no additional expenses.
///
/// There is no `NonGenericTypePathCell` because it can be replaced by a static string literal.
///
/// See more information in [`NonGenericTypeInfoCell`].
pub struct NonGenericTypeCell<T: TypedProperty>(OnceLock<T>);

/// Container for static storage of non-generic type information.
///
/// This is usually used to implement [`Typed`](crate::info::Typed).
///
/// ## Example
///
/// ```ignore
/// #[derive(Reflect)]
/// #[reflect(Typed = false)]
/// struct A2 {
///     a: u32
/// }
///
/// impl Typed for A2 {
///     fn type_info() -> &'static TypeInfo {
///         static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
///         CELL.get_or_init(||TypeInfo::Struct(
///             StructInfo::new::<A2>(&[
///                 NamedField::new::<u32>("a")
///             ])
///         ))
///     }
/// }
///
/// let info = A2::type_info().as_struct().unwrap();
/// assert_eq!(info.field("a").unwrap().type_path(), "u32");
/// assert_eq!(info.type_name(), "A2");
/// ```
pub type NonGenericTypeInfoCell = NonGenericTypeCell<TypeInfo>;

impl<T: TypedProperty> NonGenericTypeCell<T> {
    /// Create a empty cell.
    ///
    /// See [`NonGenericTypeInfoCell`].
    #[inline]
    pub const fn new() -> Self {
        Self(OnceLock::new())
    }

    /// Returns a reference to the `Info` stored in the cell.
    ///
    /// If there is no entry found, a new one will be generated from the given function.
    ///
    /// See [`NonGenericTypeInfoCell`].
    #[inline]
    pub fn get_or_init<F>(&self, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        self.0.get_or_init(f)
    }
}

/// Container for static storage of type information with generics.
///
/// If the type contains generics, the `static CELL` in the function may be shared by multiple types,
/// therefore, the interior of the container was used [`TypeIdMap`] and [`RwLock`].
///
/// See more information in [`GenericTypeInfoCell`] and [`GenericTypePathCell`].
pub struct GenericTypeCell<T: TypedProperty>(RwLock<TypeIdMap<&'static T>>);

/// Container for static storage of type information with generics.
///
/// ## Example
///
/// ```ignore
/// #[derive(Reflect)]
/// #[reflect(Typed = false)]
/// struct A3<T>(T);
///
/// impl<T: Typed + Reflect> Typed for A3<T> {
///     fn type_info() -> &'static TypeInfo {
///         static CELL: GenericTypeInfoCell = GenericTypeInfoCell::new();
///         CELL.get_or_insert::<Self>(||TypeInfo::TupleStruct(
///             TupleStructInfo::new::<A3<T>>(&[
///                 UnnamedField::new::<T>(0)
///             ])
///         ))
///     }
/// }
///
/// let info = <A3<u64>>::type_info().as_tuple_struct().unwrap();
/// assert_eq!(info.field_at(0).unwrap().type_path(), "u64");
/// assert_eq!(info.type_name(), "A3<u64>");
/// ```
pub type GenericTypeInfoCell = GenericTypeCell<TypeInfo>;

/// Container for static storage of type path with generics.
///
/// ## Example
///
/// ```ignore
/// use vc_reflect::impls;
///
/// #[derive(Reflect)]
/// #[reflect(TypePath = false)]
/// enum A4<T>{
///     None,
///     Some(T),
/// }
///
/// impl<T: TypePath> TypePath for A4<T> {
///     fn type_path() -> &'static str {
///         static CELL: GenericTypePathCell = GenericTypePathCell::new();
///         CELL.get_or_insert::<Self>(||{
///             impls::concat(&["test::A4", "<", T::type_path() , ">"])
///         })
///     }
///     fn type_name() -> &'static str {
///         static CELL: GenericTypePathCell = GenericTypePathCell::new();
///         CELL.get_or_insert::<Self>(||{
///             impls::concat(&["A4", "<", T::type_name() , ">"])
///         })
///     }
///     fn type_ident() -> &'static str { "A4" }
/// }
///
/// assert_eq!(<A4<i32>>::type_path(), "test::A4<i32>");
/// assert_eq!(<A4<u8>>::type_name(), "A4<u8>");
/// ```
pub type GenericTypePathCell = GenericTypeCell<String>;

impl<T: TypedProperty> GenericTypeCell<T> {
    /// Create a empty cell.
    #[inline]
    pub const fn new() -> Self {
        Self(RwLock::new(TypeIdMap::new()))
    }

    /// Returns a reference to the `Info` stored in the cell.
    ///
    /// This method will then return the correct `Info` reference for the given type `T`.
    /// If there is no entry found, a new one will be generated from the given function.
    #[inline(always)]
    pub fn get_or_insert<G: Any + ?Sized>(&self, f: impl FnOnce() -> T) -> &T {
        // Separate to reduce code compilation times
        self.get_or_insert_by_type_id(TypeId::of::<G>(), f)
    }

    // Separate to reduce code compilation times
    #[inline(never)]
    fn get_or_insert_by_type_id(&self, type_id: TypeId, f: impl FnOnce() -> T) -> &T {
        match self.get_by_type_id(type_id) {
            Some(info) => info,
            None => self.insert_by_type_id(type_id, f()),
        }
    }

    // Separate to reduce code compilation times
    #[inline(never)]
    fn get_by_type_id(&self, type_id: TypeId) -> Option<&T> {
        self.0
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&type_id)
            .copied()
    }

    // Separate to reduce code compilation times
    #[inline(never)]
    fn insert_by_type_id(&self, type_id: TypeId, value: T) -> &T {
        self.0
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .get_or_insert(type_id, || Box::leak(Box::new(value)))
    }
}

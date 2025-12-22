use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::format;
use core::any::TypeId;

use crate::Reflect;
use crate::derive::TypePath;
use crate::impls::NonGenericTypeInfoCell;
use crate::info::{OpaqueInfo, TypeInfo, TypePath, Typed};
use crate::ops::{ApplyError, ReflectCloneError};
use crate::registry::{TypeRegistry, TypeTraitDefault};

/// # `SkipSerde` - Field Serialization Control
///
/// A custom attribute used to skip serialization and deserialization of specific fields.
///
/// **Important**: This type does not support [`Reflect::reflect_clone`] or [`Reflect::to_dynamic`].
/// Users should not attempt to access it through [`CustomAttributes`](crate::info::CustomAttributes).
///
/// ## Key Notes
///
/// - **Scope**: Only effective for fields (not applicable to enum variant)
/// - **Limitation**: If `#[reflect(serde)]` is enabled on the type, the serde crate's
///   implementation will be used instead, rendering this attribute ineffective.
///
/// **Important**: This attribute cannot be used in newtype field.
/// (If the type is a tuple-struct or tuple-variant there is only one field,
/// it cannot be marked with this attribute.)
///
/// ## Usage Examples
///
/// ### `SkipSerde::None` - Complete Skipping
///
/// Skips both serialization and deserialization entirely.
///
/// ```no_run
/// # use core::marker::PhantomData;
/// # use vc_reflect::{derive::Reflect, serde::SkipSerde};
/// #[derive(Reflect)]
/// struct A<T> {
///     #[reflect(@SkipSerde::None)]
///     _marker: PhantomData<T>,
///     text: String
/// }
/// ```
///
/// ### `SkipSerde::Default` - Use Default Value
///
/// Skips serialization and uses the default value during deserialization.
///
/// **Requirement**: The field's type must implement `TypeTraitDefault` (marked with `#[reflect(default)]`).
///
/// ```no_run
/// # use core::marker::PhantomData;
/// # use vc_reflect::{derive::Reflect, serde::SkipSerde};
/// #[derive(Reflect)]
/// #[reflect(default)]
/// struct A<T> {
///     #[reflect(@SkipSerde::Default)]
///     _marker: PhantomData<T>,  // PhantomData satisfies the default requirement
///     text: String
/// }
/// impl<T> Default for A<T>{
///     /* ... */
/// # fn default() -> Self {
/// #     A{ _marker: PhantomData, text: String::from("") }
/// # }
/// }
/// ```
///
/// ### `SkipSerde::Clone` - Clone Existing Value
///
/// Skips serialization and clones the field's value during deserialization.
///
/// **Recommendation**: Use the [`SkipSerde::clone`] function instead of the [`SkipSerde::Clone`]
/// enum variant directly. The function validates [`Reflect::reflect_clone`] availability in debug builds.
///
/// ```no_run
/// # use vc_reflect::{derive::Reflect, serde::SkipSerde};
/// #[derive(Reflect)]
/// struct A {
///     text: String,
///     #[reflect(@SkipSerde::clone::<&'static str>(""))]
///     docs: &'static str,
/// }
/// ```
#[derive(TypePath)]
#[reflect(type_path = "vc_reflect::serde::SkipSerde")]
pub enum SkipSerde {
    /// Skip directly when deserializing
    None,
    /// Use default values when deserializing, The type needs to register `TypeTraitDefault`
    Default,
    /// Clone a value when deserializing, need to support reflect_clone and have the correct type.
    Clone(Box<dyn Reflect>),
}

impl SkipSerde {
    pub fn clone<T: Reflect>(x: T) -> Self {
        crate::cfg::debug! {
            match x.reflect_clone() {
                Ok(v) => assert_eq!(
                    v.ty_id(),
                    TypeId::of::<T>(),
                    "`SkipSerde::Clone` type mismatched: {}",
                    x.reflect_type_path(),
                ),
                Err(err) => panic!("`SkipSerde::Clone` error: {err}"),
            }
        }

        Self::Clone(x.into_boxed_reflect())
    }

    pub(crate) fn get<E: serde_core::de::Error>(
        &self,
        id: TypeId,
        registry: &TypeRegistry,
    ) -> Result<Option<Box<dyn Reflect>>, E> {
        match self {
            SkipSerde::None => Ok(None),
            SkipSerde::Default => {
                if let Some(generator) = registry.get_type_trait::<TypeTraitDefault>(id) {
                    Ok(Some(generator.default()))
                } else {
                    Err(E::custom(
                        "`SkipSerde::Default` but `TypeTraitDefault` was not found.",
                    ))
                }
            }
            SkipSerde::Clone(reflect) => {
                crate::cfg::debug! {
                    if {
                        if reflect.ty_id() != id {
                            return Err(E::custom(
                                "`SkipSerde::Clone` but type mismatched.",
                            ));
                        }
                        match reflect.reflect_clone() {
                            Ok(val) => Ok(Some(val)),
                            Err(err) => Err(E::custom(format!(
                                "`SkipSerde::Clone` but `reflect_clone` failed: {err} ."
                            ))),
                        }
                    } else {
                        Ok(Some(reflect.reflect_clone().unwrap()))
                    }
                }
            }
        }
    }
}

impl Typed for SkipSerde {
    fn type_info() -> &'static TypeInfo {
        static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
        CELL.get_or_init(|| TypeInfo::Opaque(OpaqueInfo::new::<Self>()))
    }
}

impl Reflect for SkipSerde {
    crate::reflection::impl_reflect_cast_fn!(Opaque);

    /// # Should not be used.
    fn try_apply(&mut self, _value: &dyn Reflect) -> Result<(), ApplyError> {
        Err(ApplyError::NotSupport {
            type_path: Cow::Borrowed(Self::type_path()),
        })
    }

    /// # Should not be used.
    fn reflect_clone(&self) -> Result<Box<dyn Reflect>, ReflectCloneError> {
        Err(ReflectCloneError::NotSupport {
            type_path: Cow::Borrowed(Self::type_path()),
        })
    }

    /// # Should not be used.
    fn to_dynamic(&self) -> Box<dyn Reflect> {
        Box::new(SkipSerde::None)
    }

    fn reflect_debug(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SkipSerde::None => f.write_str("SkipSerde::None"),
            SkipSerde::Default => f.write_str("SkipSerde::Default"),
            SkipSerde::Clone(val) => {
                f.write_str("SkipSerde::Clone(")?;
                val.reflect_debug(f)?;
                f.write_str(")")
            }
        }
    }
}

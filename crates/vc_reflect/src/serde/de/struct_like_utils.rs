use alloc::format;
use alloc::string::{String, ToString};
use core::fmt;

use serde_core::de::{Error, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde_core::{Deserialize, Deserializer};

use super::error_utils::make_custom_error;
use super::{DeserializeDriver, DeserializeProcessor};

use crate::info::{NamedField, StructInfo, StructVariantInfo};
use crate::ops::DynamicStruct;
use crate::registry::TypeRegistry;
use crate::serde::SkipSerde;

// -----------------------------------------------------------------------------
// Infomation trait

/// A helper trait for accessing type information from struct-like types.
pub(super) trait StructLikeInfo {
    fn field<E: Error>(&self, name: &str) -> Result<&NamedField, E>;
    fn field_at<E: Error>(&self, index: usize) -> Result<&NamedField, E>;
    fn field_len(&self) -> usize;
    fn iter_fields(&self) -> impl ExactSizeIterator<Item = &NamedField>;
}

impl StructLikeInfo for StructInfo {
    fn field<E: Error>(&self, name: &str) -> Result<&NamedField, E> {
        Self::field(self, name).ok_or_else(|| {
            Error::custom(format!(
                "no field named `{}` on struct `{}`",
                name,
                self.type_path(),
            ))
        })
    }

    fn field_at<E: Error>(&self, index: usize) -> Result<&NamedField, E> {
        Self::field_at(self, index).ok_or_else(|| {
            Error::custom(format!(
                "no field at index `{}` on struct `{}`",
                index,
                self.type_path(),
            ))
        })
    }

    #[inline]
    fn field_len(&self) -> usize {
        Self::field_len(self)
    }

    #[inline]
    fn iter_fields(&self) -> impl ExactSizeIterator<Item = &NamedField> {
        self.iter()
    }
}

impl StructLikeInfo for StructVariantInfo {
    fn field<E: Error>(&self, name: &str) -> Result<&NamedField, E> {
        Self::field(self, name).ok_or_else(|| {
            Error::custom(format!(
                "no field named `{}` on variant `{}`",
                name,
                self.name(),
            ))
        })
    }

    fn field_at<E: Error>(&self, index: usize) -> Result<&NamedField, E> {
        Self::field_at(self, index).ok_or_else(|| {
            Error::custom(format!(
                "no field at index `{}` on variant `{}`",
                index,
                self.name(),
            ))
        })
    }

    #[inline]
    fn field_len(&self) -> usize {
        Self::field_len(self)
    }

    #[inline]
    fn iter_fields(&self) -> impl ExactSizeIterator<Item = &NamedField> {
        self.iter()
    }
}

// -----------------------------------------------------------------------------
// Ident parser

#[derive(Debug, Clone, Eq, PartialEq)]
struct Ident(pub String);

impl<'de> Deserialize<'de> for Ident {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct IdentVisitor;

        impl<'de> Visitor<'de> for IdentVisitor {
            type Value = Ident;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("identifier")
            }

            #[inline]
            fn visit_str<E: Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(Ident(value.to_string()))
            }

            #[inline]
            fn visit_string<E: Error>(self, value: String) -> Result<Self::Value, E> {
                Ok(Ident(value))
            }
        }

        deserializer.deserialize_identifier(IdentVisitor)
    }
}

// -----------------------------------------------------------------------------
// struct visitor

/// Deserializes a [struct-like] type from a mapping of fields, returning a [`DynamicStruct`].
///
/// [struct-like]: StructLikeInfo
pub(super) fn visit_struct<'de, T, V, P>(
    map: &mut V,
    info: &'static T,
    registry: &TypeRegistry,
    mut processor: Option<&mut P>,
) -> Result<DynamicStruct, V::Error>
where
    T: StructLikeInfo,
    V: MapAccess<'de>,
    P: DeserializeProcessor,
{
    let mut dynamic_struct = DynamicStruct::with_capacity(info.field_len());

    while let Some(Ident(key)) = map.next_key::<Ident>()? {
        let field = info.field::<V::Error>(&key)?;

        // cannot skip here, we need to call `next_value_seed`.

        let Some(type_meta) = registry.get(field.ty_id()) else {
            return Err(make_custom_error(format!(
                "no TypeMeta found for type `{}`",
                field.type_info().type_path(),
            )));
        };

        let value = map.next_value_seed(DeserializeDriver::new_internal(
            type_meta,
            registry,
            processor.as_deref_mut(),
        ))?;
        dynamic_struct.extend_boxed(key, value);
    }

    for field in info.iter_fields() {
        if let Some(skip_serde) = field.get_attribute::<SkipSerde>()
            && let Some(val) = skip_serde.get(field.ty_id(), registry)?
        {
            dynamic_struct.extend_boxed(field.name(), val);
        }
    }

    Ok(dynamic_struct)
}

/// Deserializes a [struct-like] type from a sequence of fields, returning a [`DynamicStruct`].
///
/// [struct-like]: StructLikeInfo
pub(super) fn visit_struct_seq<'de, T, V, P>(
    seq: &mut V,
    info: &T,
    registry: &TypeRegistry,
    mut processor: Option<&mut P>,
) -> Result<DynamicStruct, V::Error>
where
    T: StructLikeInfo,
    V: SeqAccess<'de>,
    P: DeserializeProcessor,
{
    let len = info.field_len();
    let mut dynamic_struct = DynamicStruct::with_capacity(len);

    for index in 0..len {
        let field = info.field_at::<V::Error>(index)?;

        if let Some(skip_serde) = field.get_attribute::<SkipSerde>() {
            if let Some(value) = skip_serde.get(field.ty_id(), registry)? {
                dynamic_struct.extend_boxed(field.name(), value);
            }
            continue;
        }

        let Some(type_meta) = registry.get(field.ty_id()) else {
            return Err(make_custom_error(format!(
                "no TypeMeta found for type `{}`",
                field.type_info().type_path(),
            )));
        };

        let value = seq.next_element_seed(DeserializeDriver::new_internal(
            type_meta,
            registry,
            processor.as_deref_mut(),
        ))?;

        let value = match value {
            Some(val) => val,
            None => {
                return Err(make_custom_error(format!(
                    "invalid length, expected: `{}`, actual: `{}`",
                    len, index,
                )));
            }
        };

        dynamic_struct.extend_boxed(field.name(), value);
    }

    if seq.next_element::<IgnoredAny>()?.is_some() {
        return Err(make_custom_error(format!(
            "invalid length, expected: `{}`, actual: `> {}`",
            len, len,
        )));
    }

    Ok(dynamic_struct)
}

use alloc::format;

use serde_core::de::{Error, IgnoredAny, SeqAccess};

use super::error_utils::make_custom_error;
use super::{DeserializeDriver, DeserializeProcessor};

use crate::info::{TupleInfo, TupleStructInfo, TupleVariantInfo, UnnamedField};
use crate::ops::DynamicTuple;
use crate::registry::TypeRegistry;
use crate::serde::SkipSerde;

// -----------------------------------------------------------------------------
// Infomation trait

pub(super) trait TupleLikeInfo {
    fn field_at<E: Error>(&self, index: usize) -> Result<&UnnamedField, E>;
    fn field_len(&self) -> usize;
}

impl TupleLikeInfo for TupleInfo {
    fn field_at<E: Error>(&self, index: usize) -> Result<&UnnamedField, E> {
        match Self::field_at(self, index) {
            Some(info) => Ok(info),
            None => Err(make_custom_error(format!(
                "no field at index `{}` on tuple `{}`",
                index,
                self.type_path(),
            ))),
        }
    }

    #[inline]
    fn field_len(&self) -> usize {
        Self::field_len(self)
    }
}

impl TupleLikeInfo for TupleStructInfo {
    fn field_at<E: Error>(&self, index: usize) -> Result<&UnnamedField, E> {
        match Self::field_at(self, index) {
            Some(info) => Ok(info),
            None => Err(make_custom_error(format!(
                "no field at index `{}` on tuple-struct `{}`",
                index,
                self.type_path(),
            ))),
        }
    }

    #[inline]
    fn field_len(&self) -> usize {
        Self::field_len(self)
    }
}

impl TupleLikeInfo for TupleVariantInfo {
    fn field_at<E: Error>(&self, index: usize) -> Result<&UnnamedField, E> {
        match Self::field_at(self, index) {
            Some(info) => Ok(info),
            None => Err(make_custom_error(format!(
                "no field at index `{}` on tuple variant `{}`",
                index,
                self.name(),
            ))),
        }
    }

    #[inline]
    fn field_len(&self) -> usize {
        Self::field_len(self)
    }
}

// -----------------------------------------------------------------------------
// tuple visitor

/// Deserializes a [tuple-like] type from a sequence of elements, returning a [`DynamicTuple`].
///
/// [tuple-like]: TupleLikeInfo
pub(super) fn visit_tuple<'de, T, V, P>(
    seq: &mut V,
    info: &T,
    registry: &TypeRegistry,
    mut processor: Option<&mut P>,
) -> Result<DynamicTuple, V::Error>
where
    T: TupleLikeInfo,
    V: SeqAccess<'de>,
    P: DeserializeProcessor,
{
    let len = info.field_len();
    let mut dynamic_tuple = DynamicTuple::with_capacity(len);

    for index in 0..len {
        let field_info = info.field_at::<V::Error>(index)?;

        // skip serde fields
        if let Some(skip_serde) = field_info.get_attribute::<SkipSerde>() {
            if let Some(val) = skip_serde.get(field_info.ty_id(), registry)? {
                dynamic_tuple.extend_boxed(val);
            }
            continue;
        }

        let Some(type_meta) = registry.get(field_info.ty_id()) else {
            return Err(make_custom_error(format!(
                "no TypeMeta found for type `{}`",
                field_info.type_info().type_path(),
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

        dynamic_tuple.extend_boxed(value);
    }

    if seq.next_element::<IgnoredAny>()?.is_some() {
        return Err(make_custom_error(format!(
            "invalid length, expected: `{}`, actual: `> {}`",
            len, len,
        )));
    }

    Ok(dynamic_tuple)
}

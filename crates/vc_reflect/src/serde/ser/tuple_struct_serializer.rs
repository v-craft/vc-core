use alloc::format;

use serde_core::ser::SerializeTupleStruct;
use serde_core::{Serialize, Serializer};
use vc_utils::vec::FastVec;

use super::error_utils::make_custom_error;
use super::{SerializeDriver, SerializeProcessor};

use crate::info::TypeInfo;
use crate::ops::TupleStruct;
use crate::registry::TypeRegistry;
use crate::serde::SkipSerde;

/// A serializer for [`TupleStruct`] values.
pub(super) struct TupleStructSerializer<'a, P: SerializeProcessor> {
    pub tuple_struct: &'a dyn TupleStruct,
    pub registry: &'a TypeRegistry,
    pub processor: Option<&'a P>,
}

impl<P: SerializeProcessor> Serialize for TupleStructSerializer<'_, P> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let Some(type_info) = self.tuple_struct.represented_type_info() else {
            return Err(make_custom_error(format! {
                "cannot get represented type info for `{}`",
                self.tuple_struct.reflect_type_path()
            }));
        };

        let TypeInfo::TupleStruct(tuple_struct_info) = type_info else {
            return Err(make_custom_error(format!(
                "expected tuple struct but received {type_info:?}"
            )));
        };

        let field_indecies = tuple_struct_info
            .iter()
            .filter(|f| !f.has_attribute::<SkipSerde>())
            .map(|f| f.index())
            .collect::<FastVec<_, 8>>();

        let mut state = serializer
            .serialize_tuple_struct(tuple_struct_info.type_ident(), field_indecies.len())?;

        for &index in field_indecies.as_slice() {
            if let Some(value) = self.tuple_struct.field(index) {
                state.serialize_field(&SerializeDriver::new_internal(
                    value,
                    self.registry,
                    self.processor,
                ))?;
            } else {
                return Err(make_custom_error(format!(
                    "field `{index}` was missing while serializing type {}",
                    tuple_struct_info.type_path()
                )));
            }
        }

        state.end()
    }
}

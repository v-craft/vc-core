use alloc::format;

use serde_core::ser::SerializeStruct;
use serde_core::{Serialize, Serializer};
use vc_utils::vec::FastVec;

use super::error_utils::make_custom_error;
use super::{SerializeDriver, SerializeProcessor};

use crate::info::TypeInfo;
use crate::ops::Struct;
use crate::registry::TypeRegistry;
use crate::serde::SkipSerde;

/// A serializer for [`Struct`] values.
pub(super) struct StructSerializer<'a, P: SerializeProcessor> {
    pub struct_value: &'a dyn Struct,
    pub registry: &'a TypeRegistry,
    pub processor: Option<&'a P>,
}

impl<P: SerializeProcessor> Serialize for StructSerializer<'_, P> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let Some(type_info) = self.struct_value.represented_type_info() else {
            return Err(make_custom_error(format! {
                "cannot get represented type info for `{}`",
                self.struct_value.reflect_type_path()
            }));
        };

        let TypeInfo::Struct(struct_info) = type_info else {
            return Err(make_custom_error(format!(
                "expected struct but received {type_info:?}"
            )));
        };

        let field_names = struct_info
            .iter()
            .filter(|f| !f.has_attribute::<SkipSerde>())
            .map(|f| f.name())
            .collect::<FastVec<_, 8>>();

        let mut state = serializer.serialize_struct(struct_info.type_ident(), field_names.len())?;

        for &name in field_names.as_slice() {
            if let Some(value) = self.struct_value.field(name) {
                state.serialize_field(
                    name,
                    &SerializeDriver::new_internal(value, self.registry, self.processor),
                )?;
            } else {
                return Err(make_custom_error(format!(
                    "field `{name}` was missing while serializing type {}",
                    struct_info.type_path()
                )));
            }
        }

        state.end()
    }
}

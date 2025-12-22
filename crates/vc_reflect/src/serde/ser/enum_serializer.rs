use alloc::format;

use serde_core::ser::{SerializeStructVariant, SerializeTupleVariant};
use serde_core::{Serialize, Serializer};
use vc_utils::vec::FastVec;

use super::error_utils::make_custom_error;
use super::{SerializeDriver, SerializeProcessor};

use crate::info::{TypeInfo, VariantInfo, VariantKind};
use crate::ops::Enum;
use crate::registry::TypeRegistry;
use crate::serde::SkipSerde;

/// A serializer for [`Enum`] values.
pub(super) struct EnumSerializer<'a, P: SerializeProcessor> {
    pub enum_value: &'a dyn Enum,
    pub registry: &'a TypeRegistry,
    pub processor: Option<&'a P>,
}

impl<P: SerializeProcessor> Serialize for EnumSerializer<'_, P> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let Some(type_info) = self.enum_value.represented_type_info() else {
            return Err(make_custom_error(format! {
                "cannot get represented type info for `{}`",
                self.enum_value.reflect_type_path(),
            }));
        };

        let TypeInfo::Enum(enum_info) = type_info else {
            return Err(make_custom_error(format!(
                "expected enum but received {type_info:?}"
            )));
        };

        let variant_index = self.enum_value.variant_index() as u32;

        let Some(variant_info) = enum_info.variant_at(variant_index as usize) else {
            return Err(make_custom_error(format!(
                "variant at index `{variant_index}` does not exist for `{}`",
                enum_info.type_path(),
            )));
        };

        let field_len = self.enum_value.field_len();
        let enum_name = enum_info.type_ident();
        let variant_name = variant_info.name();
        let variant_kind = variant_info.variant_kind();

        crate::cfg::debug! {{
            // the variant name from data
            let variant_name_from_data = self.enum_value.variant_name();
            // the correct index of the data variant name.
            let Some(right_index) = enum_info.index_of(variant_name_from_data) else {
                panic!(
                    "variant name `{variant_name_from_data}` does not exist for `{}`",
                    enum_info.type_path(),
                );
            };
            assert_eq!(
                variant_name_from_data, variant_name,
                "Mismatched variant index, expected: `{}`, actual: `{}`.",
                right_index, variant_index,
            );
        }}

        match variant_kind {
            VariantKind::Unit => {
                if enum_name == "Option" && enum_info.module_path() == Some("core::option") {
                    serializer.serialize_none()
                } else {
                    serializer.serialize_unit_variant(enum_name, variant_index, variant_name)
                }
            }
            VariantKind::Struct => {
                let VariantInfo::Struct(struct_info) = variant_info else {
                    return Err(make_custom_error(format!(
                        "expected struct variant type but received {variant_info:?}"
                    )));
                };

                let field_names = struct_info
                    .iter()
                    .filter(|f| !f.has_attribute::<SkipSerde>())
                    .map(|f| f.name())
                    .collect::<FastVec<_, 8>>();

                let mut state = serializer.serialize_struct_variant(
                    enum_name,
                    variant_index,
                    variant_name,
                    field_names.len(),
                )?;

                for &name in field_names.as_slice() {
                    if let Some(value) = self.enum_value.field(name) {
                        state.serialize_field(
                            name,
                            &SerializeDriver::new_internal(value, self.registry, self.processor),
                        )?;
                    } else {
                        return Err(make_custom_error(format!(
                            "field `{name}` was missing while serializing type {}",
                            enum_info.type_path()
                        )));
                    }
                }

                state.end()
            }
            VariantKind::Tuple if field_len == 1 => {
                crate::cfg::debug! {{
                    let VariantInfo::Tuple(tuple_info) = variant_info else {
                        panic!("expected tuple variant type but received {variant_info:?}");
                    };
                    assert_eq!(
                        tuple_info.field_len(), field_len,
                        "Mismatched variant field length: expect: `{}`, actual: `{}`.",
                        tuple_info.field_len(), field_len,
                    );
                    let first_field = tuple_info.field_at(0).unwrap();
                    assert!(
                        !first_field.has_attribute::<SkipSerde>(),
                        "newtype can not skip field in serialization and deserialization."
                    );
                }}

                let field = self.enum_value.field_at(0).expect("valid index");

                if enum_name == "Option" && enum_info.module_path() == Some("core::option") {
                    serializer.serialize_some(&SerializeDriver::new_internal(
                        field,
                        self.registry,
                        self.processor,
                    ))
                } else {
                    serializer.serialize_newtype_variant(
                        enum_name,
                        variant_index,
                        variant_name,
                        &SerializeDriver::new_internal(field, self.registry, self.processor),
                    )
                }
            }
            VariantKind::Tuple => {
                let VariantInfo::Tuple(tuple_info) = variant_info else {
                    return Err(make_custom_error(format!(
                        "expected tuple variant type but received {variant_info:?}"
                    )));
                };

                let field_indecies = tuple_info
                    .iter()
                    .filter(|f| !f.has_attribute::<SkipSerde>())
                    .map(|f| f.index())
                    .collect::<FastVec<_, 8>>();

                let mut state = serializer.serialize_tuple_variant(
                    enum_name,
                    variant_index,
                    variant_name,
                    field_indecies.len(),
                )?;

                for &index in field_indecies.as_slice() {
                    if let Some(value) = self.enum_value.field_at(index) {
                        state.serialize_field(&SerializeDriver::new_internal(
                            value,
                            self.registry,
                            self.processor,
                        ))?;
                    } else {
                        return Err(make_custom_error(format!(
                            "field `{index}` was missing while serializing type {}",
                            enum_info.type_path()
                        )));
                    }
                }

                state.end()
            }
        }
    }
}

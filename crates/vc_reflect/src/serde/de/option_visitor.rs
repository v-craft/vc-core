use alloc::format;
use core::fmt;

use serde_core::Deserializer;
use serde_core::de::{DeserializeSeed, Error, Visitor};

use super::error_utils::make_custom_error;
use super::{DeserializeDriver, DeserializeProcessor};

use crate::info::{EnumInfo, VariantInfo};
use crate::ops::{DynamicEnum, DynamicTuple};
use crate::registry::TypeRegistry;

/// A [`Visitor`] for deserializing [`Option`] values.
pub(super) struct OptionVisitor<'a, P: DeserializeProcessor> {
    pub enum_info: &'static EnumInfo,
    pub registry: &'a TypeRegistry,
    pub processor: Option<&'a mut P>,
}

impl<'de, P: DeserializeProcessor> Visitor<'de> for OptionVisitor<'_, P> {
    type Value = DynamicEnum;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("reflected option value of type ")?;
        formatter.write_str(self.enum_info.type_path())
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let option = DynamicEnum::new("None", ());
        Ok(option)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let Some(variant_info) = self.enum_info.variant("Some") else {
            return Err(make_custom_error(format!(
                "invalid variant, expected `Some(_)` but got: {:?}",
                self.enum_info
            )));
        };

        match variant_info {
            VariantInfo::Tuple(tuple_info) if tuple_info.field_len() == 1 => {
                let Some(field) = tuple_info.field_at(0) else {
                    return Err(make_custom_error(format!(
                        "invalid variant, expected `Some(_)` but got: {tuple_info:?}"
                    )));
                };

                let Some(type_meta) = self.registry.get(field.type_id()) else {
                    return Err(make_custom_error(format!(
                        "no TypeMeta found for type `{}`",
                        field.type_info().type_path(),
                    )));
                };

                let de = DeserializeDriver::new_internal(type_meta, self.registry, self.processor);

                let mut value = DynamicTuple::with_capacity(1);

                value.extend_boxed(de.deserialize(deserializer)?);

                let option = DynamicEnum::new("Some", value);
                Ok(option)
            }
            info => Err(Error::custom(format!(
                "invalid variant, expected `Some(_)` but got: {info:?}"
            ))),
        }
    }
}

use alloc::format;
use core::fmt::{self, Formatter};

use serde_core::Deserializer;
use serde_core::de::{DeserializeSeed, SeqAccess, Visitor};

use crate::info::TupleStructInfo;
use crate::ops::{DynamicTuple, DynamicTupleStruct};
use crate::registry::TypeRegistry;

use super::error_utils::make_custom_error;
use super::tuple_like_utils::visit_tuple;
use super::{DeserializeDriver, DeserializeProcessor};

/// A [`Visitor`] for deserializing [`TupleStruct`] values.
///
/// [`TupleStruct`]: crate::TupleStruct
pub(super) struct TupleStructVisitor<'a, P: DeserializeProcessor> {
    pub tuple_struct_info: &'static TupleStructInfo,
    pub registry: &'a TypeRegistry,
    pub processor: Option<&'a mut P>,
}

impl<'de, P: DeserializeProcessor> Visitor<'de> for TupleStructVisitor<'_, P> {
    type Value = DynamicTupleStruct;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("reflected tuple struct value")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
    where
        V: SeqAccess<'de>,
    {
        visit_tuple(
            &mut seq,
            self.tuple_struct_info,
            self.registry,
            self.processor,
        )
        .map(DynamicTuple::into_tuple_struct)
    }

    fn visit_newtype_struct<D: Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        let field_info = self
            .tuple_struct_info
            .field_at(0)
            .ok_or(make_custom_error("Field at index 0 not found"))?;

        let Some(type_meta) = self.registry.get(field_info.type_id()) else {
            return Err(make_custom_error(format!(
                "no TypeMeta found for type `{}`",
                field_info.type_info().type_path(),
            )));
        };

        let mut dynamic_tuple = DynamicTupleStruct::with_capacity(1);

        crate::cfg::debug! {
            assert!(
                !field_info.has_attribute::<crate::serde::SkipSerde>(),
                "newtype can not skip field in serialization and deserialization."
            );
        }

        let de = DeserializeDriver::new_internal(type_meta, self.registry, self.processor);
        let value = de.deserialize(deserializer)?;

        dynamic_tuple.extend_boxed(value);

        Ok(dynamic_tuple)
    }
}

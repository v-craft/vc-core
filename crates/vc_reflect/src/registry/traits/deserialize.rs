use alloc::boxed::Box;

use serde_core::{Deserialize, Deserializer};

use crate::Reflect;
use crate::info::{TypePath, Typed};
use crate::registry::FromType;

/// A container providing `serde` deserialization support for reflected types.
///
/// Internally stores function pointers corresponding to specific types. When given a reflected type,
/// it downcasts to the concrete type and invokes the `serde` deserialization functions.
///
/// This is usually used for the internal implementation of [`vc_reflect::serde`],
/// see more infomation in [`ReflectDeserializeDriver`](vc_reflect::serde::ReflectDeserializeDriver).
///
/// # Examples
///
/// ```
/// use core::any::TypeId;
/// use vc_reflect::registry::{TypeTraitDeserialize, TypeRegistry};
/// use vc_reflect::{Reflect, derive::Reflect};
/// use serde::Deserialize;
///
/// #[derive(Reflect, Deserialize, PartialEq, Debug)]
/// #[reflect(deserialize)]
/// struct MyStruct {
///   value: i32
/// }
///
/// let input = r#"(value:123)"#;
///
/// let mut registry = TypeRegistry::new();
/// registry.register::<MyStruct>();
///
/// let processor = registry.get_type_trait::<TypeTraitDeserialize>(TypeId::of::<MyStruct>()).unwrap();
///
/// let mut deserializer = ron::Deserializer::from_str(input).unwrap();
///
/// let val = processor.deserialize(&mut deserializer).unwrap();
///
/// assert_eq!(val.take::<MyStruct>().unwrap(), MyStruct{ value: 123 });
/// ```
#[derive(Clone)]
pub struct TypeTraitDeserialize {
    func: fn(
        deserializer: &mut dyn erased_serde::Deserializer,
    ) -> Result<Box<dyn Reflect>, erased_serde::Error>,
}

impl TypeTraitDeserialize {
    /// Deserializes a reflected value.
    ///
    /// See [`TypeTraitDeserialize`] for examples.
    #[inline(always)]
    pub fn deserialize<'de, D: Deserializer<'de>>(
        &self,
        deserializer: D,
    ) -> Result<Box<dyn Reflect>, D::Error> {
        let mut erased = <dyn erased_serde::Deserializer>::erase(deserializer);
        (self.func)(&mut erased).map_err(<D::Error as serde_core::de::Error>::custom)
    }
}

impl<T: for<'a> Deserialize<'a> + Typed + Reflect> FromType<T> for TypeTraitDeserialize {
    fn from_type() -> Self {
        Self {
            func: |deserializer| Ok(Box::new(T::deserialize(deserializer)?)),
        }
    }
}

// Explicitly implemented here so that code readers do not need
// to ponder the principles of proc-macros in advance.
impl TypePath for TypeTraitDeserialize {
    #[inline(always)]
    fn type_path() -> &'static str {
        "vc_reflect::registry::TypeTraitDeserialize"
    }

    #[inline(always)]
    fn type_name() -> &'static str {
        "TypeTraitDeserialize"
    }

    #[inline(always)]
    fn type_ident() -> &'static str {
        "TypeTraitDeserialize"
    }

    #[inline(always)]
    fn module_path() -> Option<&'static str> {
        Some("vc_reflect::registry")
    }
}

// -----------------------------------------------------------------------------
// Tests

#[cfg(test)]
mod tests {
    use super::TypeTraitDeserialize;
    use crate::info::TypePath;

    #[test]
    fn type_path() {
        assert!(TypeTraitDeserialize::type_path() == "vc_reflect::registry::TypeTraitDeserialize");
        assert!(TypeTraitDeserialize::module_path() == Some("vc_reflect::registry"));
        assert!(TypeTraitDeserialize::type_ident() == "TypeTraitDeserialize");
        assert!(TypeTraitDeserialize::type_name() == "TypeTraitDeserialize");
    }
}

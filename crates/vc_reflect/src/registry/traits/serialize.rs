use serde_core::{Serialize, Serializer};

use crate::Reflect;
use crate::info::Typed;
use crate::registry::FromType;

/// A container providing `serde` serialization support for reflected types.
///
/// Internally stores function pointers corresponding to specific types. When given a reflected type,
/// it downcasts to the concrete type and invokes the `serde` serialization functions.
///
/// This is usually used for the internal implementation of [`vc_reflect::serde`],
/// see more infomation in [`ReflectSerializeDriver`](vc_reflect::serde::ReflectSerializeDriver).
///
/// # Safety
///
/// Passing an incorrectly typed `&dyn Reflect` value will cause a panic.
///
/// # Examples
///
/// ```
/// # use vc_reflect::{registry::{TypeTraitSerialize, TypeRegistry}, derive::Reflect, Reflect};
/// use serde::Serialize;
///
/// #[derive(Reflect, Serialize, PartialEq, Debug)]
/// #[reflect(serialize)]
/// struct MyStruct {
///   value: i32
/// }
///
/// let mut registry = TypeRegistry::new();
/// registry.register::<MyStruct>();
///
/// let input = MyStruct { value: 123 };
///
/// let processor = registry.get_type_trait::<TypeTraitSerialize>(input.ty_id()).unwrap();
///
/// let mut output = String::new();
/// let mut serializer = ron::Serializer::new(&mut output, None).unwrap();
///
/// processor.serialize(&input, &mut serializer);
///
/// assert_eq!(output, r#"(value:123)"#);
/// ```
#[derive(Clone)]
pub struct TypeTraitSerialize {
    fun: fn(value: &dyn Reflect) -> &dyn erased_serde::Serialize,
}

impl<T: erased_serde::Serialize + Typed + Reflect> FromType<T> for TypeTraitSerialize {
    fn from_type() -> Self {
        Self {
            fun: |value| match value.downcast_ref::<T>() {
                Some(val) => val as &dyn erased_serde::Serialize,
                None => {
                    panic!(
                        "Serial type mismatched, Serial Type `{}` with Value Type: {}",
                        T::type_path(),
                        value.reflect_type_path(),
                    );
                }
            },
        }
    }
}

impl TypeTraitSerialize {
    /// Call T's [`Serialize`]
    ///
    /// [`TypeTraitSerialize`] does not have a type flag,
    /// but the functions used internally are type specific.
    ///
    /// # Panic
    /// - Mismatched Type
    #[inline(always)]
    pub fn serialize<S: Serializer>(
        &self,
        value: &dyn Reflect,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        (self.fun)(value).serialize(serializer)
    }
}

crate::derive::impl_type_path!(::vc_reflect::registry::TypeTraitSerialize);

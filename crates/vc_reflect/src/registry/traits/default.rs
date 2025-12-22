use alloc::boxed::Box;

use crate::Reflect;
use crate::info::Typed;
use crate::registry::FromType;

/// A container providing [`Default`] support for reflected types.
///
/// Then, you can create a reflect value using [`TypeRegistry`] and [`TypeId`] (or [`TypePath`]).
///
/// # Examples
///
/// ```
/// use vc_reflect::{Reflect, registry::{TypeRegistry, TypeTraitDefault}};
///
/// let registry = TypeRegistry::new(); // `new` will register some basic type
///
/// let generator = registry
///     .get_with_type_name("String").unwrap()
///     .get_trait::<TypeTraitDefault>().unwrap();
///
/// let s: Box<dyn Reflect> = generator.default();
///
/// assert_eq!(s.take::<String>().unwrap(), "");
/// ```
///
/// [`TypePath`]: crate::info::TypePath::type_path
/// [`TypeRegistry`]: crate::registry::TypeRegistry
/// [`TypeId`]: core::any::TypeId
#[derive(Clone)]
pub struct TypeTraitDefault {
    func: fn() -> Box<dyn Reflect>,
}

impl TypeTraitDefault {
    /// Call T's [`Default`]
    ///
    /// [`TypeTraitDefault`] does not have a type flag,
    /// but the functions used internally are type specific.
    #[inline(always)]
    pub fn default(&self) -> Box<dyn Reflect> {
        (self.func)()
    }
}

impl<T: Default + Typed + Reflect> FromType<T> for TypeTraitDefault {
    fn from_type() -> Self {
        Self {
            func: || Box::<T>::default(),
        }
    }
}

crate::derive::impl_type_path!(::vc_reflect::registry::TypeTraitDefault);

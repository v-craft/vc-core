use alloc::boxed::Box;

use crate::Reflect;
use crate::info::{TypePath, Typed};
use crate::registry::FromType;

/// A container providing [`Default`] support for reflected types.
///
/// Then, you can create a reflect value using [`TypeRegistry`] and [`TypeId`] (or [`TypePath`]).
///
/// # Examples
///
/// ```
/// use vc_reflect::{Reflect, registry::{TypeRegistry, ReflectDefault}};
///
/// let registry = TypeRegistry::new(); // `new` will register some basic type
///
/// let generator = registry
///     .get_with_type_name("String").unwrap()
///     .get_trait::<ReflectDefault>().unwrap();
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
pub struct ReflectDefault {
    func: fn() -> Box<dyn Reflect>,
}

impl ReflectDefault {
    /// Call T's [`Default`]
    ///
    /// [`ReflectDefault`] does not have a type flag,
    /// but the functions used internally are type specific.
    #[inline(always)]
    pub fn default(&self) -> Box<dyn Reflect> {
        (self.func)()
    }
}

impl<T: Default + Typed + Reflect> FromType<T> for ReflectDefault {
    fn from_type() -> Self {
        Self {
            func: || Box::<T>::default(),
        }
    }
}

// Explicitly implemented here so that code readers do not need
// to ponder the principles of proc-macros in advance.
impl TypePath for ReflectDefault {
    #[inline(always)]
    fn type_path() -> &'static str {
        "vc_reflect::registry::ReflectDefault"
    }

    #[inline(always)]
    fn type_name() -> &'static str {
        "ReflectDefault"
    }

    #[inline(always)]
    fn type_ident() -> &'static str {
        "ReflectDefault"
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
    use super::ReflectDefault;
    use crate::info::TypePath;

    #[test]
    fn type_path() {
        assert!(ReflectDefault::type_path() == "vc_reflect::registry::ReflectDefault");
        assert!(ReflectDefault::module_path() == Some("vc_reflect::registry"));
        assert!(ReflectDefault::type_ident() == "ReflectDefault");
        assert!(ReflectDefault::type_name() == "ReflectDefault");
    }
}

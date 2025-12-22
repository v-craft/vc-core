use crate::{
    FromReflect, Reflect,
    impls::NonGenericTypeInfoCell,
    info::{OpaqueInfo, TypeInfo, TypePath, Typed},
    ops::ApplyError,
    registry::TypeTraitFromReflect,
    registry::{FromType, GetTypeMeta, TypeMeta, TypeTraitFromPtr},
};
use alloc::boxed::Box;
use core::panic::Location;

impl TypePath for &'static Location<'static> {
    fn type_path() -> &'static str {
        "core::panic::Location"
    }

    fn type_name() -> &'static str {
        "Location"
    }

    fn type_ident() -> &'static str {
        "Location"
    }
}

impl Typed for &'static Location<'static> {
    fn type_info() -> &'static TypeInfo {
        static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
        CELL.get_or_init(|| TypeInfo::Opaque(OpaqueInfo::new::<Self>()))
    }
}

impl Reflect for &'static Location<'static> {
    crate::reflection::impl_reflect_cast_fn!(Opaque);

    fn try_apply(&mut self, value: &dyn Reflect) -> Result<(), ApplyError> {
        if let Some(value) = value.downcast_ref::<Self>() {
            self.clone_from(value);
            Ok(())
        } else {
            Err(ApplyError::MismatchedTypes {
                from_type: value.reflect_type_path().into(),
                to_type: <Self as TypePath>::type_path().into(),
            })
        }
    }

    fn to_dynamic(&self) -> Box<dyn Reflect> {
        Box::new(*self)
    }

    fn reflect_clone(&self) -> Result<Box<dyn Reflect>, crate::ops::ReflectCloneError> {
        Ok(Box::new(*self))
    }

    fn reflect_partial_eq(&self, value: &dyn Reflect) -> Option<bool> {
        if let Some(value) = value.downcast_ref::<Self>() {
            Some(PartialEq::eq(self, value))
        } else {
            Some(false)
        }
    }

    fn reflect_hash(&self) -> Option<u64> {
        let mut hasher = crate::reflect_hasher();
        core::hash::Hash::hash(self, &mut hasher);
        Some(core::hash::Hasher::finish(&hasher))
    }

    fn reflect_debug(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
}

impl GetTypeMeta for &'static Location<'static> {
    fn get_type_meta() -> TypeMeta {
        let mut meta = TypeMeta::with_capacity::<Self>(2);
        meta.insert_trait::<TypeTraitFromPtr>(FromType::<Self>::from_type());
        meta.insert_trait::<TypeTraitFromReflect>(FromType::<Self>::from_type());
        meta
    }
}

impl FromReflect for &'static Location<'static> {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        reflect.downcast_ref::<Self>().copied()
    }
}

crate::derive::impl_auto_register!(&'static Location<'static>);

use crate::{
    FromReflect, Reflect,
    derive::{impl_reflect_opaque, impl_type_path},
    impls::NonGenericTypeInfoCell,
    info::{OpaqueInfo, TypeInfo, Typed},
    registry::{
        FromType, GetTypeMeta, TypeMeta, TypeTraitDeserialize, TypeTraitFromPtr,
        TypeTraitFromReflect, TypeTraitSerialize
    }
};
use alloc::borrow::Cow;
use std::path::Path;

impl_reflect_opaque!(::std::path::PathBuf(full));

impl_type_path!(::std::path::Path);

impl Typed for &'static Path {
    fn type_info() -> &'static TypeInfo {
        static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
        CELL.get_or_init(|| TypeInfo::Opaque(OpaqueInfo::new::<Self>()))
    }
}

impl Reflect for &'static Path {
    crate::impls::impl_simple_type_reflect!(Opaque);
}

impl FromReflect for &'static Path {
    #[inline]
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        reflect.downcast_ref::<Self>().copied()
    }
}

impl GetTypeMeta for &'static Path {
    fn get_type_meta() -> TypeMeta {
        let mut type_meta = TypeMeta::with_capacity::<Self>(3);
        type_meta.insert_trait::<TypeTraitFromPtr>(FromType::<Self>::from_type());
        type_meta.insert_trait::<TypeTraitFromReflect>(FromType::<Self>::from_type());
        type_meta.insert_trait::<TypeTraitSerialize>(FromType::<Self>::from_type());
        type_meta
    }
}

impl Typed for Cow<'static, Path> {
    fn type_info() -> &'static TypeInfo {
        static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
        CELL.get_or_init(|| TypeInfo::Opaque(OpaqueInfo::new::<Self>()))
    }
}

impl Reflect for Cow<'static, Path> {
    crate::impls::impl_simple_type_reflect!(Opaque);
}

impl FromReflect for Cow<'static, Path> {
    #[inline]
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        reflect.downcast_ref::<Self>().cloned()
    }
}

impl GetTypeMeta for Cow<'static, Path> {
    fn get_type_meta() -> TypeMeta {
        let mut type_meta: TypeMeta = TypeMeta::with_capacity::<Self>(4);
        type_meta.insert_trait::<TypeTraitFromPtr>(FromType::<Self>::from_type());
        type_meta.insert_trait::<TypeTraitFromReflect>(FromType::<Self>::from_type());
        type_meta.insert_trait::<TypeTraitDeserialize>(FromType::<Self>::from_type());
        type_meta.insert_trait::<TypeTraitSerialize>(FromType::<Self>::from_type());
        type_meta
    }
}

crate::derive::impl_auto_register!(Cow<'static, Path>);
crate::derive::impl_auto_register!(&'static Path);

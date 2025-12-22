use crate::{
    FromReflect, Reflect,
    impls::NonGenericTypeInfoCell,
    info::{OpaqueInfo, TypeInfo, TypePath, Typed},
    registry::{
        FromType, GetTypeMeta, TypeMeta, TypeTraitFromPtr, TypeTraitFromReflect, TypeTraitSerialize,
    },
};

impl TypePath for str {
    #[inline]
    fn type_path() -> &'static str {
        "str"
    }
    #[inline]
    fn type_name() -> &'static str {
        "str"
    }
    #[inline]
    fn type_ident() -> &'static str {
        "str"
    }
}

// impl TypePath for &'static str
// See `native_ref.rs`

impl Typed for &'static str {
    fn type_info() -> &'static TypeInfo {
        static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
        CELL.get_or_init(|| TypeInfo::Opaque(OpaqueInfo::new::<Self>()))
    }
}

impl Reflect for &'static str {
    crate::impls::impl_simple_type_reflect!(Opaque);
}

impl GetTypeMeta for &'static str {
    fn get_type_meta() -> TypeMeta {
        let mut type_meta = TypeMeta::with_capacity::<Self>(3);
        type_meta.insert_trait::<TypeTraitFromPtr>(FromType::<Self>::from_type());
        type_meta.insert_trait::<TypeTraitFromReflect>(FromType::<Self>::from_type());
        type_meta.insert_trait::<TypeTraitSerialize>(FromType::<Self>::from_type());
        type_meta
    }
}

impl FromReflect for &'static str {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        reflect.downcast_ref::<Self>().copied()
    }
}

crate::derive::impl_auto_register!(&'static str);

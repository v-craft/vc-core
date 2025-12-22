use proc_macro2::TokenStream;
use quote::quote;

#[inline]
pub(crate) fn type_meta_(vc_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vc_reflect_path::registry::TypeMeta
    }
}

#[inline]
pub(crate) fn get_type_meta_(vc_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vc_reflect_path::registry::GetTypeMeta
    }
}

#[inline]
pub(crate) fn from_type_(vc_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vc_reflect_path::registry::FromType
    }
}

#[inline]
pub(crate) fn type_registry_(vc_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vc_reflect_path::registry::TypeRegistry
    }
}

#[inline]
pub(crate) fn type_trait_default_(vc_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vc_reflect_path::registry::TypeTraitDefault
    }
}

#[inline]
pub(crate) fn type_trait_from_ptr_(vc_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vc_reflect_path::registry::TypeTraitFromPtr
    }
}

#[inline]
pub(crate) fn type_trait_from_reflect_(vc_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vc_reflect_path::registry::TypeTraitFromReflect
    }
}

#[inline]
pub(crate) fn type_trait_serialize_(vc_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vc_reflect_path::registry::TypeTraitSerialize
    }
}

#[inline]
pub(crate) fn type_trait_deserialize_(vc_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vc_reflect_path::registry::TypeTraitDeserialize
    }
}

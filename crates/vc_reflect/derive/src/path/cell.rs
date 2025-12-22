use proc_macro2::TokenStream;
use quote::quote;

#[inline(always)]
pub(crate) fn non_generic_type_info_cell_(vc_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vc_reflect_path::impls::NonGenericTypeInfoCell
    }
}

#[inline(always)]
pub(crate) fn generic_type_info_cell_(vc_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vc_reflect_path::impls::GenericTypeInfoCell
    }
}

#[inline(always)]
pub(crate) fn generic_type_path_cell_(vc_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vc_reflect_path::impls::GenericTypePathCell
    }
}

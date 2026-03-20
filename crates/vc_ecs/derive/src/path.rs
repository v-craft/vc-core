//! This independent module is used to provide the required path.
//! So as to minimize changes when the `vc_ecs` structure is modified.

use proc_macro2::TokenStream;
use quote::quote;

// -----------------------------------------------------------------------------
// Crate Path

/// Get the correct access path to the `vc_ecs` crate.
pub(crate) fn vc_ecs() -> syn::Path {
    vc_macro_utils::Manifest::shared(|manifest| manifest.get_crate_path("vc_ecs"))
}

pub(crate) use vc_macro_utils::full_path as fp;

#[inline(always)]
pub(crate) fn macro_utils_(vc_ecs_path: &syn::Path) -> TokenStream {
    quote! {
        #vc_ecs_path::__macro_exports::macro_utils
    }
}

// -----------------------------------------------------------------------------
// Resource

#[inline(always)]
pub(crate) fn cloner_(vc_ecs_path: &syn::Path) -> TokenStream {
    quote! {
        #vc_ecs_path::utils::Cloner
    }
}

#[inline(always)]
pub(crate) fn resource_(vc_ecs_path: &syn::Path) -> TokenStream {
    quote! {
        #vc_ecs_path::resource::Resource
    }
}

#[inline(always)]
pub(crate) fn component_(vc_ecs_path: &syn::Path) -> TokenStream {
    quote! {
        #vc_ecs_path::component::Component
    }
}

#[inline(always)]
pub(crate) fn required_(vc_ecs_path: &syn::Path) -> TokenStream {
    quote! {
        #vc_ecs_path::component::Required
    }
}

#[inline(always)]
pub(crate) fn component_storage_(vc_ecs_path: &syn::Path) -> TokenStream {
    quote! {
        #vc_ecs_path::component::ComponentStorage
    }
}

#[inline(always)]
pub(crate) fn component_collector_(vc_ecs_path: &syn::Path) -> TokenStream {
    quote! {
        #vc_ecs_path::component::ComponentCollector
    }
}

#[inline(always)]
pub(crate) fn component_writer_(vc_ecs_path: &syn::Path) -> TokenStream {
    quote! {
        #vc_ecs_path::component::ComponentWriter
    }
}

#[inline(always)]
pub(crate) fn bundle_(vc_ecs_path: &syn::Path) -> TokenStream {
    quote! {
        #vc_ecs_path::bundle::Bundle
    }
}

#[inline(always)]
pub(crate) fn schedule_label_(vc_ecs_path: &syn::Path) -> TokenStream {
    quote! {
        #vc_ecs_path::schedule::ScheduleLabel
    }
}

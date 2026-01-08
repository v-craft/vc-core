//! This independent module is used to provide the required path.
//! So as to minimize changes when the `vc_reflect` structure is modified.
//!
//! The only special feature is the path of vc_reflect itself,
//! See [`vc_reflect`] function doc.

use proc_macro2::TokenStream;
use quote::quote;

// -----------------------------------------------------------------------------
// Crate Path

/// Get the correct access path to the `vc_reflect` crate.
///
/// Not all modules can access the reflection crate itself through `vc_reflect`,
/// we have to scan the builder's `cargo.toml`.
///
/// 1. For crates that depend on `vc_reflect`, `::vc_reflect` is returned here`.
/// 2. For crates that depend on `voidcraft`, `::voidcraft::reflect` is returned here`.
/// 3. For crates that depend on `vc_core`, `::vc_core::reflect` is returned here`.
/// 4. For crates that depend on `vc`, `::vc::reflect` is returned here`.
/// 5. For other situations, `::vc_reflect` is returned here, but this may be incorrect.
///
/// The cost of this function is relatively high (accessing files, obtaining read-write lock permissions, querying content...),
/// so the crate path is mainly obtained through parameter passing rather than reacquiring.
pub(crate) fn vc_reflect() -> syn::Path {
    vc_macro_utils::Manifest::shared(|manifest| manifest.get_crate_path("vc_reflect"))
}

// -----------------------------------------------------------------------------
// Modules

mod cell;
mod info;
mod ops;
mod registry;

pub(crate) mod fp;

// -----------------------------------------------------------------------------
// Internal API

pub(crate) use cell::*;
pub(crate) use info::*;
pub(crate) use ops::*;
pub(crate) use registry::*;

// mod access;
// `vc_reflect::access` does not require additional content.

#[inline(always)]
pub(crate) fn macro_utils_(vc_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vc_reflect_path::__macro_exports::macro_utils
    }
}

#[cfg(feature = "auto_register")]
#[inline(always)]
pub(crate) fn auto_register_(vc_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vc_reflect_path::__macro_exports::auto_register
    }
}

#[inline(always)]
pub(crate) fn reflect_(vc_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vc_reflect_path::Reflect
    }
}

#[inline(always)]
pub(crate) fn from_reflect_(vc_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vc_reflect_path::FromReflect
    }
}

#[inline(always)]
pub(crate) fn reflect_hasher_(vc_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vc_reflect_path::reflect_hasher
    }
}

//! Full path type markers for Rust core items.

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

/// Full Path (FP) for [`Any`](core::any::Any)
pub struct AnyFP;
/// Full Path (FP) for [`Clone`]
pub struct CloneFP;
/// Full Path (FP) for [`Default`]
pub struct DefaultFP;
/// Full Path (FP) for [`Option`]
pub struct OptionFP;
/// Full Path (FP) for [`Result`]
pub struct ResultFP;
/// Full Path (FP) for [`Send`]
pub struct SendFP;
/// Full Path (FP) for [`Sync`]
pub struct SyncFP;
/// Full Path (FP) for [`PartialEq`]
pub struct PartialEqFP;
/// Full Path (FP) for [`PartialOrd`]
pub struct PartialOrdFP;
/// Full Path (FP) for [`Eq`]
pub struct EqFP;
/// Full Path (FP) for [`Hash`](core::hash::Hash)
pub struct HashFP;
/// Full Path (FP) for [`Hasher`](core::hash::Hasher)
pub struct HasherFP;
/// Full Path (FP) for [`Debug`](core::fmt::Debug)
pub struct DebugFP;
/// Full Path (FP) for [`TypeId`](core::any::TypeId)
pub struct TypeIdFP;

impl ToTokens for AnyFP {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        quote!(::core::any::Any).to_tokens(tokens);
    }
}

impl ToTokens for CloneFP {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        quote!(::core::clone::Clone).to_tokens(tokens);
    }
}

impl ToTokens for DefaultFP {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        quote!(::core::default::Default).to_tokens(tokens);
    }
}

impl ToTokens for OptionFP {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        quote!(::core::option::Option).to_tokens(tokens);
    }
}

impl ToTokens for ResultFP {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        quote!(::core::result::Result).to_tokens(tokens);
    }
}

impl ToTokens for SendFP {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        quote!(::core::marker::Send).to_tokens(tokens);
    }
}

impl ToTokens for SyncFP {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        quote!(::core::marker::Sync).to_tokens(tokens);
    }
}

impl ToTokens for PartialEqFP {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        quote!(::core::cmp::PartialEq).to_tokens(tokens);
    }
}

impl ToTokens for PartialOrdFP {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        quote!(::core::cmp::PartialOrd).to_tokens(tokens);
    }
}

impl ToTokens for EqFP {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        quote!(::core::cmp::Eq).to_tokens(tokens);
    }
}

impl ToTokens for HashFP {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        quote!(::core::hash::Hash).to_tokens(tokens);
    }
}

impl ToTokens for HasherFP {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        quote!(::core::hash::Hasher).to_tokens(tokens);
    }
}

impl ToTokens for DebugFP {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        quote!(::core::fmt::Debug).to_tokens(tokens);
    }
}

impl ToTokens for TypeIdFP {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        quote!(::core::any::TypeId).to_tokens(tokens);
    }
}

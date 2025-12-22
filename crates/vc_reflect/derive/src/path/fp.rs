use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

/// Full Path (FP) for [`core::any::Any`]
pub(crate) struct AnyFP;
/// Full Path (FP) for [`Clone`]
pub(crate) struct CloneFP;
/// Full Path (FP) for [`Default`]
pub(crate) struct DefaultFP;
/// Full Path (FP) for [`Option`]
pub(crate) struct OptionFP;
/// Full Path (FP) for [`Result`]
pub(crate) struct ResultFP;
/// Full Path (FP) for [`Send`]
pub(crate) struct SendFP;
/// Full Path (FP) for [`Sync`]
pub(crate) struct SyncFP;
/// Full Path (FP) for [`PartialEq`]
pub(crate) struct PartialEqFP;
/// Full Path (FP) for [`Hash`](core::hash::Hash)
pub(crate) struct HashFP;
/// Full Path (FP) for [`Hasher`](core::hash::Hasher)
pub(crate) struct HasherFP;
/// Full Path (FP) for [`Debug`](core::fmt::Debug)
pub(crate) struct DebugFP;
/// Full Path (FP) for [`Debug`](core::any::TypeId)
pub(crate) struct TypeIdFP;

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

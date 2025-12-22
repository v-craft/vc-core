use quote::{quote, quote_spanned};

use super::{get_auto_register_impl, impl_trait_get_type_meta};
use super::{impl_trait_reflect, impl_trait_type_path, impl_trait_typed};

use crate::derive_data::ReflectMeta;

/// Implement full reflect for unit type.
pub(crate) fn impl_unit(meta: &ReflectMeta) -> proc_macro2::TokenStream {
    // trait: TypePath
    let type_path_trait_tokens = if meta.attrs().impl_switchs.impl_type_path {
        impl_trait_type_path(meta)
    } else {
        crate::utils::empty()
    };

    // trait: Typed
    let typed_trait_tokens = if meta.attrs().impl_switchs.impl_typed {
        impl_trait_typed(meta, meta.to_info_tokens(), false)
    } else {
        crate::utils::empty()
    };

    // trait: Reflect
    let reflect_trait_tokens = if meta.attrs().impl_switchs.impl_reflect {
        let try_apply_tokens = get_unit_try_apply_impl(meta);
        let to_dynamic_tokens = get_unit_to_dynamic_impl(meta);
        let reflect_clone_tokens = get_unit_clone_impl(meta);
        let reflect_partial_eq_tokens = get_unit_partial_eq_impl(meta);
        let reflect_hash_tokens = get_unit_hash_impl(meta);
        let reflect_debug_tokens = get_unit_debug_impl(meta);

        impl_trait_reflect(
            meta,
            quote!(Opaque),
            try_apply_tokens,
            to_dynamic_tokens,
            reflect_clone_tokens,
            reflect_partial_eq_tokens,
            reflect_hash_tokens,
            reflect_debug_tokens,
            false,
        )
    } else {
        crate::utils::empty()
    };

    // trait: GetTypeTraits
    let get_type_meta_tokens = if meta.attrs().impl_switchs.impl_get_type_meta {
        impl_trait_get_type_meta(meta, crate::utils::empty())
    } else {
        crate::utils::empty()
    };

    // trait: FromReflect
    let from_reflect_tokens = if meta.attrs().impl_switchs.impl_from_reflect {
        impl_unit_from_reflect(meta)
    } else {
        crate::utils::empty()
    };

    // featuer: auto_resiter
    let auto_register_tokens = get_auto_register_impl(meta);

    quote! {
        #type_path_trait_tokens

        #typed_trait_tokens

        #reflect_trait_tokens

        #get_type_meta_tokens

        #from_reflect_tokens

        #auto_register_tokens
    }
}

/// Generate `Reflect::try_apply` implementation tokens.
fn get_unit_try_apply_impl(meta: &ReflectMeta) -> proc_macro2::TokenStream {
    use crate::path::fp::{CloneFP, OptionFP, ResultFP};

    let vc_reflect_path = meta.vc_reflect_path();
    let macro_utils_ = crate::path::macro_utils_(vc_reflect_path);
    let reflect_ = crate::path::reflect_(vc_reflect_path);
    let apply_error_ = crate::path::apply_error_(vc_reflect_path);
    let type_path_ = crate::path::type_path_(vc_reflect_path);
    let dynamic_type_path_ = crate::path::dynamic_type_path_(vc_reflect_path);

    let err = quote! {
        #ResultFP::Err(
            #apply_error_::MismatchedTypes {
                from_type: #macro_utils_::Cow::Borrowed(#dynamic_type_path_::reflect_type_path(__input)),
                to_type: #macro_utils_::Cow::Borrowed(<Self as #type_path_>::type_path()),
            }
        )
    };

    if meta.attrs().avail_traits.clone.is_some() {
        quote! {
            fn try_apply(&mut self, __input: &dyn #reflect_) -> #ResultFP<(), #apply_error_> {
                if let #OptionFP::Some(__input) = <dyn #reflect_>::downcast_ref::<Self>(__input) {
                    *self = #CloneFP::clone(__input);
                    return #ResultFP::Ok(());
                }
                #err
            }
        }
    } else {
        quote! {
            fn try_apply(&mut self, __input: &dyn #reflect_) -> #ResultFP<(), #apply_error_> {
                if <dyn #reflect_>::is::<Self>(__input) {
                    return #ResultFP::Ok(());
                }
                #err
            }
        }
    }
}

/// Generate `Reflect::to_dynamic` implementation tokens.
fn get_unit_to_dynamic_impl(meta: &ReflectMeta) -> proc_macro2::TokenStream {
    use crate::path::fp::CloneFP;

    let vc_reflect_path = meta.vc_reflect_path();
    let macro_utils_ = crate::path::macro_utils_(vc_reflect_path);
    let reflect_ = crate::path::reflect_(vc_reflect_path);

    if meta.attrs().avail_traits.clone.is_some() {
        quote! {
            #[inline]
            fn to_dynamic(&self) -> #macro_utils_::Box<dyn #reflect_> {
                #macro_utils_::Box::new(<Self as #CloneFP>::clone(self))
            }
        }
    } else {
        quote! {
            #[inline]
            fn to_dynamic(&self) -> #macro_utils_::Box<dyn #reflect_> {
                #macro_utils_::Box::new(Self)
            }
        }
    }
}

/// Generate `Reflect::reflect_clone` implementation tokens.
fn get_unit_clone_impl(meta: &ReflectMeta) -> proc_macro2::TokenStream {
    use crate::path::fp::{CloneFP, ResultFP};

    let vc_reflect_path = meta.vc_reflect_path();
    let macro_utils_ = crate::path::macro_utils_(vc_reflect_path);
    let reflect_ = crate::path::reflect_(vc_reflect_path);
    let reflect_clone_error_ = crate::path::reflect_clone_error_(vc_reflect_path);

    if let Some(span) = meta.attrs().avail_traits.clone {
        quote_spanned! { span =>
            #[inline]
            fn reflect_clone(&self) -> #ResultFP<#macro_utils_::Box<dyn #reflect_>, #reflect_clone_error_> {
                #ResultFP::Ok(#macro_utils_::Box::new(<Self as #CloneFP>::clone(self)))
            }
        }
    } else {
        quote! {
            #[inline]
            fn reflect_clone(&self) -> #ResultFP<#macro_utils_::Box<dyn #reflect_>, #reflect_clone_error_> {
                #ResultFP::Ok(#macro_utils_::Box::new(Self))
            }
        }
    }
}

/// Generate `Reflect::reflect_partial_eq` implementation tokens.
fn get_unit_partial_eq_impl(meta: &ReflectMeta) -> proc_macro2::TokenStream {
    use crate::path::fp::{OptionFP, PartialEqFP};
    let vc_reflect_path = meta.vc_reflect_path();
    let reflect_ = crate::path::reflect_(vc_reflect_path);

    if let Some(span) = meta.attrs().avail_traits.partial_eq {
        quote_spanned! { span =>
            #[inline]
            fn reflect_partial_eq(&self, __input: &dyn #reflect_) -> #OptionFP<bool> {
                if let #OptionFP::Some(__input) = <dyn #reflect_>::downcast_ref::<Self>(__input) {
                    return #OptionFP::Some( #PartialEqFP::eq(self, __input) );
                }
                #OptionFP::Some( false )
            }
        }
    } else {
        quote! {
            #[inline]
            fn reflect_partial_eq(&self, __input: &dyn #reflect_) -> #OptionFP<bool> {
                if <dyn #reflect_>::is::<Self>(__input) {
                    #OptionFP::Some( true )
                } else {
                    #OptionFP::Some( false )
                }
            }
        }
    }
}

/// Generate `Reflect::reflect_hash` implementation tokens.
fn get_unit_hash_impl(meta: &ReflectMeta) -> proc_macro2::TokenStream {
    use crate::path::fp::{HashFP, HasherFP, OptionFP, TypeIdFP};

    let vc_reflect_path = meta.vc_reflect_path();
    let reflect_hasher = crate::path::reflect_hasher_(vc_reflect_path);

    if let Some(span) = meta.attrs().avail_traits.hash {
        quote_spanned! { span =>
            #[inline]
            fn reflect_hash(&self) -> #OptionFP<u64> {
                let mut hasher = #reflect_hasher();
                <Self as #HashFP>::hash(self, &mut hasher);
                #OptionFP::Some(#HasherFP::finish(&hasher))
            }
        }
    } else {
        quote! {
            #[inline]
            fn reflect_hash(&self) -> #OptionFP<u64> {
                let mut hasher = #reflect_hasher();
                <#TypeIdFP as #HashFP>::hash( &::core::any::TypeId::of::<Self>() , &mut hasher);
                #OptionFP::Some(#HasherFP::finish(&hasher))
            }
        }
    }
}

/// Generate `Reflect::reflect_debug` implementation tokens.
fn get_unit_debug_impl(meta: &ReflectMeta) -> proc_macro2::TokenStream {
    use crate::path::fp::DebugFP;

    let type_path_ = crate::path::type_path_(meta.vc_reflect_path());

    if let Some(span) = meta.attrs().avail_traits.debug {
        quote_spanned! { span =>
            #[inline]
            fn reflect_debug(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                <Self as #DebugFP>::fmt(self, f)
            }
        }
    } else {
        quote! {
            #[inline]
            fn reflect_debug(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(f, <Self as #type_path_>::type_path())
            }
        }
    }
}

/// Generate `FromReflect` trait implementation tokens.
fn impl_unit_from_reflect(meta: &ReflectMeta) -> proc_macro2::TokenStream {
    use crate::path::fp::OptionFP;

    let vc_reflect_path = meta.vc_reflect_path();
    let reflect_ = crate::path::reflect_(vc_reflect_path);
    let from_reflect_ = crate::path::from_reflect_(vc_reflect_path);

    let real_ident = meta.real_ident();
    let (impl_generics, ty_generics, where_clause) = meta.split_generics(false, false, false);
    // ↑ unit struct has no fields, and this implementation is not related to generics..

    quote! {
        impl #impl_generics #from_reflect_ for #real_ident #ty_generics #where_clause  {
            fn from_reflect(__input: &dyn #reflect_) -> #OptionFP<Self> {
                if <dyn #reflect_>::is::<Self>(__input) {
                    #OptionFP::Some(Self)
                } else {
                    #OptionFP::None
                }
            }
        }
    }
}

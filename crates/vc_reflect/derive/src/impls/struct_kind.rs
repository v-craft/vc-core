use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote, quote_spanned};
use syn::Ident;

use super::{get_auto_register_impl, get_struct_clone_impl};
use super::{get_common_try_apply_tokens, impl_trait_typed};
use super::{impl_struct_from_reflect, impl_trait_reflect};
use super::{impl_trait_get_type_meta, impl_trait_type_path};

use crate::derive_data::{FieldAccessors, ReflectMeta, ReflectStruct};

/// Implement full reflect for struct type.
pub(crate) fn impl_struct(info: &ReflectStruct) -> TokenStream {
    let meta = info.meta();

    // trait: TypePath
    let type_path_trait_tokens = if meta.attrs().impl_switchs.impl_type_path {
        impl_trait_type_path(meta)
    } else {
        crate::utils::empty()
    };

    // trait: Typed
    let typed_trait_tokens = if meta.attrs().impl_switchs.impl_typed {
        impl_trait_typed(meta, info.to_info_tokens(false), false)
    } else {
        crate::utils::empty()
    };

    // trait: Struct
    let struct_trait_tokens = if meta.attrs().impl_switchs.impl_struct {
        impl_trait_struct(info)
    } else {
        crate::utils::empty()
    };

    // trait: Reflect
    let reflect_trait_tokens = if meta.attrs().impl_switchs.impl_reflect {
        let try_apply_tokens = get_struct_try_apply_impl(meta);
        let to_dynamic_tokens = get_struct_to_dynamic_impl(meta);
        let reflect_clone_tokens = get_struct_clone_impl(info);
        let reflect_partial_eq_tokens = get_struct_partial_eq_impl(meta);
        let reflect_hash_tokens = get_struct_hash_impl(meta);
        let reflect_debug_tokens = get_struct_debug_impl(meta);

        impl_trait_reflect(
            meta,
            quote!(Struct),
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
        impl_trait_get_type_meta(meta, get_registry_dependencies(info))
    } else {
        crate::utils::empty()
    };

    // trait: FromReflect
    let get_from_reflect_tokens = if meta.attrs().impl_switchs.impl_from_reflect {
        impl_struct_from_reflect(info, false)
    } else {
        crate::utils::empty()
    };

    // featuer: auto_resiter
    let auto_register_tokens = get_auto_register_impl(meta);

    quote! {
        #type_path_trait_tokens

        #typed_trait_tokens

        #struct_trait_tokens

        #reflect_trait_tokens

        #get_type_meta_tokens

        #get_from_reflect_tokens

        #auto_register_tokens
    }
}

/// Generate `Struct` trait implementation tokens.
fn impl_trait_struct(info: &ReflectStruct) -> TokenStream {
    use crate::path::fp::OptionFP;
    let meta = info.meta();

    let vc_reflect_path = meta.vc_reflect_path();
    let struct_ = crate::path::struct_(vc_reflect_path);
    let reflect_ = crate::path::reflect_(vc_reflect_path);
    let struct_field_iter_ = crate::path::struct_field_iter_(vc_reflect_path);
    let dynamic_struct_ = crate::path::dynamic_struct_(vc_reflect_path);
    let option_ = OptionFP.to_token_stream();

    let field_names = info
        .active_fields()
        .map(|field| {
            field
                .data
                .ident
                .as_ref()
                .map(ToString::to_string)
                .expect("Struct should not have unnamed fields.")
        })
        .collect::<Vec<String>>();

    let FieldAccessors {
        fields_ref,
        fields_mut,
        field_indices,
        field_count,
    } = FieldAccessors::new(info);

    let real_ident = meta.real_ident();
    let (impl_generics, ty_generics, where_clause) = meta.split_generics(true, false, false);

    quote! {
        impl #impl_generics #struct_ for #real_ident #ty_generics #where_clause {
            fn field(&self, name: &str) -> #OptionFP<&dyn #reflect_> {
                match name {
                    #(#field_names => #option_::Some(#fields_ref),)*
                    _ => #OptionFP::None,
                }
            }

            fn field_mut(&mut self, name: &str) -> #OptionFP<&mut dyn #reflect_> {
                match name {
                    #(#field_names => #option_::Some(#fields_mut),)*
                    _ => #OptionFP::None,
                }
            }

            fn field_at(&self, index: usize) -> #OptionFP<&dyn #reflect_> {
                match index {
                    #(#field_indices => #option_::Some(#fields_ref),)*
                    _ => #OptionFP::None,
                }
            }

            fn field_at_mut(&mut self, index: usize) -> #OptionFP<&mut dyn #reflect_> {
                match index {
                    #(#field_indices => #option_::Some(#fields_mut),)*
                    _ => #OptionFP::None,
                }
            }

            fn name_at(&self, index: usize) -> #OptionFP<&str> {
                match index {
                    #(#field_indices => #option_::Some(#field_names),)*
                    _ => #OptionFP::None,
                }
            }

            #[inline]
            fn field_len(&self) -> usize {
                #field_count
            }

            #[inline]
            fn iter_fields(&self) -> #struct_field_iter_ {
                #struct_field_iter_::new(self)
            }

            // Do not use default implementation to reduce `match` queries.
            fn to_dynamic_struct(&self) -> #dynamic_struct_ {
                let mut dynamic = #dynamic_struct_::with_capacity(#struct_::field_len(self));
                dynamic.set_type_info(#reflect_::represented_type_info(self));
                #(dynamic.extend_boxed(#field_names, #reflect_::to_dynamic(#fields_ref));)*
                dynamic
            }
        }
    }
}

/// Generate `Reflect::try_apply` implementation tokens.
fn get_struct_try_apply_impl(meta: &ReflectMeta) -> TokenStream {
    use crate::path::fp::ResultFP;

    let vc_reflect_path = meta.vc_reflect_path();
    let reflect_ = crate::path::reflect_(vc_reflect_path);
    let apply_error_ = crate::path::apply_error_(vc_reflect_path);
    let struct_try_apply_ = crate::path::struct_try_apply_(vc_reflect_path);

    let input_ = Ident::new("__ident", Span::call_site());

    let clone_tokens = get_common_try_apply_tokens(meta, &input_);

    quote! {
        fn try_apply(&mut self, #input_: &dyn #reflect_) -> #ResultFP<(), #apply_error_> {
            #clone_tokens

            #struct_try_apply_(self, #input_)
        }
    }
}

/// Generate `Reflect::to_dynamic` implementation tokens.
fn get_struct_to_dynamic_impl(meta: &ReflectMeta) -> TokenStream {
    let vc_reflect_path = meta.vc_reflect_path();
    let macro_utils_ = crate::path::macro_utils_(vc_reflect_path);
    let reflect_ = crate::path::reflect_(vc_reflect_path);
    let struct_ = crate::path::struct_(vc_reflect_path);

    quote! {
        #[inline]
        fn to_dynamic(&self) -> #macro_utils_::Box<dyn #reflect_> {
            #macro_utils_::Box::new(<Self as #struct_>::to_dynamic_struct(self) )
        }
    }
}

/// Generate `Reflect::reflect_partial_eq` implementation tokens.
fn get_struct_partial_eq_impl(meta: &ReflectMeta) -> TokenStream {
    use crate::path::fp::{OptionFP, PartialEqFP};
    let vc_reflect_path = meta.vc_reflect_path();
    let reflect_ = crate::path::reflect_(vc_reflect_path);

    if let Some(span) = meta.attrs().avail_traits.partial_eq {
        quote_spanned! { span =>
            #[inline]
            fn reflect_partial_eq(&self, other: &dyn #reflect_) -> #OptionFP<bool> {
                if let #OptionFP::Some(value) = other.downcast_ref::<Self>() {
                    return #OptionFP::Some( #PartialEqFP::eq(self, value) );
                }
                #OptionFP::Some( false )
            }
        }
    } else {
        let struct_partial_eq_ = crate::path::struct_partial_eq_(vc_reflect_path);
        quote! {
            #[inline]
            fn reflect_partial_eq(&self, other: &dyn #reflect_) -> #OptionFP<bool> {
                #struct_partial_eq_(self, other)
            }
        }
    }
}

/// Generate `Reflect::reflect_hash` implementation tokens.
fn get_struct_hash_impl(meta: &ReflectMeta) -> TokenStream {
    use crate::path::fp::{HashFP, HasherFP, OptionFP};
    let vc_reflect_path = meta.vc_reflect_path();

    if let Some(span) = meta.attrs().avail_traits.hash {
        let reflect_hasher = crate::path::reflect_hasher_(vc_reflect_path);
        quote_spanned! { span =>
            #[inline]
            fn reflect_hash(&self) -> #OptionFP<u64> {
                let mut hasher = #reflect_hasher();
                <Self as #HashFP>::hash(self, &mut hasher);
                #OptionFP::Some(#HasherFP::finish(&hasher))
            }
        }
    } else {
        let struct_hash_ = crate::path::struct_hash_(vc_reflect_path);
        quote! {
            #[inline]
            fn reflect_hash(&self) -> #OptionFP<u64> {
                #struct_hash_(self)
            }
        }
    }
}

/// Generate `Reflect::reflect_debug` implementation tokens.
fn get_struct_debug_impl(meta: &ReflectMeta) -> TokenStream {
    use crate::path::fp::DebugFP;

    if let Some(span) = meta.attrs().avail_traits.debug {
        quote_spanned! { span =>
            #[inline]
            fn reflect_debug(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                <Self as #DebugFP>::fmt(self, f)
            }
        }
    } else {
        let struct_debug_ = crate::path::struct_debug_(meta.vc_reflect_path());
        quote! {
            #[inline]
            fn reflect_debug(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                #struct_debug_(self, f)
            }
        }
    }
}

/// Generate partial `GetTypeMeta` implementation tokens.
fn get_registry_dependencies(info: &ReflectStruct) -> TokenStream {
    let vc_reflect_path = info.meta().vc_reflect_path();
    let type_registry_ = crate::path::type_registry_(vc_reflect_path);

    let field_types = info.active_fields().map(|x| &x.data.ty);

    quote! {
        fn register_dependencies(__registry: &mut #type_registry_) {
            #(#type_registry_::register::<#field_types>(__registry);)*
        }
    }
}

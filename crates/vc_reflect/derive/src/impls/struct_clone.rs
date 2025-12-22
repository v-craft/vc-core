use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};

use crate::derive_data::ReflectStruct;

// Generate `Reflect::reflect_clone` tokens for struct and tuple-struct.
pub(crate) fn get_struct_clone_impl(info: &ReflectStruct) -> TokenStream {
    use crate::path::fp::{CloneFP, DefaultFP, OptionFP, ResultFP};

    let meta = info.meta();
    let vc_reflect_path = meta.vc_reflect_path();
    let macro_utils_ = crate::path::macro_utils_(vc_reflect_path);
    let reflect_ = crate::path::reflect_(vc_reflect_path);
    let reflect_clone_error_ = crate::path::reflect_clone_error_(vc_reflect_path);
    let type_path_ = crate::path::type_path_(vc_reflect_path);

    if let Some(span) = meta.attrs().avail_traits.clone {
        quote_spanned! { span =>
            #[inline]
            fn reflect_clone(&self) -> #ResultFP<#macro_utils_::Box<dyn #reflect_>, #reflect_clone_error_> {
                #ResultFP::Ok(#macro_utils_::Box::new(<Self as #CloneFP>::clone(self)))
            }
        }
    } else if meta.attrs().avail_traits.default.is_some() {
        let mut tokens = TokenStream::new();

        for field in info.active_fields() {
            let field_ty = &field.data.ty;
            let member = field.to_member();

            tokens.extend(quote! {
                __new_value.#member = #macro_utils_::__reflect_clone_field::<#field_ty>(&self.#member)?;
            });
        }

        quote! {
            fn reflect_clone(&self) -> #ResultFP<#macro_utils_::Box<dyn #reflect_>, #reflect_clone_error_> {
                let mut __new_value = <Self as #DefaultFP>::default();

                #tokens

                #ResultFP::Ok(#macro_utils_::Box::new(__new_value))
            }
        }
    } else {
        for field in info.fields().iter() {
            if let Some(span) = field.attrs.ignore {
                let field_id = field.field_id(vc_reflect_path);
                return quote_spanned! { span =>
                    #[inline]
                    fn reflect_clone(&self) -> #ResultFP<#macro_utils_::Box<dyn #reflect_>, #reflect_clone_error_> {
                        #ResultFP::Err(#reflect_clone_error_::FieldNotCloneable {
                            type_path:  #macro_utils_::Cow::Borrowed(<Self as #type_path_>::type_path())
                            field: #field_id,
                            variant: #OptionFP::None,
                        })
                    }
                };
            }
        }

        let mut tokens = TokenStream::new();

        for field in info.fields().iter() {
            let field_ty = &field.data.ty;
            let member = field.to_member();

            tokens.extend(quote! {
                #member: #macro_utils_::__reflect_clone_field::<#field_ty>(&self.#member)?,
            });
        }

        quote! {
            fn reflect_clone(&self) -> #ResultFP<#macro_utils_::Box<dyn #reflect_>, #reflect_clone_error_> {
                #ResultFP::Ok(#macro_utils_::Box::new(
                    Self {
                        #tokens
                    }
                ) as #macro_utils_::Box<dyn #reflect_>)
            }
        }
    }
}

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Index, parse_quote};

pub(crate) fn impl_derive_bundle(ast: DeriveInput) -> TokenStream {
    let vc_ecs_path = crate::path::vc_ecs();
    let bundle_ = crate::path::bundle_(&vc_ecs_path);
    let component_collector_ = crate::path::component_collector_(&vc_ecs_path);
    let component_writer_ = crate::path::component_writer_(&vc_ecs_path);

    let type_ident = ast.ident;
    let mut generics = ast.generics;
    if generics.type_params().next().is_some() {
        generics
            .make_where_clause()
            .predicates
            .push(parse_quote! { Self: Send + Sync + Sized + 'static });
    } else if generics.lifetimes().next().is_some() {
        generics
            .make_where_clause()
            .predicates
            .push(parse_quote! { Self: 'static });
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let field_access = match &ast.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let access: Vec<_> = fields
                    .named
                    .iter()
                    .map(|field| {
                        let ident = field.ident.as_ref().unwrap();
                        let ty = &field.ty;
                        (quote! { #ident }, ty)
                    })
                    .collect();
                access
            }
            Fields::Unnamed(fields) => {
                let access: Vec<_> = fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(i, field)| {
                        let idx = Index::from(i);
                        let ty = &field.ty;
                        (quote! { #idx }, ty)
                    })
                    .collect();
                access
            }
            Fields::Unit => {
                return quote! {
                    unsafe impl #impl_generics #bundle_ for #type_ident #ty_generics #where_clause {
                        fn collect_components(_collector: &mut #component_collector_) {}
                        unsafe fn write_explicit(_writer: &mut #component_writer_, _base: usize) {}
                        unsafe fn write_required(_writer: &mut #component_writer_) {}
                    }
                }
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(&type_ident, "Bundle can only be derived for structs")
                .into_compile_error()
                .into();
        }
    };

    let collect_calls = field_access.iter().map(|(_, ty)| {
        quote! {
            <#ty as #bundle_>::collect_components(__collector__);
        }
    });

    let write_explicit_calls = field_access.iter().map(|(ident, ty)| {
        quote! {
            unsafe {
                let __offset__ = ::core::mem::offset_of!(Self, #ident) + __base__;
                <#ty as #bundle_>::write_explicit(__writer__, __offset__);
            }
        }
    });

    let write_required_calls = field_access.iter().map(|(_, ty)| {
        quote! {
            unsafe {
                <#ty as #bundle_>::write_required(__writer__);
            }
        }
    });

    quote! {
        #[expect(unsafe_code, reason = "bundle implementation is unsafe.")]
        unsafe impl #impl_generics #bundle_ for #type_ident #ty_generics #where_clause {
            fn collect_components(__collector__: &mut #component_collector_) {
                #(#collect_calls)*
            }

            unsafe fn write_explicit(__writer__: &mut #component_writer_, __base__: usize) {
                #(#write_explicit_calls)*
            }

            unsafe fn write_required(__writer__: &mut #component_writer_) {
                #(#write_required_calls)*
            }
        }
    }
    .into()
}

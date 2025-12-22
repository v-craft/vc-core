use proc_macro2::Span;
use quote::{ToTokens, quote};
use syn::{Field, Ident};

use super::{FieldAttributes, ReflectMeta};

// -----------------------------------------------------------------------------
// Define

pub(crate) struct ReflectStruct<'a> {
    meta: ReflectMeta<'a>,
    fields: Vec<StructField<'a>>,
}

pub(crate) struct StructField<'a> {
    pub data: &'a Field,
    pub attrs: FieldAttributes,
    pub declaration_index: usize,
    /// This index accounts for the removal of [ignored] fields.
    pub reflection_index: Option<usize>,
}

// -----------------------------------------------------------------------------
// Field Implementation

impl<'a> StructField<'a> {
    /// Generates a `TokenStream` for `NamedField` or `UnnamedField` construction.
    ///
    /// This function is only allowed to be called for active fields(self.reflection_index is some).
    pub fn to_info_tokens(&self, vc_reflect_path: &syn::Path) -> proc_macro2::TokenStream {
        let field_info = if self.data.ident.is_some() {
            crate::path::named_field_(vc_reflect_path) // String Literal
        } else {
            crate::path::unnamed_field_(vc_reflect_path) // Num Literal
        };

        let name = match &self.data.ident {
            Some(ident) => ident.to_string().to_token_stream(), // String Literal
            None => match self.reflection_index {
                Some(index) => index.to_token_stream(),
                None => unreachable!(
                    "`StructField::to_info_tokens` is only allowed to be called for active fields."
                ),
            },
        };

        let ty = &self.data.ty;

        // See [`CustomAttributes::get_expression_with`]
        let with_custom_attributes = self
            .attrs
            .custom_attributes
            .get_expression_with(vc_reflect_path);
        // See [`ReflectDocs::get_expression_with`]
        // If feature is diabled, this function will return a empty TokenStream, so it's safe.
        let with_docs = self.attrs.docs.get_expression_with();

        quote! {
            #field_info::new::<#ty>(#name)
                #with_custom_attributes
                #with_docs
        }
    }

    /// Returns a token stream for generating a `FieldId` for this field.
    pub fn field_id(&self, vc_reflect_path: &syn::Path) -> proc_macro2::TokenStream {
        let macro_utils_ = crate::path::macro_utils_(vc_reflect_path);
        match &self.data.ident {
            Some(ident) => {
                let name = ident.to_string();
                quote!(#macro_utils_::Cow::Borrowed(#name))
            }
            None => {
                let index = self.declaration_index.to_string();
                quote!(#macro_utils_::Cow::Borrowed(#index))
            }
        }
    }

    /// Generates a [`Member`] based on this field.
    ///
    /// If the field is unnamed, the declaration index is used.
    /// This allows this member to be used for both active and ignored fields.
    pub fn to_member(&self) -> syn::Member {
        match &self.data.ident {
            Some(ident) => syn::Member::Named(ident.clone()),
            None => syn::Member::Unnamed(self.declaration_index.into()),
        }
    }

    /// Get the field name for the `field` function of `Struct/TupleStruct`.
    ///
    /// - Named fields return values similar to `"name"`.
    /// - Unnamedfields return values similar to `2`.
    pub fn reflect_accessor(&self) -> proc_macro2::TokenStream {
        if self.attrs.ignore.is_some() {
            panic!("Non-active fields cannot obtain reflect_accessor.");
        }
        match &self.data.ident {
            Some(ident) => ident.to_string().to_token_stream(),
            None => self.reflection_index.to_token_stream(),
        }
    }
}

// -----------------------------------------------------------------------------
// Struct Implementation

impl<'a> ReflectStruct<'a> {
    pub fn new(meta: ReflectMeta<'a>, fields: Vec<StructField<'a>>) -> Self {
        let mut val = Self { meta, fields };

        let active_types = val.active_fields().map(|f| f.data.ty.clone()).collect();

        val.meta.set_active_types(active_types);
        val
    }

    /// Access the metadata associated with this struct definition.
    #[inline]
    pub fn meta(&self) -> &ReflectMeta<'a> {
        &self.meta
    }

    /// The complete set of fields in this struct.
    #[inline]
    pub fn fields(&self) -> &[StructField<'a>] {
        &self.fields
    }

    /// Get an iterator of fields which are exposed to the reflection API.
    pub fn active_fields(&self) -> impl Iterator<Item = &StructField<'a>> {
        self.fields()
            .iter()
            .filter(|field| field.attrs.ignore.is_none())
    }

    pub fn to_info_tokens(&self, is_tuple: bool) -> proc_macro2::TokenStream {
        let vc_reflect_path = self.meta.vc_reflect_path();

        let type_info_path = crate::path::type_info_(vc_reflect_path);

        let type_info_kind = if is_tuple {
            Ident::new("TupleStruct", Span::call_site())
        } else {
            Ident::new("Struct", Span::call_site())
        };

        let info_struct_path = if is_tuple {
            crate::path::tuple_struct_info_(vc_reflect_path)
        } else {
            crate::path::struct_info_(vc_reflect_path)
        };

        let field_infos = self
            .active_fields()
            .map(|field| field.to_info_tokens(vc_reflect_path));

        // See [`CustomAttributes::get_expression_with`]
        let with_custom_attributes = self.meta.with_custom_attributes_expression();
        // See [`ReflectDocs::get_expression_with`]
        // If feature is diabled, this function will return a empty TokenStream, so it's safe.
        let with_docs = self.meta.with_docs_expression();
        // See [`ReflectMeta::with_generics_expression`]
        let with_generics = self.meta.with_generics_expression();

        quote! {
            #type_info_path::#type_info_kind(
                #info_struct_path::new::<Self>(&[ #(#field_infos),* ])
                    #with_generics
                    #with_custom_attributes
                    #with_docs
            )
        }
    }
}

// -----------------------------------------------------------------------------
// Field Accessor

/// A helper struct for creating field accessors.
pub(crate) struct FieldAccessors {
    /// The referenced field accessors, such as `&self.foo`.
    pub fields_ref: Vec<proc_macro2::TokenStream>,
    /// The mutably referenced field accessors, such as `&mut self.foo`.
    pub fields_mut: Vec<proc_macro2::TokenStream>,
    /// The ordered set of field indices (basically just the range of [0, `field_count`).
    pub field_indices: Vec<usize>,
    /// The number of fields in the reflected struct.
    pub field_count: usize,
}

impl FieldAccessors {
    pub fn new(info: &ReflectStruct) -> Self {
        let (fields_ref, fields_mut): (Vec<_>, Vec<_>) = info
            .active_fields()
            .map(|field| {
                let member = field.to_member();
                (quote!(&self.#member), quote!(&mut self.#member))
            })
            .unzip();

        let field_count = fields_ref.len();
        let field_indices = (0..field_count).collect();

        Self {
            fields_ref,
            fields_mut,
            field_indices,
            field_count,
        }
    }
}

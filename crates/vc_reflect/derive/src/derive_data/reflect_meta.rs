use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Generics, ImplGenerics, Path, Token, Type, TypeGenerics, punctuated::Punctuated};

use super::{TypeAttributes, TypeParser};
use crate::utils::StringExpr;

pub(crate) struct ReflectMeta<'a> {
    vc_reflect_path: Path,
    attrs: TypeAttributes,
    type_parser: TypeParser<'a>,
    // cannot use `BTreeSet` becausee `syn::Type` does not impl `Ord`.
    active_types: HashSet<Type, FixedState>,
}

impl core::fmt::Debug for ReflectMeta<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ReflectMeta")
            .field("vc_reflect_path", &self.vc_reflect_path.to_token_stream())
            .field("type_parser", &self.type_parser)
            .field("attrs", &self.attrs)
            .finish()
    }
}

impl<'a> ReflectMeta<'a> {
    #[inline]
    pub fn new(attrs: TypeAttributes, type_parser: TypeParser<'a>) -> Self {
        Self {
            attrs,
            type_parser,
            vc_reflect_path: crate::path::vc_reflect(),
            active_types: HashSet::default(),
        }
    }

    /// Used for [`ReflectStruct`](crate::derive_data::ReflectStruct) and [`ReflectEnum`](crate::derive_data::ReflectEnum),
    /// set the active field type during initialization.
    #[inline]
    pub(super) fn set_active_types(&mut self, active_types: HashSet<Type, FixedState>) {
        self.active_types = active_types;
    }

    #[inline]
    pub fn vc_reflect_path(&self) -> &Path {
        &self.vc_reflect_path
    }

    #[inline]
    pub fn attrs(&self) -> &TypeAttributes {
        &self.attrs
    }

    /// Generate docs codes
    ///
    /// Similar to following:
    ///
    /// ```ignore
    /// .with_docs(::core::option::Option::Some("......"))
    /// ```
    #[inline]
    pub fn with_docs_expression(&self) -> TokenStream {
        self.attrs.docs.get_expression_with()
    }

    /// Generate custom attibutes codes
    ///
    /// Similar to following:
    ///
    /// ```ignore
    /// .with_custom_attributes(
    ///     _path_::CustomAttributes::new()
    ///         (.with_attribute( ... ))*
    /// )
    /// ```
    #[inline]
    pub fn with_custom_attributes_expression(&self) -> TokenStream {
        self.attrs
            .custom_attributes
            .get_expression_with(&self.vc_reflect_path)
    }

    /// Generate generics codes
    ///
    /// Similar to following:
    ///
    /// ```ignore
    /// .with_generics(
    ///     _path_::Generics::from([
    ///         _path_::GenericsInfo::Type(_path_::TypeParamInfo::new::<_>(..)),
    ///         _path_::GenericsInfo::Const(....),
    ///         ......
    ///     ])
    /// )
    /// ```
    pub fn with_generics_expression(&self) -> TokenStream {
        let vc_reflect_path = &self.vc_reflect_path;
        let generics_ = crate::path::generics_(vc_reflect_path);
        let generic_info_ = crate::path::generic_info_(vc_reflect_path);
        let type_param_info_ = crate::path::type_param_info_(vc_reflect_path);
        let const_param_info_ = crate::path::const_param_info_(vc_reflect_path);

        let generics = self
            .generics()
            .params
            .iter()
            .filter_map(|param| match param {
                syn::GenericParam::Lifetime(_) => None,
                syn::GenericParam::Type(type_param) => {
                    let ident = &type_param.ident;
                    let name = ident.to_string();
                    let with_default = type_param
                        .default
                        .as_ref()
                        .map(|default_ty| quote!(.with_default::<#default_ty>()));

                    Some(quote! {
                        #generic_info_::Type(
                            #type_param_info_::new::<#ident>(
                                #name
                            )
                            #with_default
                        )
                    })
                }
                syn::GenericParam::Const(const_param) => {
                    let ty = &const_param.ty;
                    let ident = &const_param.ident;
                    let name = const_param.ident.to_string();

                    Some(quote! {
                        #generic_info_::Const(
                            #const_param_info_::new::<#ty>(
                                #name, #ident
                            )
                        )
                    })
                }
            })
            .collect::<Punctuated<_, Token![,]>>();

        if generics.is_empty() {
            return crate::utils::empty();
        }

        quote! {
            .with_generics(
                #generics_::from([ #generics ])
            )
        }
    }

    #[inline]
    pub fn generics(&self) -> &'a Generics {
        self.type_parser.generics()
    }

    #[inline]
    pub fn impl_with_generic(&self) -> bool {
        self.type_parser.impl_with_generic()
    }

    #[inline]
    pub fn real_ident(&self) -> TokenStream {
        self.type_parser.real_ident()
    }

    // #[inline]
    // pub fn crate_name(&self) -> Option<StringExpr> {
    //     self.type_parser.crate_name()
    // }

    #[inline]
    pub fn module_path(&self) -> Option<StringExpr> {
        self.type_parser.module_path()
    }

    #[inline]
    pub fn type_ident(&self) -> StringExpr {
        self.type_parser.type_ident()
    }

    #[inline]
    pub fn assert_ident_tokens(&self) -> TokenStream {
        #[cfg(debug_assertions)]
        if let TypeParser::Primitive(_) = &self.type_parser {
            let ident = self.real_ident();
            return quote! {
                mod __assert_primitive_ident {
                    type AssertIdentValidity = #ident;
                }
            };
        }
        crate::utils::empty()
    }

    #[inline]
    pub fn type_name(&self) -> StringExpr {
        self.type_parser.type_name(&self.vc_reflect_path)
    }

    #[inline]
    pub fn type_path(&self) -> StringExpr {
        self.type_parser.type_path(&self.vc_reflect_path)
    }

    #[inline]
    pub fn type_name_into_owned(&self) -> TokenStream {
        self.type_parser
            .type_name(&self.vc_reflect_path)
            .into_owned(&self.vc_reflect_path)
    }

    #[inline]
    pub fn type_path_into_owned(&self) -> TokenStream {
        self.type_parser
            .type_path(&self.vc_reflect_path)
            .into_owned(&self.vc_reflect_path)
    }

    /// Return the required generic parameters.
    ///
    /// The three parameters returned are `impl_generics`, `ty_generics`, `where_clause`.
    ///
    /// ## Example
    ///
    /// ```ignore
    /// let real_ident = meta.real_ident();
    /// let (impl_generics, ty_generics, where_clause) =
    ///     meta.split_generics(true, false, true);
    ///
    /// quote! {
    ///     impl #impl_generics TraitName for #real_ident #ty_generics  #where_clause {
    ///         /* ... */
    ///     }
    /// }
    /// ```
    ///
    /// We need to add many constraints in the 'where' block (for **generic** types), the situation here is quite complex,
    ///
    /// ## Type Itself
    ///
    /// For the type itself, if it has lifecycle params, needs to be labeled with `'statc`.
    /// If it has type generic params, needs to be labeled with `Any + Send + Sync`.
    ///
    /// ## TypePath
    ///
    /// When implementing `TypePath`, we only require all type parameters to implement `TypePath`.
    ///
    /// Specifically, `TypePath` is the most fundamental trait, and in order to ensure that the constraints of other traits are correct,
    /// they also need to meet this requirement (unless `TypePath` is implemented on their own).
    ///
    /// ## Typed
    ///
    /// When implementing Typed, it is necessary to handle generic parameter information.
    /// The `new` function for generic information requires that the generic implement TypePath!!!!
    ///
    /// Therefore, as long as either `TypePath` or `Typed` uses auto-implementation,
    /// a `TypePath` constraint will be added to all type generic parameters in every traits' implementations.
    ///
    /// Additionally, since `XxxInfo:: new::<T>` requires T to implement `Reflect`,
    /// The constraints of `Typed` have to be consistent with `Reflect`.
    ///
    /// Therefore, all **field** type(has type generic params) require `Typed + Reflect`.
    ///
    /// ## Reflect
    ///
    /// Consistent with Typed.
    ///
    /// ## FromReflect
    ///
    /// Obviously, if there are fields, it is required that all fields implement `FromReflect`.
    /// But we only need to explicitly constrain fields with generic parameters.
    ///
    /// ## GetTypeMeta
    ///
    /// Obviously, if there are fields, it is required that all fields implement `GetTypeMeta`.
    /// But we only need to explicitly constrain fields with generic parameters.
    ///
    /// Specifically, if `FromReflect` is enabled,
    /// we need to insert `TypeTraitFromReflect`, so the constraints must also be consistent with `FromReflect`.
    /// (Required that all fields implement `FromReflect`)
    ///
    /// ## Summary
    ///
    /// - Type Itself:
    ///     - `'staitc`: exists lifetimes.
    ///     - `Any + Sync + Send`: exists type params.
    /// - Type Params:
    ///     - `TypePath`: as long as one of `Typed` and `TypePath` is enabled.
    /// - Field Type(witl type param):
    ///     - `Typed + Reflect`: all implementations except for `TypePath` trait.
    ///     - `GetTypeMeta`: only `GetTypeMeta`.
    ///     - `FromReflect`: `FromReflect` and `GetTypeMeta`, if `FromReflect` is enabled.
    ///
    /// Therefore, we need three function parameters to control them.
    ///
    /// ## Special enumeration implementation.
    ///
    /// Due to the implementation of enumeration `Reflect::apply` relying on its own `FromReflect`,
    /// All implementations except for `TypePath` require `FromReflect` constraint for field type.
    ///
    /// But if the automatic implementation of `FromReflect` is turned off, there is no need for it at this time.
    /// (`Reflect` will also be adjusted.)
    pub fn split_generics(
        &self,
        add_reflect_typed: bool,
        add_get_type_meta: bool,
        add_from_reflect: bool,
    ) -> (ImplGenerics<'_>, TypeGenerics<'_>, TokenStream) {
        use crate::path::fp::{AnyFP, SendFP, SyncFP};

        let add_type_path =
            self.attrs().impl_switchs.impl_type_path || self.attrs().impl_switchs.impl_typed;
        let add_from_reflect = add_from_reflect && self.attrs().impl_switchs.impl_from_reflect;

        let generics = self.generics();

        let mut generic_where_clause = quote! { where };

        if generics.type_params().next().is_some() {
            generic_where_clause.extend(quote! { Self: #AnyFP + #SendFP + #SyncFP, });
        } else if generics.lifetimes().next().is_some() {
            generic_where_clause.extend(quote! { Self: 'static, });
        }

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        // Maintain existing where clause bounds, if any.
        if let Some(where_clause) = where_clause {
            let predicates = where_clause.predicates.iter();
            generic_where_clause.extend(quote! { #(#predicates,)* });
        }

        let mut predicates: Punctuated<TokenStream, Token![,]> = Punctuated::new();

        if add_type_path {
            predicates.extend(self.type_path_predicates());
        }

        if add_reflect_typed {
            let p = self.field_type_predicates(add_get_type_meta, add_from_reflect);
            if let Some(p) = p {
                predicates.extend(p);
            }
        }

        generic_where_clause.extend(quote! { #predicates });

        (impl_generics, ty_generics, generic_where_clause)
    }

    fn type_path_predicates(&self) -> impl Iterator<Item = TokenStream> + '_ {
        let type_path_ = crate::path::type_path_(&self.vc_reflect_path);
        self.generics().type_params().map(move |param| {
            let ident = &param.ident;
            quote!(#ident : #type_path_)
        })
    }

    fn field_type_predicates(
        &self,
        add_get_type_meta: bool,
        add_from_reflect: bool,
    ) -> Option<impl Iterator<Item = TokenStream> + '_> {
        if self.active_types.is_empty() {
            return None;
        }

        let type_param_idents = self
            .generics()
            .type_params()
            .map(|type_param| type_param.ident.clone())
            .collect::<Vec<syn::Ident>>();

        if type_param_idents.is_empty() {
            return None;
        }

        let vc_reflect_path = &self.vc_reflect_path;
        let reflect_ = crate::path::reflect_(vc_reflect_path);
        let typed_ = crate::path::typed_(vc_reflect_path);

        let get_type_meta_ = if add_get_type_meta {
            let get_type_meta_ = crate::path::get_type_meta_(vc_reflect_path);
            quote!( + #get_type_meta_ )
        } else {
            crate::utils::empty()
        };

        let add_from_reflect_ = if add_from_reflect {
            let from_reflect_ = crate::path::from_reflect_(vc_reflect_path);
            quote!( + #from_reflect_ )
        } else {
            crate::utils::empty()
        };

        // Do any of the identifiers in `idents` appear in `token_stream`?
        fn is_any_ident_in_token_stream(idents: &[syn::Ident], token_stream: TokenStream) -> bool {
            for token_tree in token_stream {
                match token_tree {
                    proc_macro2::TokenTree::Ident(ident) => {
                        if idents.contains(&ident) {
                            return true;
                        }
                    }
                    proc_macro2::TokenTree::Group(group) => {
                        if is_any_ident_in_token_stream(idents, group.stream()) {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
            false
        }

        Some(self.active_types.iter().filter_map(move |ty| {
            if is_any_ident_in_token_stream(&type_param_idents, ty.to_token_stream()) {
                Some(quote! {
                    #ty: #reflect_ + #typed_ #add_from_reflect_ # get_type_meta_
                })
            } else {
                None
            }
        }))
    }

    /// For Opaque Type
    pub fn to_info_tokens(&self) -> TokenStream {
        let vc_reflect_path = &self.vc_reflect_path;

        let opaque_info_ = crate::path::opaque_info_(vc_reflect_path);
        let type_info_ = crate::path::type_info_(vc_reflect_path);
        let with_custom_attributes = self.with_custom_attributes_expression();
        let with_docs = self.with_docs_expression();
        let with_generics = self.with_generics_expression();

        quote! {
            #type_info_::Opaque(
                #opaque_info_::new::<Self>()
                    #with_custom_attributes
                    #with_generics
                    #with_docs
            )
        }
    }
}

pub(crate) struct FixedHasher(u64);

impl core::hash::Hasher for FixedHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.0 = self.0.rotate_right(8).wrapping_add(*b as u64)
        }
        for b in bytes {
            self.0 = self.0.rotate_right(7).wrapping_add((*b % 41) as u64)
        }
    }
}

/// A simple fixed hash state, used to ensure that
/// `where` expression generated multiple times is consistent by the same `ReflectMeta`(multiple compilations).
///
/// There is no need to worry about performance issues in this scenario.
/// The number of fields in a structure is usually very small.
/// And usually no behavior will cause hash conflicts.
#[derive(Copy, Clone, Default)]
pub(super) struct FixedState;

impl core::hash::BuildHasher for FixedState {
    type Hasher = FixedHasher;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        FixedHasher(0)
    }
}

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, Path, Token, parse::ParseStream};

/// A container for custom attribute expressions.
///
/// This corresponds to `vc_reflect::info::CustomAttributes`.
#[derive(Default, Debug)]
pub(crate) struct CustomAttributes {
    attributes: Vec<Expr>,
}

impl CustomAttributes {
    /// Inserts a custom attribute into the list.
    pub fn push(&mut self, value: Expr) -> syn::Result<()> {
        self.attributes.push(value);
        Ok(())
    }

    /// Parse `@` attribute.
    ///
    /// Examples:
    /// - `#[reflect(@Foo))]`
    /// - `#[reflect(@Bar::baz("qux"))]`
    /// - `#[reflect(@0..256u8)]`
    pub fn parse_inner_stream(&mut self, input: ParseStream) -> syn::Result<()> {
        input.parse::<Token![@]>()?;
        self.push(input.parse()?)
    }

    /// If `custom_attributes` is empty, this function will return an empty token stream.
    ///
    /// Otherwise, it will return content similar to this:
    ///
    /// ```ignore
    /// .with_custom_attributes(
    ///     _path_::CustomAttributes::new()
    ///         (.with_attribute( ... ))*
    /// )
    /// ```
    ///
    /// The type path will be parsed before returning.
    pub fn get_expression_with(&self, vc_reflect_path: &Path) -> TokenStream {
        if self.attributes.is_empty() {
            return crate::utils::empty();
        }

        let capacity = self.attributes.len();

        let with_attributes = self.attributes.iter().map(|value| {
            quote! {
                .with_attribute(#value)
            }
        });

        let custom_attributes_ = crate::path::custom_attributes_(vc_reflect_path);

        quote! {
            .with_custom_attributes(
                #custom_attributes_::with_capacity(#capacity)
                    #(#with_attributes)*
            )
        }
    }
}

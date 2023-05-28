use proc_macro2::{Ident, TokenStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{parse_quote, Attribute, Error, Expr, Fields, Lit, Meta, Token, Variant};

use crate::macros::enum_discriminants::get_discriminant;
use crate::Result;

pub struct ErrorVariant {
    pub name: Ident,
    pub message: Option<TokenStream>,
    pub attributes: Vec<Attribute>,
    pub fields: Fields,
    pub code: Option<u32>,
    pub deprecated: bool,
}

impl ErrorVariant {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new instance of the ErrorAttributes struct from the given attributes.
    pub fn from(mut variant: Variant) -> Result<ErrorVariant> {
        let code = get_discriminant(&variant)?;
        variant
            .attrs
            .retain(|attr| !attr.path().is_ident("discriminant"));

        let mut error_variant = ErrorVariant {
            name: variant.ident,
            message: None,
            attributes: variant.attrs,
            fields: variant.fields,
            code,
            deprecated: false,
        };

        error_variant.parse_attributes()?;

        Ok(error_variant)
    }

    /// Parses the attributes of a variant.
    fn parse_attributes(&mut self) -> Result<()> {
        let mut index = 0;
        while index < self.attributes.len() {
            let attribute = &self.attributes[index];
            let attribute_path = attribute.path();

            if attribute_path.is_ident("msg") {
                let attribute = self.attributes.remove(index);
                let attribute_span = attribute.span();

                if self.message.is_some() {
                    return Err(Error::new(
                        attribute_span,
                        "The msg attribute can only be used once",
                    ));
                }

                let args = match &attribute.meta {
                    Meta::List(v) => &v.tokens,
                    _ => {
                        return Err(Error::new(
                            attribute_span,
                            "The msg attribute expects arguments following the format of the format! macro",
                        ));
                    }
                };

                // Check first argument is a literal string.
                let expr_list: Punctuated<Expr, Token![,]> = parse_quote! { #args };

                match expr_list.first() {
                    Some(Expr::Lit(v)) => match &v.lit {
                        Lit::Str(_) => {}
                        v => {
                            return Err(Error::new(v.span(), "This must be a literal string"));
                        }
                    },
                    _ => {
                        return Err(Error::new(
                            expr_list.span(),
                            "First attribute must be a literal string",
                        ));
                    }
                }

                self.message = Some(args.clone());
            } else if attribute_path.is_ident("deprecated") {
                let attribute_span = attribute.span();

                if self.deprecated {
                    return Err(Error::new(
                        attribute_span,
                        "The deprecated attribute can only be used once",
                    ));
                }

                self.deprecated = true;
                index += 1;
            } else {
                index += 1;
            }
        }

        Ok(())
    }
}

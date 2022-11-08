use proc_macro2::Ident;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, Attribute, Error, Fields, Lit, LitInt, MetaList, NestedMeta, Token, Variant,
};

use crate::Result;

pub struct ErrorVariant {
    pub name: Ident,
    pub message: Option<Punctuated<NestedMeta, Token![,]>>,
    pub attributes: Vec<Attribute>,
    pub fields: Fields,
    pub code: Option<LitInt>,
    pub deprecated: bool,
}

impl ErrorVariant {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new instance of the ErrorAttributes struct from the given attributes.
    pub fn from(variant: Variant) -> Result<ErrorVariant> {
        if let Some((_, discriminant)) = variant.discriminant {
            return Err(Error::new(
                discriminant.span(),
                "Discriminants are not supported, please use the `#[code]` attribute instead.",
            ));
        }

        let mut error_variant = ErrorVariant {
            name: variant.ident,
            message: None,
            attributes: variant.attrs,
            fields: variant.fields,
            code: None,
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

            if attribute.path.is_ident("msg") {
                let attribute = self.attributes.remove(index);
                let attribute_span = attribute.span();

                if self.message.is_some() {
                    return Err(Error::new(
                        attribute_span,
                        "The msg attribute can only be used once",
                    ));
                }

                let path = &attribute.path;
                let tokens = &attribute.tokens;
                let tokens = quote! {#path #tokens};

                let args = match parse_macro_input::parse::<MetaList>(tokens.into()) {
                    Ok(v) => v,
                    Err(_) => {
                        return Err(Error::new(
                            attribute_span,
                            "The msg attribute expects arguments following the format of the format! macro",
                        ));
                    }
                };

                // Check first argument is a literal string.
                let first_argument = args.nested.first().unwrap();
                match first_argument {
                    NestedMeta::Lit(v) => match v {
                        Lit::Str(_) => {}
                        v => {
                            return Err(Error::new(v.span(), "This must be a literal string"));
                        }
                    },
                    NestedMeta::Meta(v) => {
                        return Err(Error::new(v.span(), "This must be a literal string"));
                    }
                }

                self.message = Some(args.nested);
            } else if attribute.path.is_ident("code") {
                let attribute = self.attributes.remove(index);
                let attribute_span = attribute.span();

                if self.code.is_some() {
                    return Err(Error::new(
                        attribute_span,
                        "The code attribute can only be used once",
                    ));
                }

                let path = &attribute.path;
                let tokens = &attribute.tokens;
                let tokens = quote! {#path #tokens};

                let args = match parse_macro_input::parse::<MetaList>(tokens.into()) {
                    Ok(v) => v,
                    Err(_) => {
                        return Err(Error::new(
                            attribute_span,
                            "The code attribute expects one integer literal as arguments",
                        ));
                    }
                };

                if args.nested.len() != 1 {
                    return Err(Error::new(
                        attribute_span,
                        "The code attribute expects only one argument",
                    ));
                }

                // Check first argument is a literal string.
                let first_argument = args.nested.first().unwrap();
                match first_argument {
                    NestedMeta::Lit(v) => match v {
                        Lit::Int(v) => {
                            self.code = Some(v.clone());
                        }
                        v => {
                            return Err(Error::new(v.span(), "This must be a literal string"));
                        }
                    },
                    NestedMeta::Meta(v) => {
                        return Err(Error::new(v.span(), "This must be a literal string"));
                    }
                }
            } else if attribute.path.is_ident("deprecated") {
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

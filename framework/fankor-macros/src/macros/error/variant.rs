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
    pub continue_from: Option<LitInt>,
    pub deprecated: bool,
}

impl ErrorVariant {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new instance of the ErrorAttributes struct from the given attributes.
    pub fn from(variant: Variant) -> Result<ErrorVariant> {
        if let Some((_, discriminant)) = variant.discriminant {
            return Err(Error::new(
                discriminant.span(),
                "Discriminants are not supported, please use the `#[continue_from]` attribute instead.",
            ));
        }

        let mut error_variant = ErrorVariant {
            name: variant.ident,
            message: None,
            attributes: variant.attrs,
            fields: variant.fields,
            continue_from: None,
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
            } else if attribute.path.is_ident("continue_from") {
                let attribute = self.attributes.remove(index);
                let attribute_span = attribute.span();

                if self.continue_from.is_some() {
                    return Err(Error::new(
                        attribute_span,
                        "The continue_from attribute can only be used once",
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
                            "The continue_from attribute expects one integer literal as arguments",
                        ));
                    }
                };

                if args.nested.len() != 1 {
                    return Err(Error::new(
                        attribute_span,
                        "The continue_from attribute expects only one argument",
                    ));
                }

                // Check first argument is a literal string.
                let first_argument = args.nested.first().unwrap();
                match first_argument {
                    NestedMeta::Lit(v) => match v {
                        Lit::Int(v) => {
                            self.continue_from = Some(v.clone());
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

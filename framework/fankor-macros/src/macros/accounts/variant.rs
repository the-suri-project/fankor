use proc_macro2::Ident;
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Attribute, Error, Lit, LitInt, MetaList, NestedMeta, Variant};

use crate::Result;

pub struct AccountVariant {
    pub name: Ident,
    pub attributes: Vec<Attribute>,
    pub discriminant: Option<LitInt>,
    pub deprecated: bool,
}

impl AccountVariant {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new instance of the ErrorAttributes struct from the given attributes.
    pub fn from(variant: Variant) -> Result<AccountVariant> {
        if let Some((_, discriminant)) = variant.discriminant {
            return Err(Error::new(
                discriminant.span(),
                "Discriminants are not supported",
            ));
        }

        if !variant.fields.is_empty() {
            return Err(Error::new(variant.span(), "Fields are not supported"));
        }

        let mut account_variant = AccountVariant {
            name: variant.ident,
            attributes: variant.attrs,
            discriminant: None,
            deprecated: false,
        };

        account_variant.parse_attributes()?;

        Ok(account_variant)
    }

    /// Parses the attributes of a variant.
    fn parse_attributes(&mut self) -> Result<()> {
        let mut index = 0;
        while index < self.attributes.len() {
            let attribute = &self.attributes[index];

            if attribute.path.is_ident("discriminant") {
                let attribute = self.attributes.remove(index);
                let attribute_span = attribute.span();

                if self.discriminant.is_some() {
                    return Err(Error::new(
                        attribute_span,
                        "The discriminant attribute can only be used once",
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
                            "The discriminant attribute expects one integer literal as arguments",
                        ));
                    }
                };

                if args.nested.len() != 1 {
                    return Err(Error::new(
                        attribute_span,
                        "The discriminant attribute expects only one argument",
                    ));
                }

                // Check first argument is a literal string.
                let first_argument = args.nested.first().unwrap();
                match first_argument {
                    NestedMeta::Lit(v) => match v {
                        Lit::Int(v) => {
                            self.discriminant = Some(v.clone());
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

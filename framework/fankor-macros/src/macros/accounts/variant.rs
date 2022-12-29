use crate::macros::serialize::get_discriminant;
use proc_macro2::Ident;
use syn::spanned::Spanned;
use syn::{Attribute, Error, Variant};

use crate::Result;

pub struct AccountVariant {
    pub name: Ident,
    pub attributes: Vec<Attribute>,
    pub discriminant: Option<u8>,
    pub deprecated: bool,
}

impl AccountVariant {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new instance of the ErrorAttributes struct from the given attributes.
    pub fn from(variant: Variant) -> Result<AccountVariant> {
        let discriminant = get_discriminant(&variant)?;

        if !variant.fields.is_empty() {
            return Err(Error::new(variant.span(), "Fields are not supported"));
        }

        let mut account_variant = AccountVariant {
            name: variant.ident,
            attributes: variant.attrs,
            discriminant,
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

            if attribute.path.is_ident("deprecated") {
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

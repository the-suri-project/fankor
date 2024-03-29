use proc_macro2::Ident;
use syn::{Attribute, Error, Variant};
use syn::spanned::Spanned;

use crate::Result;

pub struct AccountVariant {
    pub name: Ident,
    pub attributes: Vec<Attribute>,
}

impl AccountVariant {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new instance of the ErrorAttributes struct from the given attributes.
    pub fn from(variant: Variant) -> Result<AccountVariant> {
        if variant.discriminant.is_some() {
            return Err(Error::new(
                variant.span(),
                "Native discriminants not yet supported in BPF compiler",
            ));
        }

        if !variant.fields.is_empty() {
            return Err(Error::new(variant.span(), "Fields are not supported"));
        }

        let account_variant = AccountVariant {
            name: variant.ident,
            attributes: variant.attrs,
        };

        Ok(account_variant)
    }
}

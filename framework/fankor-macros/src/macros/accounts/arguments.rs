use proc_macro2::Ident;
use quote::{format_ident, ToTokens};
use syn::{spanned::Spanned, AttributeArgs, Error, ItemEnum, Meta, NestedMeta};

use crate::utils::unwrap_string_from_literal;
use crate::Result;

pub struct AccountsArguments {
    /// The main accounts type name. This helps to reuse the discriminants
    /// of the main accounts enum.
    pub accounts_type_name: Option<Ident>,

    /// List of attributes to apply to the enum.
    pub attrs: Vec<syn::Attribute>,
}

impl AccountsArguments {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new instance of the AccountsArguments struct from the given attributes.
    pub fn from(args: AttributeArgs, enum_item: &ItemEnum) -> Result<AccountsArguments> {
        let mut result = AccountsArguments {
            accounts_type_name: None,
            attrs: Vec::new(),
        };

        match args.len() {
            0 => {}
            1 => {
                result.accounts_type_name = match &args[0] {
                    NestedMeta::Meta(v) => match v {
                        Meta::NameValue(meta) => {
                            if meta.path.is_ident("base") {
                                let literal = unwrap_string_from_literal(meta.lit.clone())?;
                                Some(format_ident!("{}", literal.value(), span = literal.span()))
                            } else {
                                return Err(Error::new(
                                    meta.path.span(),
                                    format!("Unknown attribute: {}", meta.path.to_token_stream()),
                                ));
                            }
                        }
                        Meta::List(v) => return Err(Error::new(v.span(), "Unknown argument")),
                        Meta::Path(v) => return Err(Error::new(v.span(), "Unknown argument")),
                    },
                    NestedMeta::Lit(v) => {
                        return Err(Error::new(v.span(), "Unknown argument"));
                    }
                };
            }
            _ => {
                return Err(Error::new(
                    enum_item.span(),
                    "Accounts macro expects only one argument, the accounts type name",
                ))
            }
        }

        Ok(result)
    }
}

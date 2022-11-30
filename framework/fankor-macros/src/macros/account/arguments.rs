use crate::utils::unwrap_string_from_literal;
use crate::Result;
use proc_macro2::Ident;
use proc_macro2::Span;
use quote::{format_ident, ToTokens};
use syn::{spanned::Spanned, AttributeArgs, Error, Meta, NestedMeta};

pub struct AccountArguments {
    /// The accounts type name.
    pub accounts_type_name: Ident,
}

impl AccountArguments {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new instance of the AccountArguments struct from the given attributes.
    pub fn from(args: AttributeArgs, span: Span) -> Result<AccountArguments> {
        if args.is_empty() {
            return Err(Error::new(
                span,
                "Account macro expects the accounts type name as an argument",
            ));
        }

        if args.len() > 1 {
            return Err(Error::new(
                span,
                "Account macro expects only one argument, the accounts type name",
            ));
        }

        let accounts_type_name = match &args[0] {
            NestedMeta::Meta(v) => match v {
                Meta::NameValue(meta) => {
                    if meta.path.is_ident("base") {
                        let lit = unwrap_string_from_literal(meta.lit.clone())?;
                        format_ident!("{}", lit.value(), span = lit.span())
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

        Ok(AccountArguments { accounts_type_name })
    }
}

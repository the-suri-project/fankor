use crate::Result;
use proc_macro2::Span;
use syn::{spanned::Spanned, AttributeArgs, Error, Ident, Meta, NestedMeta};

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
                Meta::NameValue(v) => return Err(Error::new(v.span(), "Unknown argument")),
                Meta::List(v) => return Err(Error::new(v.span(), "Unknown argument")),
                Meta::Path(v) => match v.get_ident() {
                    Some(v) => v.clone(),
                    None => return Err(Error::new(v.span(), "Only simple types are supported")),
                },
            },
            NestedMeta::Lit(v) => {
                return Err(Error::new(v.span(), "Unknown argument"));
            }
        };

        Ok(AccountArguments { accounts_type_name })
    }
}

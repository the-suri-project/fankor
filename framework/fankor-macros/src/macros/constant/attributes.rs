use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{AttributeArgs, Error, LitStr, Meta, NestedMeta};

use fankor_syn::expressions::unwrap_string_from_literal;

use crate::Result;

pub struct ConstantAttributes {
    /// The name of the constant. If missing uses the Rust identifier.
    pub alias: Option<LitStr>,
}

impl ConstantAttributes {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new instance of the ConstantAttributes struct from the given attributes.
    pub fn from(args: AttributeArgs) -> Result<ConstantAttributes> {
        let mut result = ConstantAttributes { alias: None };

        for arg in args {
            match arg {
                NestedMeta::Meta(v) => match v {
                    Meta::NameValue(meta) => {
                        if meta.path.is_ident("alias") {
                            result.alias = Some(unwrap_string_from_literal(meta.lit)?);
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
            }
        }

        Ok(result)
    }
}

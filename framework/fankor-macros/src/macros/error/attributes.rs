use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{AttributeArgs, Error, LitInt, Meta, NestedMeta};

use fankor_syn::expressions::unwrap_int_from_literal;

use crate::Result;

pub struct ErrorAttributes {
    /// The starting offset of the error list.
    pub offset: Option<LitInt>,

    /// Whether the test should be skipped or not.
    pub skip_test: bool,
}

impl ErrorAttributes {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new instance of the ErrorAttributes struct from the given attributes.
    pub fn from(args: AttributeArgs) -> Result<ErrorAttributes> {
        let mut result = ErrorAttributes {
            offset: None,
            skip_test: false,
        };

        for arg in args {
            match arg {
                NestedMeta::Meta(v) => match v {
                    Meta::NameValue(meta) => {
                        if meta.path.is_ident("offset") {
                            result.offset = Some(unwrap_int_from_literal(meta.lit)?);
                        } else {
                            return Err(Error::new(
                                meta.path.span(),
                                format!("Unknown attribute: {}", meta.path.to_token_stream()),
                            ));
                        }
                    }
                    Meta::List(v) => return Err(Error::new(v.span(), "Unknown argument")),
                    Meta::Path(v) => {
                        if v.is_ident("skip_test") {
                            result.skip_test = true;
                        } else {
                            return Err(Error::new(v.span(), "Unknown argument"));
                        }
                    }
                },
                NestedMeta::Lit(v) => {
                    return Err(Error::new(v.span(), "Unknown argument"));
                }
            }
        }

        Ok(result)
    }
}

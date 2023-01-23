use quote::ToTokens;
use syn::{spanned::Spanned, AttributeArgs, Error, LitInt, Meta, NestedMeta};

use crate::utils::unwrap_int_from_literal;
use crate::Result;

pub struct ErrorArguments {
    /// The starting offset of the error list.
    pub offset: Option<LitInt>,

    /// Whether to add the TsGen macro or not.
    pub skip_ts_gen: bool,

    /// List of attributes to apply to the enum.
    pub attrs: Vec<syn::Attribute>,
}

impl ErrorArguments {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new instance of the ErrorArguments struct from the given attributes.
    pub fn from(args: AttributeArgs) -> Result<ErrorArguments> {
        let mut result = ErrorArguments {
            offset: None,
            skip_ts_gen: false,
            attrs: Vec::new(),
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
                    Meta::Path(path) => {
                        if path.is_ident("skip_ts_gen") {
                            result.skip_ts_gen = true;
                        } else {
                            return Err(Error::new(
                                path.span(),
                                format!("Unknown attribute: {}", path.to_token_stream()),
                            ));
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

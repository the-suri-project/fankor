use crate::macros::instruction_accounts::parser::CustomMetaList;
use proc_macro2::Ident;
use quote::format_ident;
use syn::spanned::Spanned;
use syn::{Attribute, Error, Expr};

use crate::utils::unwrap_string_from_literal;
use crate::Result;

pub struct InstructionArguments {
    // Attributes.
    pub args: Option<Ident>,
}

impl InstructionArguments {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new instance of the Field struct from the given attributes.
    pub fn from(attributes: &[Attribute]) -> Result<InstructionArguments> {
        let mut result = InstructionArguments { args: None };

        for attribute in attributes {
            if !attribute.path.is_ident("instruction") {
                continue;
            }

            let attribute_span = attribute.span();
            let args = match attribute.parse_args::<CustomMetaList>() {
                Ok(v) => v,
                Err(_) => {
                    return Err(Error::new(
                        attribute_span,
                        "The account attribute expects arguments",
                    ));
                }
            };

            // Check each argument.
            for meta in args.list {
                let name = meta.name;
                if let Some(value) = meta.value {
                    match name.to_string().as_str() {
                        "args" => {
                            if result.args.is_some() {
                                return Err(Error::new(
                                    name.span(),
                                    "The args argument can only be defined once",
                                ));
                            }

                            match value {
                                Expr::Lit(v) => {
                                    let lit = unwrap_string_from_literal(v.lit)?;

                                    result.args =
                                        Some(format_ident!("{}", lit.value(), span = lit.span()))
                                }
                                _ => {
                                    return Err(Error::new(name.span(), "Unknown argument"));
                                }
                            }
                        }
                        _ => {
                            return Err(Error::new(name.span(), "Unknown argument"));
                        }
                    }
                } else {
                    return match name.to_string().as_str() {
                        "args" => Err(Error::new(
                            name.span(),
                            "The args argument must use a value: args = <expr>",
                        )),
                        _ => Err(Error::new(name.span(), "Unknown argument")),
                    };
                }
            }
        }

        Ok(result)
    }
}

use crate::macros::instruction_accounts::parser::CustomMetaList;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::{Attribute, Error, Expr};

use crate::Result;

pub struct InstructionArguments {
    // Attributes.
    pub args: Option<Ident>,
    pub initial_validation: Option<Validation>,
    pub final_validation: Option<Validation>,
}

pub enum Validation {
    Implicit,
    Explicit(TokenStream),
}

impl InstructionArguments {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new instance of the Field struct from the given attributes.
    pub fn from(attributes: &[Attribute]) -> Result<InstructionArguments> {
        let mut result = InstructionArguments {
            args: None,
            initial_validation: None,
            final_validation: None,
        };

        for attribute in attributes {
            if !attribute.path.is_ident("account") {
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
                                Expr::Path(v) => {
                                    if v.path.segments.len() != 1 {
                                        return Err(Error::new(
                                            name.span(),
                                            "The args argument must be a single identifier",
                                        ));
                                    }

                                    let ident = v.path.segments.first().unwrap().ident.clone();
                                    result.args = Some(ident)
                                }
                                _ => {
                                    return Err(Error::new(name.span(), "Unknown argument"));
                                }
                            }
                        }
                        "initial_validation" => {
                            result.initial_validation = Some(Validation::Explicit(quote!(#value)));
                        }
                        "final_validation" => {
                            result.final_validation = Some(Validation::Explicit(quote!(#value)));
                        }
                        _ => {
                            return Err(Error::new(name.span(), "Unknown argument"));
                        }
                    }
                } else {
                    match name.to_string().as_str() {
                        "args" => {
                            return Err(Error::new(
                                name.span(),
                                "The args argument must use a value: args = <expr>",
                            ))
                        }
                        "initial_validation" => {
                            result.initial_validation = Some(Validation::Implicit);
                        }
                        "final_validation" => {
                            result.final_validation = Some(Validation::Implicit);
                        }
                        _ => return Err(Error::new(name.span(), "Unknown argument")),
                    };
                }
            }
        }

        Ok(result)
    }
}

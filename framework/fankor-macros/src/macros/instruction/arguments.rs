use crate::fnk_syn::FnkMetaArgumentList;
use crate::Result;
use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use syn::Attribute;

pub struct InstructionArguments {
    pub instructions_type_name: Option<Ident>,
    pub initial_validation: Option<Validation>,
    pub final_validation: Option<Validation>,
    pub attributes: Vec<Attribute>,
}

pub enum Validation {
    Implicit,
    Explicit(TokenStream),
}

impl InstructionArguments {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new instance of the Field struct from the given attributes.
    pub fn from(mut args: FnkMetaArgumentList) -> Result<InstructionArguments> {
        args.error_on_duplicated()?;

        let result = InstructionArguments {
            instructions_type_name: args.pop_ident("program", true)?,
            initial_validation: {
                match args.pop_element("initial_validation", true)? {
                    Some(v) => match v.value {
                        Some(v) => Some(Validation::Explicit(v.to_token_stream())),
                        None => Some(Validation::Implicit),
                    },
                    None => None,
                }
            },
            final_validation: {
                match args.pop_element("final_validation", true)? {
                    Some(v) => match v.value {
                        Some(v) => Some(Validation::Explicit(v.to_token_stream())),
                        None => Some(Validation::Implicit),
                    },
                    None => None,
                }
            },
            attributes: Vec::new(),
        };

        args.error_on_unknown()?;

        Ok(result)
    }
}

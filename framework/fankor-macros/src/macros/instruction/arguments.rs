use proc_macro2::TokenStream;
use quote::ToTokens;

use crate::fnk_syn::FnkMetaArgumentList;
use crate::Result;

pub struct InstructionArguments {
    pub initial_validation: Option<Validation>,
    pub final_validation: Option<Validation>,
    pub phantom: bool,
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
            phantom: args.pop_plain("phantom", true)?,
        };

        args.error_on_unknown()?;

        Ok(result)
    }
}

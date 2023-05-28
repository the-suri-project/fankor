use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::macros::program::programs::Program;
use crate::Result;

pub fn build_lpi(program: &Program) -> Result<TokenStream> {
    let methods = program.methods.iter().map(|v| {
        let program_name = &program.name;
        let method_name = &v.snake_name;
        let type_name = &v.name;
        let discriminant_name = format_ident!("{}Discriminant", program_name);

        quote! {
            pub fn #method_name<'info>(accounts: <#type_name<'info> as ::fankor::traits::Instruction<'info>>::LPI) -> ::fankor::errors::FankorResult<::fankor::prelude::solana_program::instruction::Instruction> {
                let mut data = vec![#discriminant_name::#type_name.code()];
                let mut metas = Vec::new();
                ::fankor::traits::LpiInstruction::serialize_into_instruction_parts(&accounts, &mut data, &mut metas)?;

                Ok(::fankor::prelude::solana_program::instruction::Instruction {
                    program_id: crate::ID,
                    accounts: metas,
                    data
                })
            }
        }
    });

    Ok(quote! {
        pub mod lpi {
            //! Methods for creating this program's instructions off-chain.
            //! The created instructions must be included into a transaction before
            //! being sent to the network.

            use super::*;
            use std::io::Cursor;

            #(#methods)*
        }
    })
}

use crate::macros::program::programs::Program;
use crate::Result;
use proc_macro2::TokenStream;
use quote::quote;

pub fn build_lpi(program: &Program) -> Result<TokenStream> {
    let methods = program.methods.iter().map(|v| {
        let method_name = &v.snake_name;
        let type_name = &v.name;

        quote! {
            pub fn #method_name<'info>(accounts: <#type_name<'info> as ::fankor::traits::Instruction<'info>>::LPI) -> ::fankor::errors::FankorResult<::fankor::prelude::solana_program::instruction::Instruction> {
                let mut data = Cursor::new(vec![]);
                let mut metas = Vec::new();
                ::fankor::traits::LpiInstruction::serialize_into_instruction_parts(&accounts, &mut data, &mut metas)?;

                Ok(::fankor::prelude::solana_program::instruction::Instruction {
                    program_id: crate::ID,
                    accounts: metas,
                    data: data.into_inner()
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

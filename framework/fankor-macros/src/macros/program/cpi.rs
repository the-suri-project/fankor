use crate::macros::program::programs::Program;
use crate::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn build_cpi(program: &Program) -> Result<TokenStream> {
    let methods = program.methods.iter().map(|v| {
        let program_name = &program.name;
        let method_name = &v.snake_name;
        let type_name = &v.name;
        let discriminant_name= format_ident!("{}Discriminant", program_name);

        let (result, result_param) = if let Some(result_type) = &v.return_type {
            (quote! {
                Ok(::fankor::models::CpiReturn::new())
            }, quote! {
                ::fankor::models::CpiReturn<#result_type>
            })
        } else {
            (quote! { Ok(()) }, quote! { () })
        };

        quote! {
            pub fn #method_name<'info>(_program: &::fankor::models::Program<super::#program_name>, accounts: <#type_name<'info> as ::fankor::traits::Instruction<'info>>::CPI, signer_seeds: &[&[&[u8]]]) -> ::fankor::errors::FankorResult<#result_param> {
                let mut data = Cursor::new(vec![#discriminant_name::#type_name.code()]);
                let mut metas = Vec::new();
                let mut infos = Vec::new();
                ::fankor::traits::CpiInstruction::serialize_into_instruction_parts(&accounts, &mut data, &mut metas, &mut infos)?;

                let instruction = ::fankor::prelude::solana_program::instruction::Instruction {
                    program_id: *<super::#program_name as ::fankor::traits::ProgramType>::address(),
                    accounts: metas,
                    data: data.into_inner(),
                };

                ::fankor::prelude::solana_program::program::invoke_signed(&instruction, &infos, signer_seeds)
                    .map_or_else(|e| Err(::fankor::errors::Error::ProgramError(e)), |_| Ok(()))?;

                #result
            }
        }
    });

    Ok(quote! {
        pub mod cpi {
            //! CPI methods for calling this program's instructions inside another Solana program.

            use super::*;
            use std::io::Cursor;

            #(#methods)*
        }
    })
}

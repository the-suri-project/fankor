use crate::macros::program::programs::Program;
use fankor_syn::Result;
use proc_macro2::TokenStream;
use quote::quote;

pub fn build_cpi(program: &Program) -> Result<TokenStream> {
    let methods = program.methods.iter().map(|v| {
        let program_name = &program.name;
        let method_name = &v.name;
        let account_type = &v.account_type;
        let discriminator = &v.discriminator;

        let (arguments, argument_param) = if let Some(argument_type) = &v.argument_type {
            let arguments = quote! {
                let mut ix_data = ::fankor::prelude::borsh::BorshSerialize::try_to_vec(&arguments)?;
                data.append(&mut ix_data);
            };

            (arguments, quote! {
                , arguments: #argument_type
            })
        } else {
            (quote! {}, quote! {})
        };

        let (result, result_param) = if let Some(result_type) = &v.result_type {
            (quote! {
                Ok(::fankor::models::CpiReturn::new())
            }, quote! {
                ::fankor::models::CpiReturn<#result_type>
            })
        } else {
            (quote! { Ok(()) }, quote! { () })
        };

        quote! {
            pub fn #method_name<'info>(_program: &::fankor::models::Program<super::#program_name>, accounts: <#account_type<'info> as ::fankor::traits::InstructionAccount<'info>>::CPI #argument_param, signer_seeds: &[&[&[u8]]]) -> ::fankor::errors::FankorResult<#result_param> {
                let mut data = [#(#discriminator),*].to_vec();
                #arguments

                let mut metas = Vec::new();
                let mut infos = Vec::new();
                ::fankor::traits::CpiInstructionAccount::to_account_metas_and_infos(&accounts, &mut metas, &mut infos)?;

                let instruction = ::fankor::prelude::solana_program::instruction::Instruction {
                    program_id: *<super::#program_name as ::fankor::traits::Program>::address(),
                    accounts: metas,
                    data,
                };

                ::fankor::prelude::solana_program::program::invoke_signed(&instruction, &infos, signer_seeds)
                    .map_or_else(|e| Err(::fankor::errors::Error::ProgramError(e)), |_| Ok(()))?;

                #result
            }
        }
    });

    Ok(quote! {
        pub mod cpi {
            use super::*;

            #(#methods)*
        }
    })
}

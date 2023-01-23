use crate::macros::program::programs::Program;
use crate::Result;
use convert_case::{Case, Converter};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn build_lpi(program: &Program) -> Result<TokenStream> {
    let case_converter = Converter::new()
        .from_case(Case::Snake)
        .to_case(Case::Pascal);

    let methods = program.methods.iter().map(|v| {
        let discriminant = format_ident!("{}", case_converter.convert(v.name.to_string()), span = v.name.span());
        let program_name = &program.name;
        let method_name = &v.name;
        let account_type = &v.account_type;
        let discriminant_name = format_ident!("{}Discriminant", program_name);

        let (arguments, argument_param) = if let Some(argument_type) = &v.argument_type {
            let arguments = quote! {
                let mut ix_data = ::fankor::prelude::borsh::BorshSerialize::try_to_vec(&arguments)?;
                data.append(&mut ix_data);
            };

            (arguments, quote! {
                , arguments: &#argument_type
            })
        } else {
            (quote! {}, quote! {})
        };

        quote! {
            pub fn #method_name<'info>(accounts: <#account_type as ::fankor::traits::InstructionAccount<'info>>::LPI #argument_param) -> ::fankor::errors::FankorResult<::fankor::prelude::solana_program::instruction::Instruction> {
                let mut data = vec![#discriminant_name::#discriminant.code()];
                #arguments

                let mut metas = Vec::new();
                ::fankor::traits::LpiInstructionAccount::to_account_metas(&accounts, &mut metas)?;

                Ok(::fankor::prelude::solana_program::instruction::Instruction {
                    program_id: crate::ID,
                    accounts: metas,
                    data,
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

            #(#methods)*
        }
    })
}

use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{Error, Item};

use crate::Result;

use crate::fnk_syn::FnkMetaArgumentList;
use crate::macros::program::programs::Program;
use cpi::build_cpi;
use lpi::build_lpi;

mod cpi;
mod lpi;
mod programs;

pub fn processor(args: FnkMetaArgumentList, input: Item) -> Result<proc_macro::TokenStream> {
    // Process input.
    let item = match input {
        Item::Enum(v) => v,
        _ => {
            return Err(Error::new(
                input.span(),
                "program macro can only be applied to an enum declaration",
            ));
        }
    };

    let program = Program::from(args, item)?;
    let name = &program.name;
    let discriminant_name = format_ident!("{}Discriminant", program.name);
    let name_str = name.to_string();

    let program_entry_name = format_ident!("__fankor_internal__program_{}_entry", name);
    let program_try_entry_name = format_ident!("__fankor_internal__program_{}_try_entry", name);

    let program_methods = program
        .methods
        .iter()
        .map(|v| {
            let variant_name = &v.name;
            let attrs = &v.attrs;

            quote! {
                #(#attrs)*
                #variant_name
            }
        })
        .collect::<Vec<_>>();

    let mut discriminant_constants = Vec::new();
    let dispatch_methods = program.methods.iter().map(|v| {
        let variant_name = &v.name;
        let instruction_msg = format!("Instruction: {}", v.name);

        discriminant_constants.push(quote! {
            const #variant_name: u8 = #discriminant_name::#variant_name.code();
        });

        quote! {
            #variant_name => {
                ::fankor::prelude::msg!(#instruction_msg);

                let mut ix_data = ix_data;
                let mut ix_accounts = accounts;
                let accounts = <#variant_name<'info> as fankor::traits::Instruction>::try_from(&context, &mut ix_data, &mut ix_accounts)?;

                if ix_accounts.len() != 0 {
                    return Err(::fankor::errors::FankorErrorCode::UnusedAccounts.into());
                }

                let result = accounts.processor(context.clone())?;

                // Write return data.
                if type_id_of(&result) != type_id_of(&()) {
                    ::fankor::prelude::solana_program::program::set_return_data(&::fankor::prelude::borsh::BorshSerialize::try_to_vec(&result).unwrap());
                }

                Ok(())
            }
        }
    }).collect::<Vec<_>>();

    let dispatch_default = if let Some(fallback_method_call) = &program.fallback_method_call {
        quote! {
            _ => {
                ::fankor::prelude::msg!("Instruction: Fallback");
                #fallback_method_call
            }
        }
    } else {
        quote! {
            _ => Err(::fankor::errors::FankorErrorCode::InstructionDiscriminantNotFound.into())
        }
    };

    let cpi_mod = build_cpi(&program)?;
    let lpi_mod = build_lpi(&program)?;

    let result = quote! {
        #[automatically_derived]
        #[allow(dead_code)]
        #[derive(Debug, Copy, Clone, EnumDiscriminants)]
        #[non_exhaustive]
        #[repr(u8)]
        pub enum #name {
            #(#program_methods,)*
        }

        #[automatically_derived]
        #[cfg(any(test, feature = "test"))]
        impl #name {
            pub fn new_program_test<'info>() -> ::solana_program_test::ProgramTest {
                ::solana_program_test::ProgramTest::new(
                    #name_str,
                    crate::ID,
                    Some(
                        |first_instruction_account: usize,
                            invoke_context: &mut ::solana_program_test::InvokeContext| {
                            ::solana_program_test::builtin_process_instruction(
                                |program_id: &::fankor::prelude::Pubkey,
                                    accounts: &[::fankor::prelude::AccountInfo],
                                    data: &[u8]| {
                                    // Hacks to change the lifetime to 'info.
                                    let program_id = unsafe {
                                        std::mem::transmute::<&::fankor::prelude::Pubkey, &'info ::fankor::prelude::Pubkey>(program_id)
                                    };
                                    let accounts = unsafe {
                                        std::mem::transmute::<&[::fankor::prelude::AccountInfo], &'info [::fankor::prelude::AccountInfo<'info>]>(accounts)
                                    };

                                    #program_entry_name(program_id, accounts, data)
                                },
                                first_instruction_account,
                                invoke_context,
                            )
                        },
                    )
                )
            }
        }

        #[automatically_derived]
        impl ::fankor::traits::ProgramType for #name {
            fn name() -> &'static str {
                #name_str
            }

            fn address() -> &'static Pubkey {
                &crate::ID
            }
        }

        #[automatically_derived]
        #[cfg(not(feature = "no-entrypoint"))]
        ::fankor::prelude::solana_program::entrypoint!(#program_entry_name);

        #[allow(non_snake_case)]
        #[automatically_derived]
        fn #program_entry_name<'info>(
            program_id: &'info ::fankor::prelude::Pubkey,
            accounts: &'info [::fankor::prelude::AccountInfo<'info>],
            data: &[u8],
        ) -> ::fankor::prelude::solana_program::entrypoint::ProgramResult {
            #program_try_entry_name(program_id, accounts, data).map_err(|e| {
                e.log();
                e.into()
            })
        }

        #[allow(non_snake_case)]
        #[allow(non_upper_case_globals)]
        #[automatically_derived]
        fn #program_try_entry_name<'info>(
            program_id: &'info ::fankor::prelude::Pubkey,
            accounts: &'info [::fankor::prelude::AccountInfo<'info>],
            data: &[u8],
        ) -> ::fankor::errors::FankorResult<()> {
            if *program_id != crate::ID {
                return Err(::fankor::errors::FankorErrorCode::DeclaredProgramIdMismatch.into());
            }

            if data.is_empty() {
                return Err(::fankor::errors::FankorErrorCode::MissingInstructionDiscriminant.into());
            }

            // Process data.
            let (sighash, ix_data) = (data[0], &data[1..]);

            // Build context.
            let context = unsafe {
                ::fankor::models::FankorContext::<'info>::new_unchecked(
                    program_id,
                    accounts
                )
            };

            // Hack to change the lifetime of the context to 'info and avoid a second lifetime
            // across the whole library.
            let context = unsafe {
                std::mem::transmute::<&::fankor::models::FankorContext, &'info ::fankor::models::FankorContext>(&context)
            };

            #(#discriminant_constants)*

            match sighash {
                #(#dispatch_methods,)*
                #dispatch_default
            }
        }

        #[cfg(not(feature = "library"))]
        #cpi_mod

        #[cfg(any(test, feature = "library"))]
        #lpi_mod
    };

    // Implement TypeScript generation.
    let method_registration = program
        .methods
        .iter()
        .map(|v| {
            let name = &v.name;
            let name_str = name.to_string();
            let discriminant_name_str = discriminant_name.to_string();

            quote! {
                action_context.add_program_method::<#name<'info>>(#discriminant_name_str, #name_str).unwrap();
            }
        })
        .collect::<Vec<_>>();

    let test_name = format_ident!("__ts_gen_test__program_{}", name_str);
    let test_name_str = test_name.to_string();
    let result = quote! {
        #result

        #[cfg(feature = "ts-gen")]
        #[automatically_derived]
        #[allow(non_snake_case)]
        pub mod #test_name {
            use super::*;
            use ::fankor::prelude::ts_gen::accounts::TsInstructionGen;
            use ::fankor::prelude::ts_gen::types::TsTypesCache;
            use std::borrow::Cow;

            #[test]
            fn build<'info>() {
                // Register name action action.
                crate::__ts_gen_test__setup::BUILD_CONTEXT.register_action(#test_name_str, file!(), move |action_context| {
                    action_context.set_context_name(#name_str).unwrap();
                    action_context.add_constant("PROGRAM_NAME", #name_str).unwrap();
                    #(#method_registration)*
                })
            }
        }
    };

    Ok(result.into())
}

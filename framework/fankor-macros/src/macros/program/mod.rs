use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{AttributeArgs, Error, Item};

use crate::Result;

use crate::macros::program::programs::Program;
use cpi::build_cpi;
use lpi::build_lpi;

mod cpi;
mod lpi;
mod programs;

pub fn processor(args: AttributeArgs, input: Item) -> Result<proc_macro::TokenStream> {
    // Process arguments.
    if !args.is_empty() {
        return Err(Error::new(
            input.span(),
            "account macro does not accept arguments",
        ));
    }

    // Process input.
    let item = match input {
        Item::Impl(v) => v,
        _ => {
            return Err(Error::new(
                input.span(),
                "program macro can only be applied to impl declarations",
            ));
        }
    };

    let program = Program::from(item)?;
    let name = &program.name;
    let name_str = name.to_string();
    let item = &program.item;

    let program_entry_name = format_ident!("__fankor_internal__program_{}_entry", name);
    let program_try_entry_name = format_ident!("__fankor_internal__program_{}_try_entry", name);

    let discriminators = program.methods.iter().map(|v| {
        let fn_name = format_ident!("{}_discriminator", v.name);
        let discriminant = &v.discriminant;

        quote! {
            pub fn #fn_name() -> u8 {
                #discriminant
            }
        }
    });

    let dispatch_methods = program.methods.iter().map(|v| {
        let fn_name = &v.name;
        let discriminant = &v.discriminant;
        let account_type = &v.account_type;

        let accounts_tokens = quote! {
            let mut ix_accounts = accounts;
            let accounts = <#account_type as fankor::traits::InstructionAccount>::try_from(&context, &mut ix_accounts)?;

            if ix_accounts.len() != 0 {
                return Err(::fankor::errors::FankorErrorCode::UnusedAccounts.into());
            }
        };

        let result = if v.result_type.is_some() {
            quote! {
                ::fankor::prelude::solana_program::program::set_return_data(&::fankor::prelude::borsh::BorshSerialize::try_to_vec(&result).unwrap());
            }
        } else {
            quote! {}
        };

        let instruction_msg = format!("Instruction: {}", v.pascal_name);
        if let Some(argument_type) = &v.argument_type {
            quote! {
                #discriminant => {
                    ::fankor::prelude::msg!(#instruction_msg);

                    #accounts_tokens

                    let mut ix_data = ix_data;
                    let arguments = <#argument_type as fankor::traits::AccountDeserialize>::try_deserialize(&mut ix_data)?;
                    let result = #name::#fn_name(context.clone(), accounts, arguments)?;
                    #result

                    Ok(())
                }
            }
        } else {
            quote! {
                #discriminant => {
                    ::fankor::prelude::msg!(#instruction_msg);

                    #accounts_tokens

                    let result = #name::#fn_name(context.clone(), accounts)?;
                    #result

                    Ok(())
                }
            }
        }
    });

    let dispatch_default = if let Some(fallback_method) = &program.fallback_method {
        let fn_name = &fallback_method.name;
        quote! {
            _ => {
                ::fankor::prelude::msg!("Instruction: Fallback");
                #name::#fn_name(program_id, accounts, data)
            }
        }
    } else {
        quote! {
            _ => Err(::fankor::errors::FankorErrorCode::InstructionDiscriminatorNotFound.into())
        }
    };

    let test_method = quote! {
        #[allow(non_snake_case)]
        #[automatically_derived]
        fn new_program_test<'info>() -> ::solana_program_test::ProgramTest {
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
    };

    let cpi_mod = build_cpi(&program)?;
    let lpi_mod = build_lpi(&program)?;

    let result = quote! {
        #[derive(Debug, Copy, Clone)]
        pub struct #name;

        #item

        #[automatically_derived]
        impl #name {
            #(#discriminators)*
        }

        #[cfg(any(test))]
        #[automatically_derived]
        impl #name {
            #test_method
        }

        #[automatically_derived]
        impl ::fankor::traits::Program for #name {
            fn name() -> &'static str {
                #name_str
            }

            fn address() -> &'static Pubkey {
                &crate::ID
            }
        }

        #[automatically_derived]
        #[cfg(not(feature = "library"))]
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
                return Err(::fankor::errors::FankorErrorCode::InstructionDiscriminatorMissing.into());
            }

            // Process data.
            let (sighash, ix_data) = (data[0], &data[1..]);

            // Build context.
            let context = unsafe {
                ::fankor::models::FankorContext::<'info>::new(
                    program_id,
                    accounts
                )
            };

            // Hack to change the lifetime of the context to 'info and avoid a second lifetime
            // across the whole library.
            let context = unsafe {
                std::mem::transmute::<&::fankor::models::FankorContext, &'info ::fankor::models::FankorContext>(&context)
            };

            match sighash {
                #(#dispatch_methods,)*
                #dispatch_default
            }
        }

        #cpi_mod
        #lpi_mod
    };

    Ok(result.into())
}

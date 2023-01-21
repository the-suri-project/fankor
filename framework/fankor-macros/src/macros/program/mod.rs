use convert_case::{Case, Converter};
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
    let case_converter = Converter::new()
        .from_case(Case::Snake)
        .to_case(Case::Pascal);

    // Process arguments.
    if !args.is_empty() {
        return Err(Error::new(
            input.span(),
            "program macro does not accept arguments",
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
    let discriminant_name = format_ident!("{}Discriminant", program.name);
    let name_str = name.to_string();
    let item = &program.item;

    let program_entry_name = format_ident!("__fankor_internal__program_{}_entry", name);
    let program_try_entry_name = format_ident!("__fankor_internal__program_{}_try_entry", name);

    let mut discriminant_fields = Vec::new();
    let discriminants = program
        .methods
        .iter()
        .map(|v| {
            let name = format_ident!(
                "{}",
                case_converter.convert(v.name.to_string()),
                span = v.name.span()
            );
            let discriminant = &v.discriminant;

            discriminant_fields.push(name.clone());

            quote! {
                Self::#name => #discriminant
            }
        })
        .collect::<Vec<_>>();

    let mut discriminant_constants = Vec::new();
    let dispatch_methods = program.methods.iter().map(|v| {
        let discriminant = format_ident!(
            "{}",
            case_converter.convert(v.name.to_string()),
            span = v.name.span()
        );
        let fn_name = &v.name;
        let account_type = &v.account_type;

        let (arguments_tokens, validation_call, method_call) = if let Some(argument_type) = &v.argument_type {
            let arguments_tokens = quote! {
                let mut ix_data = ix_data;
                let arguments: #argument_type = ::fankor::prelude::borsh::BorshDeserialize::deserialize(&mut ix_data)?;
            };

            let validation_call = if v.validation_with_args {
                quote! {
                    accounts.validate(&context, &arguments)?;
                }
            } else {
                quote! {
                    accounts.validate(&context)?;
                }
            };

            let method_call = quote! {
                #name::#fn_name(context.clone(), accounts, arguments)?
            };

            (arguments_tokens, validation_call, method_call)
        } else {
            let arguments_tokens = quote! {};

            let validation_call = quote! {
                accounts.validate(&context)?;
            };

            let method_call = quote! {
                #name::#fn_name(context.clone(), accounts)?
            };

            (arguments_tokens, validation_call, method_call)
        };

        let result = if v.result_type.is_some() {
            quote! {
                ::fankor::prelude::solana_program::program::set_return_data(&::fankor::prelude::borsh::BorshSerialize::try_to_vec(&result).unwrap());
            }
        } else {
            quote! {}
        };

        let instruction_msg = format!("Instruction: {}", v.pascal_name);

        discriminant_constants.push(quote! {
            const #discriminant: u8 = #discriminant_name::#discriminant.code();
        });

        quote! {
            #discriminant => {
                ::fankor::prelude::msg!(#instruction_msg);

                #arguments_tokens

                let mut ix_accounts = accounts;
                let accounts = <#account_type as fankor::traits::InstructionAccount>::try_from(&context, &mut ix_accounts)?;

                if ix_accounts.len() != 0 {
                    return Err(::fankor::errors::FankorErrorCode::UnusedAccounts.into());
                }

                #validation_call

                let result = #method_call;
                #result

                Ok(())
            }
        }
    }).collect::<Vec<_>>();

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
            _ => Err(::fankor::errors::FankorErrorCode::InstructionDiscriminantNotFound.into())
        }
    };

    let cpi_mod = build_cpi(&program)?;
    let lpi_mod = build_lpi(&program)?;

    let result = quote! {
        #[derive(Debug, Copy, Clone)]
        pub struct #name;

        #[automatically_derived]
        #[allow(dead_code)]
        #item

        #[automatically_derived]
        #[cfg(feature = "test")]
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
                return Err(::fankor::errors::FankorErrorCode::InstructionDiscriminantMissing.into());
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

            #(#discriminant_constants)*

            match sighash {
                #(#dispatch_methods,)*
                #dispatch_default
            }
        }

        #[allow(dead_code)]
        #[automatically_derived]
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
        #[non_exhaustive]
        #[repr(u8)]
        pub enum #discriminant_name {
            #(#discriminant_fields,)*
        }

        #[automatically_derived]
        impl #discriminant_name {
            #[inline(always)]
            pub const fn code(&self) -> u8 {
                match self {
                    #(#discriminants,)*
                }
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
            let discriminant = format_ident!(
                "{}",
                case_converter.convert(v.name.to_string()),
                span = v.name.span()
            );

            let name = &v.name;
            let name_str = name.to_string();
            let ty = &v.account_type;

            if let Some(argument_type) = &v.argument_type {
                quote! {
                    action_context.add_program_method_with_args::<#ty, #argument_type>(#name_str, #discriminant_name::#discriminant.code()).unwrap();
                }
            }else {
                quote! {
                    action_context.add_program_method::<#ty>(#name_str, #discriminant_name::#discriminant.code()).unwrap();
                }
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
            use ::fankor::prelude::ts_gen::accounts::TsInstructionAccountGen;
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

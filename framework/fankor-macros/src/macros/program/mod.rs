use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{AttributeArgs, Error, Item};

use fankor_syn::fankor::read_fankor_toml;
use fankor_syn::Result;

use crate::macros::program::program::Program;
use cpi::build_cpi;
use lpi::build_lpi;

mod cpi;
mod lpi;
mod program;

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

    // Read the Fankor.toml file.
    let config = read_fankor_toml();
    let instructions_config = config.instructions.as_ref().unwrap();
    let discriminator_size_u8 = instructions_config.discriminator_size.unwrap();
    let discriminator_size = discriminator_size_u8 as usize;

    let program = Program::from(item, &instructions_config)?;
    let name = &program.name;
    let name_str = name.to_string();
    let item = &program.item;

    let test_unique_program = format_ident!("__fankor_internal__test__unique_program_{}", name);
    let program_entry_name = format_ident!("__fankor_internal__program_{}_entry", name);
    let program_try_entry_name = format_ident!("__fankor_internal__program_{}_try_entry", name);

    let discriminators = program.methods.iter().map(|v| {
        let fn_name = format_ident!("{}_discriminator", v.name);
        let discriminator = &v.discriminator;

        quote! {
            pub fn #fn_name() -> [u8; #discriminator_size] {
                [#(#discriminator,)*]
            }
        }
    });

    let dispatch_methods = program.methods.iter().map(|v| {
        let fn_name = &v.name;
        let discriminator = &v.discriminator;
        let account_type = &v.account_type;

        let accounts_tokens = quote! {
            let mut ix_accounts = accounts;
            let accounts = <#account_type as fankor::traits::InstructionAccount>::try_from(&context, &mut ix_accounts)?;

            if ix_accounts.len() != 0 {
                return Err(::fankor::errors::ErrorCode::TooManyAccounts.into());
            }
        };

        let result = if v.result_type.is_some() {
            quote! {
                ::fankor::prelude::solana_program::program::set_return_data(&::fankor::prelude::borsh::BorshSerialize::try_to_vec(&result).unwrap());
            }
        }else {
            quote! {}
        };

        let instruction_msg = format!("Instruction: {}", v.pascal_name);
        if let Some(argument_type) = &v.argument_type {
            quote! {
                [#(#discriminator,)*] => {
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
                [#(#discriminator,)*] => {
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
            _ => Err(::fankor::errors::ErrorCode::InstructionDiscriminatorNotFound.into())
        }
    };

    let cpi_mod = build_cpi(&program)?;
    let lpi_mod = build_lpi(&program)?;

    let result = quote! {
        #[derive(Debug, Copy, Clone)]
        pub struct #name;

        #[cfg(not(feature = "library"))]
        #item

        #[cfg(feature = "library")]
        #[automatically_derived]
        impl #name {
            #(#discriminators)*
        }

        #[automatically_derived]
        impl ::fankor::traits::Program for #name {
            fn address() -> &'static Pubkey {
                &crate::ID
            }
        }

        #[automatically_derived]
        #[cfg(not(feature = "library"))]
        ::fankor::prelude::solana_program::entrypoint!(#program_entry_name);

        #[allow(non_snake_case)]
        #[automatically_derived]
        #[cfg(not(feature = "library"))]
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
        #[cfg(not(feature = "library"))]
        fn #program_try_entry_name<'info>(
            program_id: &'info ::fankor::prelude::Pubkey,
            accounts: &'info [::fankor::prelude::AccountInfo<'info>],
            data: &[u8],
        ) -> ::fankor::errors::FankorResult<()> {
            if *program_id != crate::ID {
                return Err(::fankor::errors::ErrorCode::DeclaredProgramIdMismatch.into());
            }

            if data.len() < #discriminator_size {
                return Err(::fankor::errors::ErrorCode::InstructionDiscriminatorMissing.into());
            }

            // Process data.
            let (sighash, ix_data) = {
                let mut sighash: [u8; #discriminator_size] = [0; #discriminator_size];
                sighash.copy_from_slice(&data[..#discriminator_size]);
                (sighash, &data[#discriminator_size..])
            };

            // Build context.
            let context = unsafe {
                ::fankor::models::FankorContext::<'info>::new(
                    #discriminator_size_u8,
                    program_id,
                    accounts
                )
            };

            // Trick to change the lifetime of the context to 'info and avoid a second lifetime
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

        #[allow(non_snake_case)]
        #[automatically_derived]
        #[cfg(test)]
        #[test]
        fn #test_unique_program() {
           let program_name = #name_str;
           let helper = &crate::__internal__idl_builder_test__root::PROGRAM_HELPER;

           if let Err(item) = helper.add_program(program_name) {
               panic!("More than one program in a single crate is not supported");
           }
        }
    };

    Ok(result.into())
}

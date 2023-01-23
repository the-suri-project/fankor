use quote::{format_ident, quote};
use syn::ItemStruct;

use crate::Result;

use crate::macros::instruction_accounts::arguments::{InstructionArguments, Validation};
use crate::macros::instruction_accounts::field::{check_fields, Field};

pub fn process_struct(item: ItemStruct) -> Result<proc_macro::TokenStream> {
    let instruction_arguments = InstructionArguments::from(&item.attrs)?;
    let name = &item.ident;
    let vis = &item.vis;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();
    let ixn_args_type = instruction_arguments
        .args
        .clone()
        .map(|args| quote! { args: &#args })
        .unwrap_or_default();

    let mapped_fields = item
        .fields
        .iter()
        .map(|v| Field::from(v.clone()))
        .collect::<Result<Vec<Field>>>()?;
    check_fields(&mapped_fields)?;

    let try_from_fn_deserialize = mapped_fields.iter().map(|v| {
        let name = &v.name;
        let ty = &v.ty;

        quote! {
            let #name = <#ty as ::fankor::traits::InstructionAccount>::try_from(context, accounts)?;
        }
    });

    let mut pda_methods = Vec::new();
    let try_from_fn_conditions = mapped_fields.iter().map(|v| {
        let name = &v.name;
        let name_str = name.to_string();

        let data = v.data.iter().map(|v| {
            let name = &v.name;
            let value = &v.value;

            quote! {
                let #name = #value;
            }
        }).collect::<Vec<_>>();

        let mut account_info_conditions = Vec::new();
        let mut constraints_conditions = Vec::new();

        if let Some(owner) = &v.owner {
            account_info_conditions.push(quote! {{
                let actual = info.owner;
                let expected = #owner;

                if actual != expected {
                    return Err(::fankor::errors::FankorErrorCode::AccountConstraintOwnerMismatch {
                        actual: *actual,
                        expected: *expected,
                        account: #name_str,
                    }.into());
                }
            }});
        }

        if let Some(address) = &v.address {
            account_info_conditions.push(quote! {{
                let actual = info.key;
                let expected = #address;

                if actual != expected {
                    return Err(::fankor::errors::FankorErrorCode::AccountConstraintAddressMismatch {
                        actual: *actual,
                        expected: *expected,
                        account: #name_str,
                    }.into());
                }
            }});
        }

        if let Some(initialized) = &v.initialized {
            account_info_conditions.push(quote! {{
                let initialized = #initialized;

                if initialized {
                    if info.owner == &system_program::ID && info.lamports() == 0 {
                        return Err(::fankor::errors::FankorErrorCode::AccountConstraintNotInitialized {
                            account: #name_str,
                        }.into());
                    }
                } else if info.owner != &system_program::ID || info.lamports() > 0 {
                    return Err(::fankor::errors::FankorErrorCode::AccountConstraintInitialized {
                        account: #name_str,
                    }.into());
                }
            }});
        }

        if let Some(writable) = &v.writable {
            account_info_conditions.push(quote! {{
                let writable = #writable;

                if writable {
                    if !info.is_writable {
                        return Err(::fankor::errors::FankorErrorCode::AccountConstraintNotWritable {
                            account: #name_str,
                        }.into());
                    }
                } else if info.is_writable {
                    return Err(::fankor::errors::FankorErrorCode::AccountConstraintWritable {
                        account: #name_str,
                    }.into());
                }
            }});
        }

        if let Some(executable) = &v.executable {
            account_info_conditions.push(quote! {{
                let executable = #executable;

                if executable {
                    if !info.executable {
                        return Err(::fankor::errors::FankorErrorCode::AccountConstraintNotExecutable {
                            account: #name_str,
                        }.into());
                    }
                } else if info.executable {
                    return Err(::fankor::errors::FankorErrorCode::AccountConstraintExecutable {
                        account: #name_str,
                    }.into());
                }
            }});
        }

        if let Some(rent_exempt) = &v.rent_exempt {
            account_info_conditions.push(quote! {{
                let rent_exempt = #rent_exempt;
                let lamports = info.lamports();
                let data_len = info.data_len();

                let rent: sysvar::rent::Rent = ::fankor::prelude::solana_program::sysvar::Sysvar::get().expect("Cannot access Rent Sysvar");
                let is_rent_exempt = rent.is_exempt(lamports, data_len);

                if rent_exempt {
                    if !is_rent_exempt {
                        return Err(::fankor::errors::FankorErrorCode::AccountConstraintNotRentExempt {
                            account: #name_str,
                        }.into());
                    }
                } else if is_rent_exempt {
                    return Err(::fankor::errors::FankorErrorCode::AccountConstraintRentExempt {
                        account: #name_str,
                    }.into());
                }
            }});
        }

        if let Some(signer) = &v.signer {
            account_info_conditions.push(quote! {{
                let signer = #signer;

                if signer {
                    if !info.is_signer {
                        return Err(::fankor::errors::FankorErrorCode::AccountConstraintNotSigner {
                            account: #name_str,
                        }.into());
                    }
                } else if info.is_signer {
                    return Err(::fankor::errors::FankorErrorCode::AccountConstraintSigner {
                        account: #name_str,
                    }.into());
                }
            }});
        }

        if let Some(pda) = &v.pda {
            let pda_method_name = format_ident!("{}_pda_seeds", name);
            let seeds = &pda.data;

            pda_methods.push(quote! {
                pub fn #pda_method_name(&self, context: &FankorContext<'info>, #ixn_args_type) -> FankorResult<Vec<Vec<u8>>> {
                    let info = match self.#name.pda_info() {
                        Some(v) => v,
                        None => return Err(::fankor::errors::FankorErrorCode::MissingSeedsAccount.into()),
                    };

                    let mut seeds = {
                        #(#data)*
                        let seeds = #seeds;

                        seeds.into_iter().map(|v|v.to_vec()).collect::<Vec<_>>()
                    };

                    // Get bump.
                    let bump = context.get_bump_seed_from_account(info).ok_or_else(|| ::fankor::errors::FankorErrorCode::MissingPdaBumpSeed {
                        account: *info.key
                    })?;
                    seeds.push(vec![bump]);

                    Ok(seeds)
                }
            });

            let program_id = v.pda_program_id.clone().unwrap_or_else(|| quote! { context.program_id() });
            let error = match &pda.error {
                Some(v) => {
                    quote! {
                        .map_err(|_| #v)?
                    }
                },
                None => {
                    quote! { ? }
                }
            };

            account_info_conditions.push(quote! {{
                let seeds = #seeds;
                let program_id = #program_id;

                context.check_canonical_pda_with_program(info, &seeds, program_id)#error;
            }});
        }

        for constraint in &v.constraints {
            let condition = &constraint.data;
            let error = match &constraint.error {
                Some(v) => v.clone(),
                None => {
                    quote! {
                        FankorErrorCode::AccountConstraintFailed {
                            account: #name_str,
                            constraint: stringify!(#condition),
                        }
                    }
                }
            };

            constraints_conditions.push(quote! {{
                require!(#condition, #error);
            }});
        }

        let min = v.min.as_ref().map(|min| {
            quote! {{
                let expected = #min;
                let actual = self.#name.len();

                if actual < expected {
                    return Err(::fankor::errors::FankorErrorCode::AccountConstraintMinimumMismatch {
                        actual,
                        expected,
                        account: #name_str,
                    }.into());
                }
            }}
        });

        let max = v.max.as_ref().map(|max| {
            quote! {{
                let expected = #max;
                let actual = self.#name.len();

                if actual > expected {
                    return Err(::fankor::errors::FankorErrorCode::AccountConstraintMaximumMismatch {
                        actual,
                        expected,
                        account: #name_str,
                    }.into());
                }
            }}
        });

        if !account_info_conditions.is_empty() || !constraints_conditions.is_empty() {
            let account_info_conditions = if account_info_conditions.is_empty() {
                quote! {}
            } else {
                quote! {
                    let mut closure = |info: &AccountInfo<'info>| {
                        #(#account_info_conditions)*
                        Ok(())
                    };
                    verification_config.account_info = Some(&mut closure);
                }
            };

            let constraints_conditions = if constraints_conditions.is_empty() {
                quote! {}
            } else {
                quote! {
                    let mut closure = |info: &AccountInfo<'info>| {
                        #(#constraints_conditions)*
                        Ok(())
                    };
                    verification_config.constraints = Some(&mut closure);
                }
            };

            quote! {
                #(#data)*

                let mut verification_config = AccountInfoVerification::default();
                #account_info_conditions
                #constraints_conditions

                self.#name.verify_account_infos(&mut verification_config)?;

                #min
                #max
            }
        } else {
            quote! {
                #(#data)*

                #min
                #max
            }
        }
    });

    let fields = item.fields.iter().map(|v| &v.ident);

    // CpiInstructionAccount implementation
    let cpi_name = format_ident!("Cpi{}", name);
    let cpi_fields = mapped_fields.iter().map(|v| {
        let name = &v.name;
        let ty = &v.ty;

        quote! {
            pub #name:<#ty as ::fankor::traits::InstructionAccount<'info>>::CPI
        }
    });
    let cpi_fn_elements = mapped_fields.iter().map(|v| {
        let name = &v.name;

        let mut any = false;
        let (writable_let, writable_for) = if let Some(writable) = &v.writable {
            let writable_let = quote! { let writable = #writable; };
            let writable_for = quote! {
                meta.is_writable = writable;
            };

            any = true;

            (writable_let, writable_for)
        } else {
            (quote! {}, quote! {})
        };

        let (signer_let, signer_for) = if let Some(signer) = &v.signer {
            let signer_let = quote! { let signer = #signer; };
            let signer_for = quote! {
                meta.is_signer = signer;
            };

            any = true;

            (signer_let, signer_for)
        } else {
            (quote! {}, quote! {})
        };

        if any {
            quote! {
                {
                    let from = metas.len();
                    ::fankor::traits::CpiInstructionAccount::to_account_metas_and_infos(&self.#name, metas, infos)?;
                    let to = metas.len();
                    #writable_let
                    #signer_let

                    for meta in &mut metas[from..to] {
                        #writable_for
                        #signer_for
                    }
                }
            }
        } else {
            quote! {
               ::fankor::traits::CpiInstructionAccount::to_account_metas_and_infos(&self.#name, metas, infos)?;
            }
        }
    });

    // LpiInstructionAccount implementation
    let lpi_name = format_ident!("Lpi{}", name);
    let lpi_fields = mapped_fields.iter().map(|v| {
        let name = &v.name;
        let ty = &v.ty;

        quote! {
            pub #name:<#ty as ::fankor::traits::InstructionAccount<'info>>::LPI
        }
    });
    let lpi_fn_elements = mapped_fields.iter().map(|v| {
        let name = &v.name;

        let mut any = false;
        let (writable_let, writable_for) = if let Some(writable) = &v.writable {
            let writable_let = quote! { let writable = #writable; };
            let writable_for = quote! {
                meta.is_writable = writable;
            };

            any = true;

            (writable_let, writable_for)
        } else {
            (quote! {}, quote! {})
        };

        let (signer_let, signer_for) = if let Some(signer) = &v.signer {
            let signer_let = quote! { let signer = #signer; };
            let signer_for = quote! {
                meta.is_signer = signer;
            };

            any = true;

            (signer_let, signer_for)
        } else {
            (quote! {}, quote! {})
        };

        if any {
            quote! {
                {
                    let from = metas.len();
                    ::fankor::traits::LpiInstructionAccount::to_account_metas(&self.#name, metas)?;
                    let to = metas.len();
                    #writable_let
                    #signer_let

                    for meta in &mut metas[from..to] {
                        #writable_for
                        #signer_for
                    }
                }
            }
        } else {
            quote! {
                ::fankor::traits::LpiInstructionAccount::to_account_metas(&self.#name, metas)?;
            }
        }
    });

    // Min accounts.
    let min_accounts_fn_elements = mapped_fields.iter().map(|v| {
        let ty = &v.ty;

        quote! {
            min_accounts += <#ty as ::fankor::traits::InstructionAccount>::min_accounts();
        }
    });

    // Validations.
    let validation_args = if ixn_args_type.is_empty() {
        quote! {}
    } else {
        quote! { args }
    };
    let initial_validation = &instruction_arguments.initial_validation.map(|v| match v {
        Validation::Implicit => {
            quote! {
                self.initial_validation(context, #validation_args)?;
            }
        }
        Validation::Explicit(v) => v,
    });
    let final_validation = &instruction_arguments.final_validation.map(|v| match v {
        Validation::Implicit => {
            quote! {
                self.final_validation(context, #validation_args)?;
            }
        }
        Validation::Explicit(v) => v,
    });

    // Result
    let result = quote! {
        #[automatically_derived]
        impl #impl_generics ::fankor::traits::InstructionAccount<'info> for #name #ty_generics #where_clause {
            type CPI = #cpi_name <'info>;
            type LPI = #lpi_name <'info>;

            fn min_accounts() -> usize {
                let mut min_accounts = 0;
                #(#min_accounts_fn_elements)*
                min_accounts
            }

            fn try_from(
                context: &'info FankorContext<'info>,
                accounts: &mut &'info [AccountInfo<'info>],
            ) -> ::fankor::errors::FankorResult<Self> {
                #(#try_from_fn_deserialize)*

                Ok(Self {
                    #(#fields),*
                })
            }
        }

        #[automatically_derived]
        impl #impl_generics #name #ty_generics #where_clause {
            pub fn validate(
                &self,
                context: &'info FankorContext<'info>,
                #ixn_args_type
            ) -> ::fankor::errors::FankorResult<()> {
                use ::fankor::traits::InstructionAccount;

                #initial_validation

                #(#try_from_fn_conditions)*

                #final_validation

                Ok(())
            }

            #(#pda_methods)*
        }

        #[automatically_derived]
        #vis struct #cpi_name <'info> {
            #(#cpi_fields),*
        }

        #[automatically_derived]
        impl <'info> ::fankor::traits::CpiInstructionAccount<'info> for #cpi_name <'info> {
            fn to_account_metas_and_infos(
                &self,
                metas: &mut Vec<AccountMeta>,
                infos: &mut Vec<AccountInfo<'info>>,
            ) -> FankorResult<()> {
                #(#cpi_fn_elements)*
                Ok(())
            }
        }

        #[automatically_derived]
        #vis struct #lpi_name <'info> {
            #(#lpi_fields),*
        }

        #[automatically_derived]
        impl <'info> ::fankor::traits::LpiInstructionAccount for #lpi_name <'info> {
            fn to_account_metas(&self, metas: &mut Vec<::fankor::prelude::solana_program::instruction::AccountMeta>) -> ::fankor::errors::FankorResult<()> {
                #(#lpi_fn_elements)*
                Ok(())
            }
        }
    };

    // Implement TypeScript generation.
    let name_str = name.to_string();
    let mut type_replacements = Vec::new();
    let mut metas_replacements = Vec::new();
    let mut metas_fields = Vec::new();
    let ts_types = mapped_fields.iter().map(|v| {
        let ty = &v.ty;
        let types_replacement_str = format!("_r_interface_types_{}_r_", v.name);
        let metas_replacement_str = format!("_r_interface_metas_{}_r_", v.name);
        let writable = v.writable.clone().unwrap_or(quote! { false });
        let signer = v.signer.clone().unwrap_or(quote! { false });

        type_replacements.push(quote! {
             .replace(#types_replacement_str, &< #ty as TsInstructionAccountGen>::generate_type(registered_types))
        });
        metas_fields.push(metas_replacement_str.clone());

        let value_str = format!("{{}}.{}", v.name);
        metas_replacements.push(quote! {
             .replace(#metas_replacement_str, &< #ty as TsInstructionAccountGen>::get_external_account_metas(Cow::Owned(format!(#value_str, value)), #signer, #writable))
        });

        format!("{}: {}", v.name, types_replacement_str)
    }).collect::<Vec<_>>();

    let ts_type = format!(
        "export interface {} {{ {} }};",
        name_str,
        ts_types.join(",")
    );

    let ts_metas = metas_fields.join("");
    let get_metas_of_replacement_str = format!("getMetasOf{}(_r_value_r_,accountMetas);", name_str);
    let test_name = format_ident!("__ts_gen_test__instruction_accounts_{}", name_str);
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

            #[automatically_derived]
            impl #impl_generics TsInstructionAccountGen for #name #ty_generics #where_clause {
                fn value_type() -> Cow<'static, str> {
                    Cow::Borrowed(#name_str)
                }

                fn generate_type(registered_types: &mut TsTypesCache) -> Cow<'static, str> {
                    let name = Self::value_type();

                    if registered_types.contains_key(&name) {
                        return name;
                    }

                    // Prevents infinite recursion.
                    registered_types.insert(name.clone(), std::borrow::Cow::Borrowed(""));

                    let ts_type = #ts_type.to_string() #(#type_replacements)*;
                    *registered_types.get_mut(&name).unwrap() = std::borrow::Cow::Owned(ts_type);

                    name
                }

                fn get_account_metas(
                    value: Cow<'static, str>,
                    _signer: bool,
                    _writable: bool,
                ) -> Cow<'static, str> {
                    Cow::Owned(#ts_metas #(#metas_replacements)*)
                }

                fn get_external_account_metas(
                    value: Cow<'static, str>,
                    _signer: bool,
                    _writable: bool,
                ) -> Cow<'static, str> {
                    Cow::Owned(#get_metas_of_replacement_str.replace("_r_value_r_", &value))
                }
            }

            #[test]
            fn build() {
                // Register action.
                crate::__ts_gen_test__setup::BUILD_CONTEXT.register_action(#test_name_str, file!(), move |action_context| {
                    action_context.add_instruction_account::<#name>().unwrap();
                })
            }
        }
    };

    Ok(result.into())
}

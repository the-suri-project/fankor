use crate::fnk_syn::FnkMetaArgumentList;
use convert_case::{Case, Converter};
use quote::{format_ident, quote};
use syn::ItemStruct;

use crate::Result;

use crate::macros::instruction::arguments::{InstructionArguments, Validation};
use crate::macros::instruction::field::{check_fields, Field};

pub fn process_struct(
    args: FnkMetaArgumentList,
    item: ItemStruct,
) -> Result<proc_macro::TokenStream> {
    let arguments = InstructionArguments::from(args)?;
    let name = &item.ident;
    let visibility = &item.vis;
    let attributes = &item.attrs;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    let mapped_fields = item
        .fields
        .iter()
        .map(|v| Field::from(v.clone()))
        .collect::<Result<Vec<Field>>>()?;
    check_fields(&mapped_fields)?;

    let final_fields = mapped_fields.iter().map(|v| {
        let name = &v.name;
        let ty = &v.ty;
        let attrs = &v.attrs;
        let vis = &v.vis;

        quote! {
            #(#attrs)*
            #vis #name: #ty
        }
    });

    let try_from_fn_deserialize = mapped_fields.iter().map(|v| {
        let name = &v.name;
        let ty = &v.ty;

        quote! {
            let #name = <#ty as ::fankor::traits::Instruction>::try_from(context, buf, accounts)?;
        }
    });

    let mut pda_methods = Vec::new();
    let validate_method_fields = mapped_fields.iter().map(|v| {
        let name = &v.name;
        let name_str = name.to_string();

        // We need to reverse the data because it was processed in reverse order.
        let data = v.data.iter().rev().map(|v| {
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
                pub fn #pda_method_name(&self, context: &FankorContext<'info>) -> FankorResult<Vec<Vec<u8>>> {
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

        let result = if !account_info_conditions.is_empty() || !constraints_conditions.is_empty() {
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
            }
        } else {
            quote! {
                #(#data)*
            }
        };

        Ok(result)
    }).collect::<Result<Vec<_>>>()?;

    let fields = item.fields.iter().map(|v| &v.ident);

    // CpiInstruction implementation
    let cpi_name = format_ident!("Cpi{}", name);
    let cpi_fields = mapped_fields.iter().map(|v| {
        let name = &v.name;
        let ty = &v.ty;

        quote! {
            pub #name:<#ty as ::fankor::traits::Instruction<'info>>::CPI
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
                    ::fankor::traits::CpiInstruction::serialize_into_instruction_parts(&self.#name, writer, metas, infos)?;
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
               ::fankor::traits::CpiInstruction::serialize_into_instruction_parts(&self.#name, writer, metas, infos)?;
            }
        }
    });

    // LpiInstruction implementation
    let lpi_name = format_ident!("Lpi{}", name);
    let lpi_fields = mapped_fields.iter().map(|v| {
        let name = &v.name;
        let ty = &v.ty;

        quote! {
            pub #name:<#ty as ::fankor::traits::Instruction<'info>>::LPI
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
                    ::fankor::traits::LpiInstruction::serialize_into_instruction_parts(&self.#name, writer, metas)?;
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
                ::fankor::traits::LpiInstruction::serialize_into_instruction_parts(&self.#name, writer, metas)?;
            }
        }
    });

    // Validations.
    let initial_validation = &arguments.initial_validation.map(|v| match v {
        Validation::Implicit => {
            quote! {
                self.initial_validation(context)?;
            }
        }
        Validation::Explicit(v) => v,
    });

    let final_validation = &arguments.final_validation.map(|v| match v {
        Validation::Implicit => {
            quote! {
                self.final_validation(context)?;
            }
        }
        Validation::Explicit(v) => v,
    });

    // Result
    let result = quote! {
        #(#attributes)*
        #visibility struct #name #ty_generics #where_clause {
            #(#final_fields,)*
        }

        #[automatically_derived]
        impl #impl_generics ::fankor::traits::Instruction<'info> for #name #ty_generics #where_clause {
            type CPI = #cpi_name <'info>;
            type LPI = #lpi_name <'info>;

            fn try_from(
                context: &'info FankorContext<'info>,
                buf: &mut &[u8],
                accounts: &mut &'info [AccountInfo<'info>],
            ) -> ::fankor::errors::FankorResult<Self> {
                #(#try_from_fn_deserialize)*

                let result = Self {
                    #(#fields),*
                };

                // Validate instruction.
                result.validate(context)?;

                Ok(result)
            }
        }

        #[automatically_derived]
        impl #impl_generics #name #ty_generics #where_clause {
            fn validate(
                &self,
                context: &'info FankorContext<'info>,
            ) -> ::fankor::errors::FankorResult<()> {
                use ::fankor::traits::Instruction;

                #initial_validation

                #(#validate_method_fields)*

                #final_validation

                Ok(())
            }

            #(#pda_methods)*
        }

        #[automatically_derived]
        #visibility struct #cpi_name <'info> {
            #(#cpi_fields),*
        }

        #[automatically_derived]
        impl <'info> ::fankor::traits::CpiInstruction<'info> for #cpi_name <'info> {
            fn serialize_into_instruction_parts<W: std::io::Write>(
                &self,
                writer: &mut W,
                metas: &mut Vec<AccountMeta>,
                infos: &mut Vec<AccountInfo<'info>>,
            ) -> FankorResult<()> {
                use ::fankor::prelude::borsh::BorshSerialize;
                #(#cpi_fn_elements)*
                Ok(())
            }
        }

        #[automatically_derived]
        #visibility struct #lpi_name <'info> {
            #(#lpi_fields),*
        }

        #[automatically_derived]
        impl <'info> ::fankor::traits::LpiInstruction for #lpi_name <'info> {
            fn serialize_into_instruction_parts<W: std::io::Write>(
                &self,
                writer: &mut W,
                metas: &mut Vec<::fankor::prelude::solana_program::instruction::AccountMeta>
            ) -> ::fankor::errors::FankorResult<()> {
                use ::fankor::prelude::borsh::BorshSerialize;
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
    let case_converter = Converter::new().from_case(Case::Snake).to_case(Case::Camel);
    let ts_types = mapped_fields.iter().map(|v| {
        let ty = &v.ty;
        let field_name = case_converter.convert(v.name.to_string());
        let types_replacement_str = format!("_r_interface_types_{}_r_", v.name);
        let metas_replacement_str = format!("_r_interface_metas_{}_r_", v.name);
        let writable = v.writable.clone().unwrap_or(quote! { false });
        let signer = v.signer.clone().unwrap_or(quote! { false });

        type_replacements.push(quote! {
             .replace(#types_replacement_str, &< #ty as TsInstructionGen>::generate_type(registered_types))
        });
        metas_fields.push(metas_replacement_str.clone());

        let value_str = format!("{{}}.{}", field_name);
        metas_replacements.push(quote! {
             .replace(#metas_replacement_str, &< #ty as TsInstructionGen>::get_external_account_metas(Cow::Owned(format!(#value_str, value)), #signer, #writable))
        });

        format!("{}: {}", field_name, types_replacement_str)
    }).collect::<Vec<_>>();

    let ts_type = format!(
        "export interface {} {{ {} }};",
        name_str,
        ts_types.join(",")
    );

    let ts_metas = metas_fields.join("");
    let get_metas_of_replacement_str =
        format!("getMetasOf{}(_r_value_r_,accountMetas, writer);", name_str);
    let test_name = format_ident!("__ts_gen_test__instruction_accounts_{}", name_str);
    let test_name_str = test_name.to_string();
    let result = quote! {
        #result

        #[cfg(feature = "ts-gen")]
        #[automatically_derived]
        #[allow(non_snake_case)]
        mod #test_name {
            use super::*;
            use ::fankor::prelude::TsInstructionGen;
            use ::fankor::prelude::TsTypesCache;
            use std::borrow::Cow;

            #[automatically_derived]
            impl #impl_generics TsInstructionGen for #name #ty_generics #where_clause {
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

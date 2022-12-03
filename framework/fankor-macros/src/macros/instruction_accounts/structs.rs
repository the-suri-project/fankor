use quote::{format_ident, quote};
use syn::ItemStruct;

use crate::Result;

use crate::macros::instruction_accounts::arguments::InstructionArguments;
use crate::macros::instruction_accounts::field::{check_fields, Field};

pub fn process_struct(item: ItemStruct) -> Result<proc_macro::TokenStream> {
    let instruction_arguments = InstructionArguments::from(&item.attrs)?;
    let name = &item.ident;
    let vis = &item.vis;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();
    let ixn_args_type = instruction_arguments
        .args
        .map(|args| quote! { args: &#args })
        .unwrap_or(quote! {});

    let verify_fn_fields = item.fields.iter().map(|v| {
        let name = v.ident.as_ref().unwrap();
        quote! {
            self.#name.verify_account_infos(f)?;
        }
    });

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

        let mut conditions = Vec::new();

        if let Some(owner) = &v.owner {
            conditions.push(quote! {{
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
            conditions.push(quote! {{
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
            conditions.push(quote! {{
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
            conditions.push(quote! {{
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
            conditions.push(quote! {{
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
            conditions.push(quote! {{
                let rent_exempt = #rent_exempt;
                let lamports = info.lamports();
                let data_len = info.data_len();

                let rent: Rent = ::fankor::prelude::solana_program::sysvar::Sysvar::get().expect("Cannot access Rent Sysvar");
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
            conditions.push(quote! {{
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

            pda_methods.push(quote! {
                pub fn #pda_method_name(&self, context: &FankorContext<'info>, #ixn_args_type) -> FankorResult<Vec<u8>> {
                    let seeds = #pda;
                    let size = seeds.iter().fold(0, |acc, s| acc + s.len());
                    let mut buf = Vec::with_capacity(size + 1);
                    seeds.iter().for_each(|s| buf.extend_from_slice(s));

                    // Get bump.
                    let info = self.#name.info();
                    let bump = context.get_bump_seed_from_account(info).ok_or_else(|| FankorErrorCode::MissingPdaBumpSeed {
                        account: *self.#name.address()
                    })?;
                    buf.push(bump);

                    Ok(buf)
                }
            });

            let program_id = v.pda_program_id.clone().unwrap_or_else(|| quote! { context.program_id() });
            conditions.push(quote! {{
                let seeds = #pda;
                let program_id = #program_id;

                context.check_canonical_pda_with_program(info, &seeds, program_id)?;
            }});
        }

        for constraint in &v.constraints {
            conditions.push(quote! {{
                require!(#constraint, FankorErrorCode::AccountConstraintFailed {
                    account: #name_str,
                    constraint: stringify!(#constraint),
                });
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

        if !conditions.is_empty() {
            quote! {
                self.#name.verify_account_infos(&mut |info: &AccountInfo<'info>| {
                    #(#conditions)*

                    Ok(())
                })?;

                #min
                #max
            }
        } else {
            quote! {
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

            fn verify_account_infos<F>(&self, f: &mut F) -> ::fankor::errors::FankorResult<()>
            where
                F: FnMut(&AccountInfo<'info>) -> ::fankor::errors::FankorResult<()>,
            {
                #(#verify_fn_fields)*
                Ok(())
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
            pub(crate) fn validate(
                &self,
                context: &'info FankorContext<'info>,
                #ixn_args_type
            ) -> ::fankor::errors::FankorResult<()> {
                use ::fankor::traits::InstructionAccount;

                #(#try_from_fn_conditions)*

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

    Ok(result.into())
}

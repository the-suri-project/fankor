use quote::{format_ident, quote};
use syn::ItemStruct;

use fankor_syn::Result;

use crate::macros::instruction_accounts::field::{check_fields, Field, FieldKind};

pub fn process_struct(item: ItemStruct) -> Result<proc_macro::TokenStream> {
    let name = &item.ident;
    let vis = &item.vis;
    let generic_params = &item.generics.params;
    let generic_params = if generic_params.is_empty() {
        quote! {}
    } else {
        quote! { < #generic_params > }
    };
    let generic_where_clause = &item.generics.where_clause;

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

    let zero = quote! {0};
    let try_from_fn_deserialize = mapped_fields
        .iter()
        .map(|v| {
            let name = &v.name;
            let ty = &v.ty;

            if v.kind.is_vec() && v.max.is_some() {
                let min = v.min.as_ref().unwrap_or( &zero);
                let max = v.max.as_ref().unwrap();

                quote!{
                    let #name: #ty = ::fankor::try_from_vec_accounts_with_bounds(context, accounts, #min, #max)?;
                }
            } else{
                quote!{
                    let #name = <#ty as ::fankor::traits::InstructionAccount>::try_from(context, accounts)?;
                }
            }
        });
    let try_from_fn_conditions = mapped_fields.iter().map(|v| {
        let name = &v.name;
        let name_str = name.to_string();

        let mut conditions = Vec::new();

        if let Some(owner) = &v.owner {
            conditions.push(quote! {{
                let actual = info.owner;
                let expected = #owner;

                if actual != expected {
                    return Err(::fankor::errors::ErrorCode::AccountConstraintOwnerMismatch {
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
                    return Err(::fankor::errors::ErrorCode::AccountConstraintAddressMismatch {
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
                        return Err(::fankor::errors::ErrorCode::AccountConstraintNotInitialized {
                            account: #name_str,
                        }.into());
                    }
                } else if info.owner != &system_program::ID || info.lamports() > 0 {
                    return Err(::fankor::errors::ErrorCode::AccountConstraintInitialized {
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
                        return Err(::fankor::errors::ErrorCode::AccountConstraintNotWritable {
                            account: #name_str,
                        }.into());
                    }
                } else if info.is_writable {
                    return Err(::fankor::errors::ErrorCode::AccountConstraintWritable {
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
                        return Err(::fankor::errors::ErrorCode::AccountConstraintNotExecutable {
                            account: #name_str,
                        }.into());
                    }
                } else if info.executable {
                    return Err(::fankor::errors::ErrorCode::AccountConstraintExecutable {
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
                        return Err(::fankor::errors::ErrorCode::AccountConstraintNotRentExempt {
                            account: #name_str,
                        }.into());
                    }
                } else if is_rent_exempt {
                    return Err(::fankor::errors::ErrorCode::AccountConstraintRentExempt {
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
                        return Err(::fankor::errors::ErrorCode::AccountConstraintNotSigner {
                            account: #name_str,
                        }.into());
                    }
                } else if info.is_signer {
                    return Err(::fankor::errors::ErrorCode::AccountConstraintSigner {
                        account: #name_str,
                    }.into());
                }
            }});
        }

        let (min, max) = if v.kind != FieldKind::Rest {
            let min = if v.max.is_none() {
                v.min.as_ref().map(|min| {
                    quote! {{
                        let expected = #min;
                        let actual = #name.len();

                        if actual < expected {
                            return Err(::fankor::errors::ErrorCode::AccountConstraintMinimumMismatch {
                                actual,
                                expected,
                                account: #name_str,
                            }.into());
                        }
                    }}
                })
            } else {
                None
            };

            (min, None)
        } else {
            let min = v.min.as_ref().map(|min| {
                quote! {{
                    let expected = #min;
                    let actual = #name.len();

                    if actual < expected {
                        return Err(::fankor::errors::ErrorCode::AccountConstraintMinimumMismatch {
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
                    let actual = #name.len();

                    if actual > expected {
                        return Err(::fankor::errors::ErrorCode::AccountConstraintMaximumMismatch {
                            actual,
                            expected,
                            account: #name_str,
                        }.into());
                    }
                }}
            });

            (min, max)
        };

        if !conditions.is_empty() {
            quote! {
                #name.verify_account_infos(&mut |context: &FankorContext<'info>, info: &AccountInfo<'info>| {
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
            #name:<#ty as ::fankor::traits::InstructionAccount<'info>>::CPI
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
            #name:<#ty as ::fankor::traits::InstructionAccount<'info>>::LPI
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

        match &v.kind {
            FieldKind::Other => {
                quote! {
                    min_accounts += <#ty as ::fankor::traits::InstructionAccount>::min_accounts();
                }
            }
            FieldKind::Vec(ty) => {
                if let Some(min) = &v.min {
                    quote! {
                        min_accounts += #min * <#ty>::min_accounts();
                    }
                } else {
                    quote! {
                        min_accounts += <#ty as ::fankor::traits::InstructionAccount>::min_accounts();
                    }
                }
            }
            FieldKind::Rest => {
                if let Some(min) = &v.min {
                    quote! {
                        min_accounts += #min;
                    }
                } else {
                    quote! {
                        min_accounts += <#ty as ::fankor::traits::InstructionAccount>::min_accounts();
                    }
                }
            }
        }
    });

    // Result
    let result = quote! {
        #[automatically_derived]
        impl #generic_params ::fankor::traits::InstructionAccount<'info> for #name #generic_params #generic_where_clause {
            type CPI = #cpi_name <'info>;

            #[cfg(feature = "library")]
            type LPI = #lpi_name <'info>;

            fn min_accounts() -> usize {
                let mut min_accounts = 0;
                #(#min_accounts_fn_elements)*
                min_accounts
            }

            fn verify_account_infos<F>(&self, f: &mut F) -> ::fankor::errors::FankorResult<()>
            where
                F: FnMut(&FankorContext<'info>, &AccountInfo<'info>) -> ::fankor::errors::FankorResult<()>,
            {
                #(#verify_fn_fields)*
                Ok(())
            }

            fn try_from(
                context: &'info FankorContext<'info>,
                accounts: &mut &'info [AccountInfo<'info>],
            ) -> ::fankor::errors::FankorResult<Self> {
                #(#try_from_fn_deserialize)*
                #(#try_from_fn_conditions)*

                Ok(Self {
                    #(#fields),*
                })
            }
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
        #[cfg(feature = "library")]
        #vis struct #lpi_name <'info> {
            #(#lpi_fields),*
        }

        #[automatically_derived]
        #[cfg(feature = "library")]
        impl <'info> ::fankor::traits::LpiInstructionAccount for #lpi_name <'info> {
            fn to_account_metas(&self, metas: &mut Vec<::fankor::prelude::solana_program::instruction::AccountMeta>) -> ::fankor::errors::FankorResult<()> {
                #(#lpi_fn_elements)*
                Ok(())
            }
        }
    };

    Ok(result.into())
}

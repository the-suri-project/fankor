use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{Error, Item};

use fankor_syn::Result;

use crate::macros::instruction_accounts::field::Field;

mod field;
mod parser;

pub fn processor(input: Item) -> Result<proc_macro::TokenStream> {
    // Process input.
    let result = match &input {
        Item::Struct(item) => {
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

            let zero = quote! {0};
            let try_from_fn_deserialize =mapped_fields
                .iter()
                .map(|v| {
                    let name = &v.name;
                    let ty = &v.ty;

                    if v.max.is_some() {
                        let min = v.min.as_ref().unwrap_or( &zero);
                        let max = v.max.as_ref().unwrap();

                        quote!{
                            let #name: #ty = ::fankor::try_from_vec_accounts_with_bounds(context, accounts, #min, #max)?;
                        }
                    }else{
                        quote!{
                            let #name = <#ty as ::fankor::traits::InstructionAccount>::try_from(context, accounts)?;
                        }
                    }
                });
            let try_from_fn_conditions = mapped_fields.iter().filter_map(|v| {
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
                        }else if info.owner != &system_program::ID || info.lamports() > 0 {
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
                        }else if info.is_writable {
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
                        }else if info.executable {
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

                        let rent = Rent::get().expect("Cannot access Rent Sysvar");

                        let is_rent_exempt = rent.is_exempt(lamports, data_len);

                        if rent_exempt {
                            if !is_rent_exempt {
                                return Err(::fankor::errors::ErrorCode::AccountConstraintNotRentExempt {
                                    account: #name_str,
                                }.into());
                            }
                        }else if is_rent_exempt {
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
                        }else if info.is_signer {
                            return Err(::fankor::errors::ErrorCode::AccountConstraintSigner {
                                account: #name_str,
                            }.into());
                        }
                    }});
                }

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

                if !conditions.is_empty() {
                    Some(quote! {
                        #name.verify_account_infos(&mut |context: &FankorContext<'info>, info: &AccountInfo<'info>| {
                            #(#conditions)*

                            Ok(())
                        })?;

                        #min
                    })
                } else {
                    min
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
                let writable = if let Some(writable) = &v.writable {
                    quote! { Some(#writable) }
                } else {
                    quote! { None }
                };

                let signer = if let Some(signer) = &v.signer {
                    quote! { Some(#signer) }
                } else {
                    quote! { None }
                };

                quote! {
                    {
                        let from = metas.len();
                        ::fankor::traits::CpiInstructionAccount::to_account_metas_and_infos(&self.#name, metas, infos)?;
                        let to = metas.len();
                        let writable = #writable;
                        let signer = #signer;

                        for meta in &mut metas[from..to] {
                            if let Some(writable) = writable {
                                meta.is_writable = writable;
                            }

                            if let Some(signer) = signer {
                                meta.is_signer = signer;
                            }
                        }
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
                let writable = if let Some(writable) = &v.writable {
                    quote! { Some(#writable) }
                } else {
                    quote! { None }
                };

                let signer = if let Some(signer) = &v.signer {
                    quote! { Some(#signer) }
                } else {
                    quote! { None }
                };

                quote! {
                    {
                        let from = metas.len();
                        ::fankor::traits::LpiInstructionAccount::to_account_metas(&self.#name, metas)?;
                        let to = metas.len();
                        let writable = #writable;
                        let signer = #signer;

                        for meta in &mut metas[from..to] {
                            if let Some(writable) = writable {
                                meta.is_writable = writable;
                            }

                            if let Some(signer) = signer {
                                meta.is_signer = signer;
                            }
                        }
                    }
                }
            });

            // Result
            quote! {
                #[automatically_derived]
                impl #generic_params ::fankor::traits::InstructionAccount<'info> for #name #generic_params #generic_where_clause {
                    type CPI = #cpi_name <'info>;

                    #[cfg(feature = "library")]
                    type LPI = #lpi_name <'info>;

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
            }
        }
        Item::Enum(item) => {
            let name = &item.ident;
            let vis = &item.vis;
            let generic_params = &item.generics.params;
            let generic_params = if generic_params.is_empty() {
                quote! {}
            } else {
                quote! { < #generic_params > }
            };
            let generic_where_clause = &item.generics.where_clause;

            let verify_fn_fields = item.variants.iter().map(|v| {
                let variant_name = &v.ident;
                quote! {
                    #name::#variant_name(v) => v.verify_account_infos(f),
                }
            });

            let mapped_fields = item
                .variants
                .iter()
                .map(|v| Field::from_variant(v.clone()))
                .collect::<Result<Vec<Field>>>()?;

            let zero = quote! {0};
            let try_from_fn_deserialize =mapped_fields
                .iter()
                .map(|v| {
                    let variant_name = &v.name;
                    let variant_name_str = variant_name.to_string();
                    let ty = &v.ty;

                    let mut conditions = Vec::new();

                    if let Some(owner) = &v.owner {
                        conditions.push(quote! {{
                            let actual = info.owner;
                            let expected = #owner;

                            if actual != expected {
                                return Err(::fankor::errors::ErrorCode::AccountConstraintOwnerMismatch {
                                    actual: *actual,
                                    expected: *expected,
                                    account: #variant_name_str,
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
                                    account: #variant_name_str,
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
                                        account: #variant_name_str,
                                    }.into());
                                }
                            }else if info.owner != &system_program::ID || info.lamports() > 0 {
                                return Err(::fankor::errors::ErrorCode::AccountConstraintInitialized {
                                    account: #variant_name_str,
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
                                        account: #variant_name_str,
                                    }.into());
                                }
                            }else if info.is_writable {
                                return Err(::fankor::errors::ErrorCode::AccountConstraintWritable {
                                    account: #variant_name_str,
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
                                        account: #variant_name_str,
                                    }.into());
                                }
                            }else if info.executable {
                                return Err(::fankor::errors::ErrorCode::AccountConstraintExecutable {
                                    account: #variant_name_str,
                                }.into());
                            }
                        }});
                    }

                    if let Some(rent_exempt) = &v.rent_exempt {
                        conditions.push(quote! {{
                            let rent_exempt = #rent_exempt;
                            let lamports = info.lamports();
                            let data_len = info.data_len();

                            let rent = Rent::get().expect("Cannot access Rent Sysvar");

                            let is_rent_exempt = rent.is_exempt(lamports, data_len);

                            if rent_exempt {
                                if !is_rent_exempt {
                                    return Err(::fankor::errors::ErrorCode::AccountConstraintNotRentExempt {
                                        account: #variant_name_str,
                                    }.into());
                                }
                            }else if is_rent_exempt {
                                return Err(::fankor::errors::ErrorCode::AccountConstraintRentExempt {
                                    account: #variant_name_str,
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
                                        account: #variant_name_str,
                                    }.into());
                                }
                            }else if info.is_signer {
                                return Err(::fankor::errors::ErrorCode::AccountConstraintSigner {
                                    account: #variant_name_str,
                                }.into());
                            }
                        }});
                    }

                    let min = if v.max.is_none() {
                        v.min.as_ref().map(|min| {
                            quote! {{
                                let expected = #min;
                                let actual = v.len();

                                if actual < expected {
                                    return Err(::fankor::errors::ErrorCode::AccountConstraintMinimumMismatch {
                                        actual,
                                        expected,
                                        account: #variant_name_str,
                                    }.into());
                                }
                            }}
                        })
                    } else {
                        None
                    };

                    let conditions = if !conditions.is_empty() {
                        Some(quote! {
                            v.verify_account_infos(&mut |context: &FankorContext<'info>, info: &AccountInfo<'info>| {
                                #(#conditions)*

                                Ok(())
                            })?;

                            #min
                        })
                    } else {
                        min
                    };

                    if v.max.is_some() {
                        let min = v.min.as_ref().unwrap_or( &zero);
                        let max = v.max.as_ref().unwrap();

                        quote!{
                            let err = match ::fankor::try_from_vec_accounts_with_bounds(context, accounts, #min, #max) {
                                Ok(v) => {
                                    let v: #ty = v;

                                    #conditions

                                    return Ok(#name::#variant_name(v));
                                },
                                Err(e) => e,
                            };
                        }
                    } else {
                        quote!{
                            let err = match <#ty as ::fankor::traits::InstructionAccount>::try_from(context, accounts) {
                                Ok(v) => {
                                    #conditions
                                    return Ok(#name::#variant_name(v));
                                },
                                Err(e) => e,
                            };
                        }
                    }
                });

            // CpiInstructionAccount implementation
            let cpi_name = format_ident!("Cpi{}", name);
            let cpi_fields = mapped_fields.iter().map(|v| {
                let name = &v.name;
                let ty = &v.ty;

                quote! {
                    #name(<#ty as ::fankor::traits::InstructionAccount<'info>>::CPI)
                }
            });
            let cpi_fn_elements = mapped_fields
                .iter()
                .map(|v| {
                    let variant_name = &v.name;
                    let writable = if let Some(writable) = &v.writable {
                        quote! { Some(#writable) }
                    } else {
                        quote! { None }
                    };

                    let signer = if let Some(signer) = &v.signer {
                        quote! { Some(#signer) }
                    } else {
                        quote! { None }
                    };

                    quote!{
                        #cpi_name::#variant_name(v) => {
                            let from = metas.len();
                            ::fankor::traits::CpiInstructionAccount::to_account_metas_and_infos(v, metas, infos)?;
                            let to = metas.len();
                            let writable = #writable;
                            let signer = #signer;

                            for meta in &mut metas[from..to] {
                                if let Some(writable) = writable {
                                    meta.is_writable = writable;
                                }

                                if let Some(signer) = signer {
                                    meta.is_signer = signer;
                                }
                            }
                        }
                    }
                });

            // LpiInstructionAccount implementation
            let lpi_name = format_ident!("Lpi{}", name);
            let lpi_fields = mapped_fields.iter().map(|v| {
                let name = &v.name;
                let ty = &v.ty;

                quote! {
                    #name(<#ty as ::fankor::traits::InstructionAccount<'info>>::LPI)
                }
            });
            let lpi_fn_elements = mapped_fields.iter().map(|v| {
                let variant_name = &v.name;
                let writable = if let Some(writable) = &v.writable {
                    quote! { Some(#writable) }
                } else {
                    quote! { None }
                };

                let signer = if let Some(signer) = &v.signer {
                    quote! { Some(#signer) }
                } else {
                    quote! { None }
                };

                quote! {
                    #lpi_name::#variant_name(v) => {
                        let from = metas.len();
                        ::fankor::traits::LpiInstructionAccount::to_account_metas(v, metas)?;
                        let to = metas.len();
                        let writable = #writable;
                        let signer = #signer;

                        for meta in &mut metas[from..to] {
                            if let Some(writable) = writable {
                                meta.is_writable = writable;
                            }

                            if let Some(signer) = signer {
                                meta.is_signer = signer;
                            }
                        }
                    }
                }
            });

            // Result
            quote! {
                #[automatically_derived]
                impl #generic_params ::fankor::traits::InstructionAccount<'info> for #name #generic_params #generic_where_clause {
                    type CPI = #cpi_name <'info>;

                    #[cfg(feature = "library")]
                    type LPI = #lpi_name <'info>;

                    fn verify_account_infos<F>(&self, f: &mut F) -> ::fankor::errors::FankorResult<()>
                    where
                        F: FnMut(&FankorContext<'info>, &AccountInfo<'info>) -> ::fankor::errors::FankorResult<()>,
                    {
                        match self {
                            #(#verify_fn_fields)*
                        }
                    }

                    fn try_from(
                        context: &'info FankorContext<'info>,
                        accounts: &mut &'info [AccountInfo<'info>],
                    ) -> ::fankor::errors::FankorResult<Self> {
                        #(#try_from_fn_deserialize)*

                        Err(err)
                    }
                }

                #[automatically_derived]
                #vis enum #cpi_name <'info> {
                    #(#cpi_fields),*
                }

                #[automatically_derived]
                impl <'info> ::fankor::traits::CpiInstructionAccount<'info> for #cpi_name <'info> {
                    fn to_account_metas_and_infos(
                        &self,
                        metas: &mut Vec<AccountMeta>,
                        infos: &mut Vec<AccountInfo<'info>>,
                    ) -> FankorResult<()> {
                        match self {
                            #(#cpi_fn_elements),*
                        }

                        Ok(())
                    }
                }

                #[automatically_derived]
                #[cfg(feature = "library")]
                #vis enum #lpi_name <'info>{
                    #(#lpi_fields),*
                }

                #[automatically_derived]
                #[cfg(feature = "library")]
                impl <'info> ::fankor::traits::LpiInstructionAccount for #lpi_name <'info> {
                    fn to_account_metas(&self, metas: &mut Vec<::fankor::prelude::solana_program::instruction::AccountMeta>) -> ::fankor::errors::FankorResult<()> {
                        match self {
                            #(#lpi_fn_elements),*
                        }

                        Ok(())
                    }
                }
            }
        }
        _ => {
            return Err(Error::new(
                input.span(),
                "Instruction macro can only be applied to struct or enum declarations",
            ));
        }
    };

    Ok(result.into())
}

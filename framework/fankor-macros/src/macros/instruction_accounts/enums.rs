use crate::macros::instruction_accounts::arguments::{InstructionArguments, Validation};
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{Error, ItemEnum};

use crate::Result;

use crate::macros::instruction_accounts::field::{check_fields, Field, FieldKind};

pub fn process_enum(item: ItemEnum) -> Result<proc_macro::TokenStream> {
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
        .variants
        .iter()
        .map(|v| Field::from_variant(v.clone()))
        .collect::<Result<Vec<Field>>>()?;
    check_fields(&mapped_fields)?;

    let try_from_fn_deserialize = mapped_fields
        .iter()
        .map(|v| {
            let variant_name = &v.name;
            let ty = &v.ty;

            quote!{
                if accounts_len >= <#ty as ::fankor::traits::InstructionAccount>::min_accounts() {
                    let mut accounts_aux = *accounts;
                    match <#ty as ::fankor::traits::InstructionAccount>::try_from(context, &mut accounts_aux) {
                        Ok(v) => {
                            *accounts = accounts_aux;

                            return Ok(#name::#variant_name(v));
                        },
                        Err(e) => {
                            err = e;
                        }
                    }
                }
            }
        });

    let try_from_fn_conditions = mapped_fields
        .iter()
        .map(|v| {
            let variant_name = &v.name;

            let args = if ixn_args_type.is_empty() {
                quote! {}
            } else {
                quote! { args }
            };

            match &v.kind {
                // Rest is placed here because the instruction struct can be named like that.
                FieldKind::Other | FieldKind::Rest => Ok(quote! {
                    Self::#variant_name(v) => {
                        v.validate(context, #args)?;
                    }
                }),
                FieldKind::Option(_) => Ok(quote! {
                    Self::#variant_name(v) => {
                        if let Some(v) = v {
                            v.validate(context, #args)?;
                        }
                    }
                }),
                FieldKind::Vec(v) => Err(Error::new(
                    v.span(),
                    "Vec is not supported for instruction enums",
                )),
            }
        })
        .collect::<Result<Vec<_>>>()?;

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
                    #cpi_name::#variant_name(v) => {
                        let from = metas.len();
                        ::fankor::traits::CpiInstructionAccount::to_account_metas_and_infos(v, metas, infos)?;
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
                    #cpi_name::#variant_name(v) => ::fankor::traits::CpiInstructionAccount::to_account_metas_and_infos(v, metas, infos)?
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
                #lpi_name::#variant_name(v) => {
                    let from = metas.len();
                    ::fankor::traits::LpiInstructionAccount::to_account_metas(v, metas)?;
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
                #lpi_name::#variant_name(v) => ::fankor::traits::LpiInstructionAccount::to_account_metas(v, metas)?
            }
        }
    });

    // Min accounts.
    let min_accounts_fn_elements = mapped_fields.iter().map(|v| {
        let ty = &v.ty;

        match &v.kind {
            FieldKind::Other => {
                quote! {
                    min_accounts = min_accounts.min(<#ty as ::fankor::traits::InstructionAccount>::min_accounts());
                }
            }
            FieldKind::Option(ty) => {
                if let Some(min) = &v.min {
                    quote! {
                        min_accounts = min_accounts.min(#min * <#ty>::min_accounts());
                    }
                } else {
                    quote! {
                        min_accounts = min_accounts.min(<#ty as ::fankor::traits::InstructionAccount>::min_accounts());
                    }
                }
            }
            FieldKind::Vec(ty) => {
                if let Some(min) = &v.min {
                    quote! {
                        min_accounts = min_accounts.min(#min * <#ty>::min_accounts());
                    }
                } else {
                    quote! {
                        min_accounts = min_accounts.min(<#ty as ::fankor::traits::InstructionAccount>::min_accounts());
                    }
                }
            }
            FieldKind::Rest => {
                if let Some(min) = &v.min {
                    quote! {
                        min_accounts = min_accounts.min(#min);
                    }
                } else {
                    quote! {
                        min_accounts = min_accounts.min(<#ty as ::fankor::traits::InstructionAccount>::min_accounts());
                    }
                }
            }
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
    let min_accounts = if mapped_fields.is_empty() {
        0
    } else {
        usize::MAX
    };
    let result = quote! {
        #[automatically_derived]
        impl #impl_generics ::fankor::traits::InstructionAccount<'info> for #name #ty_generics #where_clause {
            type CPI = #cpi_name <'info>;
            type LPI = #lpi_name <'info>;

            fn min_accounts() -> usize {
                let mut min_accounts = #min_accounts;
                #(#min_accounts_fn_elements)*
                min_accounts
            }

            fn try_from(
                context: &'info FankorContext<'info>,
                accounts: &mut &'info [AccountInfo<'info>],
            ) -> ::fankor::errors::FankorResult<Self> {
                let mut err: ::fankor::errors::Error = ::fankor::errors::FankorErrorCode::NotEnoughAccountKeys.into();
                let accounts_len = accounts.len();

                #(#try_from_fn_deserialize)*

                Err(err)
            }
        }

        #[automatically_derived]
        impl #impl_generics #name #ty_generics #where_clause {
            pub fn validate(
                &self,
                context: &'info FankorContext<'info>,
                #ixn_args_type
            ) -> ::fankor::errors::FankorResult<()> {
                #initial_validation

                match self {
                    #(#try_from_fn_conditions)*
                }

                #final_validation

                Ok(())
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
        #vis enum #lpi_name <'info>{
            #(#lpi_fields),*
        }

        #[automatically_derived]
        impl <'info> ::fankor::traits::LpiInstructionAccount for #lpi_name <'info> {
            fn to_account_metas(&self, metas: &mut Vec<::fankor::prelude::solana_program::instruction::AccountMeta>) -> ::fankor::errors::FankorResult<()> {
                match self {
                    #(#lpi_fn_elements),*
                }

                Ok(())
            }
        }
    };

    Ok(result.into())
}

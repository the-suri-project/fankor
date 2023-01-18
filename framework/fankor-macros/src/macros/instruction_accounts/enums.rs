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

    // Implement TypeScript generation.
    let name_str = name.to_string();
    let mut type_replacements = Vec::new();
    let mut metas_replacements = Vec::new();
    let mut ts_type_names = Vec::new();
    let mut metas_fields = Vec::new();
    let ts_types = mapped_fields.iter().map(|v| {
        let name = format!("{}_{}", name_str, v.name);
        let ty = &v.ty;
        let types_replacement_str = format!("_r_interface_types_{}_r_", name);
        let metas_replacement_str = format!("_r_interface_metas_{}_r_", name);

        ts_type_names.push(name.clone());
        type_replacements.push(quote! {
             .replace(#types_replacement_str, &< #ty as TsInstructionAccountGen>::generate_type(registered_types))
        });
        metas_fields.push(format!("case '{}': {} break;", v.name, metas_replacement_str));
        metas_replacements.push(quote! {
             .replace(#metas_replacement_str, &< #ty as TsInstructionAccountGen>::get_external_account_metas(Cow::Owned(format!("{}.value", value)), false, false))
        });

        format!("export interface {} {{ type: '{}', value: {} }}", name, v.name, types_replacement_str)
    }).collect::<Vec<_>>();

    let ts_type = format!(
        "export type {} = {};\
        {}",
        name_str,
        ts_type_names.join("|"),
        ts_types.join("")
    );

    let ts_metas = format!(
        "switch (_r_value_r_.type) {{ {} default: throw new Error('Invalid account type'); }}",
        metas_fields.join(""),
    );

    let get_metas_of_replacement_str = format!("getMetasOf{}(_r_value_r_,accountMetas)", name_str);
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
                    Cow::Owned(#ts_metas.replace("_r_value_r_", &value) #(#metas_replacements)*)
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

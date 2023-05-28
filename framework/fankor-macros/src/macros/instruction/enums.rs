use quote::{format_ident, quote};
use syn::ItemEnum;

use crate::fnk_syn::FnkMetaArgumentList;
use crate::macros::instruction::arguments::{InstructionArguments, Validation};
use crate::macros::instruction::field::{check_fields, Field, FieldKind};
use crate::Result;

pub fn process_enum(args: FnkMetaArgumentList, item: ItemEnum) -> Result<proc_macro::TokenStream> {
    let arguments = InstructionArguments::from(args)?;
    let name = &item.ident;
    let name_str = name.to_string();
    let discriminant_name = format_ident!("{}Discriminant", name);
    let visibility = &item.vis;
    let attributes = &item.attrs;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    let mapped_fields = item
        .variants
        .iter()
        .map(|v| Field::from_variant(v.clone()))
        .collect::<Result<Vec<Field>>>()?;
    check_fields(&mapped_fields)?;

    let mut final_enum_variants = Vec::with_capacity(mapped_fields.len());
    let mut try_from_method_deserialize = Vec::with_capacity(mapped_fields.len());
    let mut variant_consts = Vec::with_capacity(mapped_fields.len());
    let mut validate_method_variants = Vec::with_capacity(mapped_fields.len());
    let mut discriminants = Vec::new();

    for mapped_field in &mapped_fields {
        let variant_name = &mapped_field.name;
        let ty = &mapped_field.ty;
        let attrs = &mapped_field.attrs;
        let const_name = format_ident!("{}Discriminant", variant_name);

        if let Some(ty) = ty {
            final_enum_variants.push(quote! {
                #(#attrs)*
                #variant_name(#ty)
            });

            try_from_method_deserialize.push(quote! {
                #const_name => {
                    let mut new_buf = &buf[1..];
                    let mut new_accounts = *accounts;
                    let result = <#ty as ::fankor::traits::Instruction>::try_from(context, &mut new_buf, &mut new_accounts)?;

                    *accounts = new_accounts;
                    *buf = new_buf;

                    #name::#variant_name(result)
                }
            });

            validate_method_variants.push(match &mapped_field.kind {
                // Rest is placed here because the instruction struct can be named like that.
                FieldKind::Other | FieldKind::Rest => quote! {
                    Self::#variant_name(v) => {
                        v.validate(context)?;
                    }
                },
                FieldKind::Option(_) => quote! {
                    Self::#variant_name(v) => {
                        if let Some(v) = v {
                            v.validate(context)?;
                        }
                    }
                },
                FieldKind::Vec(_) => quote! {
                    Self::#variant_name(v) => {
                        for v2 in v {
                            v2.validate(context)?;
                        }
                    }
                },
            });
        } else {
            final_enum_variants.push(quote! {
                #(#attrs)*
                #variant_name
            });

            try_from_method_deserialize.push(quote! {
                #const_name => #name::#variant_name,
            });

            validate_method_variants.push(quote! {
                Self::#variant_name => {}
            });
        }

        variant_consts.push(quote! {
            pub const #const_name: u8 = #discriminant_name::#variant_name.code();
        });

        discriminants.push(quote!(
            Self::#variant_name{..} => #discriminant_name::#variant_name
        ));
    }

    // CpiInstruction implementation
    let cpi_name = format_ident!("Cpi{}", name);
    let cpi_fields = mapped_fields.iter().map(|v| {
        let name = &v.name;
        let ty = &v.ty;

        if let Some(ty) = ty {
            quote! {
                #name(<#ty as ::fankor::traits::Instruction<'info>>::CPI)
            }
        } else {
            quote! {
                #name
            }
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

            if v.ty.is_some() {
                if any {
                    quote! {
                        #cpi_name::#variant_name(v) => {
                            let from = metas.len();
                            ::fankor::traits::CpiInstruction::serialize_into_instruction_parts(v, writer, metas, infos)?;
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
                        #cpi_name::#variant_name(v) => ::fankor::traits::CpiInstruction::serialize_into_instruction_parts(v, writer, metas, infos)?
                    }
                }
            } else {
                quote! {
                    #cpi_name::#variant_name => {}
                }
            }
        });

    // LpiInstruction implementation
    let lpi_name = format_ident!("Lpi{}", name);
    let lpi_fields = mapped_fields.iter().map(|v| {
        let name = &v.name;
        let ty = &v.ty;

        if let Some(ty) = ty {
            quote! {
                #name(<#ty as ::fankor::traits::Instruction<'info>>::LPI)
            }
        } else {
            quote! {
                #name
            }
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

        if v.ty.is_some() {
            if any {
                quote! {
                    #lpi_name::#variant_name(v) => {
                        let from = metas.len();
                        ::fankor::traits::LpiInstruction::serialize_into_instruction_parts(v, writer, metas)?;
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
                    #lpi_name::#variant_name(v) => ::fankor::traits::LpiInstruction::serialize_into_instruction_parts(v, writer, metas)?
                }
            }
        } else {
            quote! {
                #lpi_name::#variant_name => {}
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
        #[derive(EnumDiscriminants)]
        #[non_exhaustive]
        #[repr(u8)]
        #(#attributes)*
        #visibility enum #name #ty_generics #where_clause {
            #(#final_enum_variants,)*
        }

        #[automatically_derived]
        impl #impl_generics ::fankor::traits::Instruction<'info> for #name #ty_generics #where_clause {
            type CPI = #cpi_name <'info>;
            type LPI = #lpi_name <'info>;

            #[allow(non_upper_case_globals)]
            fn try_from(
                context: &'info FankorContext<'info>,
                buf: &mut &[u8],
                accounts: &mut &'info [AccountInfo<'info>],
            ) -> ::fankor::errors::FankorResult<Self> {
                if buf.is_empty() {
                    return Err(FankorErrorCode::NotEnoughDataToDeserializeInstruction.into());
                }

                #(#variant_consts)*
                let result = match buf[0] {
                    #(#try_from_method_deserialize)*
                    _ => {
                        return Err(FankorErrorCode::InstructionDidNotDeserialize {
                            account: #name_str.to_string(),
                        }
                        .into())
                    }
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

                match self {
                    #(#validate_method_variants)*
                }

                #final_validation

                Ok(())
            }
        }

        #[automatically_derived]
        #visibility enum #cpi_name <'info> {
            #(#cpi_fields),*
        }

        #[automatically_derived]
        impl <'info> #cpi_name <'info> {
            pub const fn discriminant(&self) -> #discriminant_name {
                match self {
                    #(#discriminants,)*
                }
            }
        }

        #[automatically_derived]
        impl <'info> ::fankor::traits::CpiInstruction<'info> for #cpi_name <'info> {
            fn serialize_into_instruction_parts<W: std::io::Write>(
                &self,
                writer: &mut W,
                metas: &mut Vec<AccountMeta>,
                infos: &mut Vec<AccountInfo<'info>>,
            ) -> FankorResult<()> {
                use ::fankor::prelude::BorshSerialize;

                self.discriminant().code().serialize(writer)?;

                match self {
                    #(#cpi_fn_elements),*
                }

                Ok(())
            }
        }

        #[automatically_derived]
        #visibility enum #lpi_name <'info>{
            #(#lpi_fields),*
        }

        #[automatically_derived]
        impl <'info> #lpi_name <'info> {
            pub const fn discriminant(&self) -> #discriminant_name {
                match self {
                    #(#discriminants,)*
                }
            }
        }

        #[automatically_derived]
        impl <'info> ::fankor::traits::LpiInstruction for #lpi_name <'info> {
            fn serialize_into_instruction_parts<W: std::io::Write>(
                &self,
                writer: &mut W,
                metas: &mut Vec<::fankor::prelude::solana_program::instruction::AccountMeta>
            ) -> ::fankor::errors::FankorResult<()> {
                use ::fankor::prelude::BorshSerialize;

                self.discriminant().code().serialize(writer)?;

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
        let variant_name = &v.name;
        let name = format!("{}_{}", name_str, v.name);
        let ty = &v.ty;
        let types_replacement_str = format!("_r_interface_types_{}_r_", name);
        let metas_replacement_str = format!("_r_interface_metas_{}_r_", name);

        ts_type_names.push(name.clone());

        if let Some(ty) = ty {
            type_replacements.push(quote! {
                 .replace(#types_replacement_str, &< #ty as TsInstructionGen>::generate_type(registered_types))
            });
            metas_fields.push(format!("case '{}': writer.writeByte({}.{}); {} break;", v.name, discriminant_name, variant_name, metas_replacement_str));
            metas_replacements.push(quote! {
                 .replace(#metas_replacement_str, &< #ty as TsInstructionGen>::get_external_account_metas(Cow::Owned(format!("{}.value", value)), false, false))
            });

            format!("export interface {} {{ type: '{}', value: {} }}", name, v.name, types_replacement_str)
        } else {
            metas_fields.push(format!("case '{}': writer.writeByte({}.{}); break;", v.name, discriminant_name, variant_name));

            format!("export interface {} {{ type: '{}' }}", name, v.name)
        }
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

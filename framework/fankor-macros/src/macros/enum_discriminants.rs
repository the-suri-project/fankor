use std::collections::HashSet;
use std::fmt::Display;
use std::str::FromStr;

use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{Attribute, Error, Fields, Item, Meta, Variant};

use crate::utils::{unwrap_int_from_literal, unwrap_lit_from_expression};
use crate::Result;

pub fn processor(input: Item) -> Result<proc_macro::TokenStream> {
    // Process input.
    let result = match &input {
        Item::Enum(item) => {
            let visibility = &item.vis;
            let name = &item.ident;
            let discriminant_name = format_ident!("{}Discriminant", item.ident);

            let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

            let mut fields = Vec::new();
            let mut codes = Vec::new();
            let mut discriminants = Vec::new();
            let mut variant_idx = 0u8;
            let mut used_discriminants = HashSet::new();
            let mut is_last_deprecated = false;

            for variant in item.variants.iter() {
                let variant_ident = &variant.ident;

                fields.push(quote! {
                    #variant_ident
                });

                let is_deprecated = is_deprecated(&variant.attrs);
                let discriminant = get_discriminant(variant)?;

                // Calculate the discriminant.
                if let Some(v) = discriminant {
                    if v < variant_idx {
                        return Err(Error::new(
                            variant.span(),
                            "Variants must always be sorted from lowest to highest discriminant",
                        ));
                    }

                    variant_idx = v;
                } else if is_last_deprecated {
                    return Err(Error::new(
                        variant.span(),
                        format!(
                            "After a deprecated entity you must explicitly define the variant discriminant: = {}",
                            variant_idx
                        ),
                    ));
                }

                if used_discriminants.contains(&variant_idx) {
                    return Err(Error::new(
                        variant.span(),
                        format!(
                            "The discriminant attribute is already in use: {}",
                            variant_idx
                        ),
                    ));
                }

                used_discriminants.insert(variant_idx);

                match &variant.fields {
                    Fields::Named(_) => {
                        discriminants.push(quote!(
                            Self::#variant_ident{..} => #discriminant_name::#variant_ident
                        ));
                    }
                    Fields::Unnamed(_) => {
                        discriminants.push(quote!(
                            Self::#variant_ident(..) => #discriminant_name::#variant_ident
                        ));
                    }
                    Fields::Unit => {
                        discriminants.push(quote!(
                            Self::#variant_ident => #discriminant_name::#variant_ident
                        ));
                    }
                }

                codes.push(quote!(
                    Self::#variant_ident => #variant_idx
                ));

                variant_idx += 1;
                is_last_deprecated = is_deprecated;
            }

            let result = quote! {
                #[allow(dead_code)]
                #[automatically_derived]
                #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
                #[non_exhaustive]
                #[repr(u8)]
                #visibility enum #discriminant_name {
                    #(#fields,)*
                }

                #[automatically_derived]
                impl #discriminant_name {
                    pub const fn code(&self) -> u8 {
                        match self {
                            #(#codes,)*
                        }
                    }
                }

                #[automatically_derived]
                impl #impl_generics #name #ty_generics #where_clause {
                    pub const fn discriminant(&self) -> #discriminant_name {
                        match self {
                            #(#discriminants,)*
                        }
                    }
                }
            };

            // TypeScript generation.
            let mut ts_enum_replacements = Vec::new();
            let ts_discriminants = fields
                .iter()
                .map(|field| {
                    let replacement_str = format!("_r_{}_r_", field);

                    ts_enum_replacements.push(quote! {
                        .replace(#replacement_str, &#discriminant_name::#field.code().to_string())
                    });

                    format!("{} = {},", field, replacement_str)
                })
                .collect::<Vec<_>>();

            let ts_enum = format!(
                "export enum {} {{
                    {}
                }}",
                discriminant_name,
                ts_discriminants.join("\n"),
            );

            let discriminant_name_str = discriminant_name.to_string();
            let test_name = format_ident!("__ts_gen_test__account_offset_{}", discriminant_name);
            let test_name_str = test_name.to_string();
            let result = quote! {
                #result

                #[cfg(feature = "ts-gen")]
                #[automatically_derived]
                #[allow(non_snake_case)]
                mod #test_name {
                    use super::*;

                    #[test]
                    fn build() {
                         // Register action.
                        crate::__ts_gen_test__setup::BUILD_CONTEXT.register_action(#test_name_str, file!(), move |action_context| {
                            let ts_enum = #ts_enum .to_string() #(#ts_enum_replacements)*;
                            action_context.add_created_type(#discriminant_name_str, std::borrow::Cow::Owned(ts_enum)).unwrap();
                        })
                    }
                }
            };

            result
        }
        _ => {
            return Err(Error::new(
                input.span(),
                "EnumDiscriminants macro can only be applied to enum declarations",
            ));
        }
    };

    Ok(result.into())
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub fn is_deprecated(attrs: &[Attribute]) -> bool {
    for attr in attrs.iter() {
        if let Meta::Path(path) = &attr.meta {
            if path.is_ident("deprecated") {
                return true;
            }
        }
    }
    false
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub fn get_discriminant<N>(variant: &Variant) -> Result<Option<N>>
where
    N: FromStr,
    N::Err: Display,
{
    if variant.discriminant.is_some() {
        return Err(Error::new(
            variant.span(),
            "Native discriminants not yet supported in BPF compiler",
        ));
    }

    let mut discriminant = None;
    for attr in variant.attrs.iter() {
        if let Meta::NameValue(name_value) = &attr.meta {
            if name_value.path.is_ident("discriminant") {
                if discriminant.is_some() {
                    return Err(Error::new(
                        variant.span(),
                        "Only one discriminant attribute is allowed",
                    ));
                }

                let literal =
                    unwrap_int_from_literal(unwrap_lit_from_expression(name_value.value.clone())?)?;
                discriminant = Some(literal.base10_parse()?);
            }
        }
    }

    Ok(discriminant)
}

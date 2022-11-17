use quote::{format_ident, quote};
use std::collections::HashSet;
use syn::spanned::Spanned;
use syn::{AttributeArgs, Error, Fields, Item};

use fankor_syn::Result;

use crate::macros::error::arguments::ErrorArguments;
use crate::macros::error::variant::ErrorVariant;

mod arguments;
mod variant;

pub fn processor(args: AttributeArgs, input: Item) -> Result<proc_macro::TokenStream> {
    // Process input.
    let enum_item = match input {
        Item::Enum(v) => v,
        _ => {
            return Err(Error::new(
                input.span(),
                "error_code macro can only be applied to enum declarations",
            ));
        }
    };

    // Process arguments.
    let attributes = ErrorArguments::from(args, &enum_item)?;

    let name = enum_item.ident;
    let attrs = &attributes.attrs;

    // Parse fields taking into account whether any variant is deprecated or not.
    let mut last_deprecated = false;
    let variants = enum_item
        .variants
        .into_iter()
        .map(|v| {
            let result = ErrorVariant::from(v)?;

            if last_deprecated && result.code.is_none() {
                return Err(Error::new(
                    result.name.span(),
                    "The next error after a deprecated one must have the #[code] attribute",
                ));
            }

            last_deprecated = result.deprecated;

            Ok(result)
        })
        .collect::<Result<Vec<_>>>()?;

    let visibility = enum_item.vis;
    let generic_params = &enum_item.generics.params;
    let generic_params = if generic_params.is_empty() {
        quote! {}
    } else {
        quote! { < #generic_params > }
    };
    let generic_where_clause = &enum_item.generics.where_clause;

    // Generate code.
    let final_enum_variants = variants.iter().map(|v| {
        let ErrorVariant {
            name,
            attributes,
            fields,
            ..
        } = &v;

        quote! {
            #(#attributes)*
            #name #fields
        }
    });

    let name_fn_variants = variants.iter().map(|v| {
        let ErrorVariant {
            name: variant_name,
            fields,
            ..
        } = &v;
        let variant_name_str = variant_name.to_string();

        match fields {
            Fields::Named(_) => quote! {
                #name::#variant_name{..} => #variant_name_str
            },
            Fields::Unnamed(_) => quote! {
                #name::#variant_name(..) => #variant_name_str
            },
            Fields::Unit => quote! {
                #name::#variant_name => #variant_name_str
            },
        }
    });

    let offset = match &attributes.offset {
        Some(offset) => quote! { + #offset },
        None => quote! { + 6000 },
    };

    let mut u32_index = 0u32;
    let mut used_codes = HashSet::new();
    let mut codes = Vec::with_capacity(variants.len());
    let from_u32_fn_variants = variants
        .iter()
        .map(|v| {
            let span = v.name.span();
            let ErrorVariant {
                name: variant_name,
                fields,
                code,
                ..
            } = &v;
            let discriminant = {
                if let Some(v) = code {
                    let new_value = v.base10_parse::<u32>()?;

                    u32_index = new_value;
                }

                if used_codes.contains(&u32_index) {
                    return Err(Error::new(
                        span,
                        format!("The code attribute is already in use: {}", u32_index),
                    ));
                }

                used_codes.insert(u32_index);

                if attributes.contains_removed_code(u32_index) {
                    return Err(Error::new(
                        name.span(),
                        format!("The discriminator '{}' is marked as removed", u32_index),
                    ));
                }

                let result = quote! { #u32_index };
                u32_index += 1;
                result
            };

            let discriminant_field_name = format!("{}::{}", name, variant_name);
            codes.push(quote! {
                (#discriminant_field_name, #discriminant)
            });

            Ok(match fields {
                Fields::Named(_) => quote! {
                    #name::#variant_name{..} => (#discriminant) #offset
                },
                Fields::Unnamed(_) => quote! {
                    #name::#variant_name(..) => (#discriminant) #offset
                },
                Fields::Unit => quote! {
                    #name::#variant_name => (#discriminant) #offset
                },
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let display_fn_variants = variants.iter().map(|v| {
        let ErrorVariant {
            name: variant_name,
            fields,
            message,
            ..
        } = &v;
        let message = match message {
            Some(v) => quote! { #v },
            None => quote! {""},
        };

        match fields {
            Fields::Named(v) => {
                let names = v.named.iter().map(|v| &v.ident);

                quote! {
                    #name::#variant_name{#(#names),*} => write!(fmt, #message)
                }
            }
            Fields::Unnamed(v) => {
                let names = v
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(i, v)| format_ident!("v{}", i, span = v.span()));

                quote! {
                    #name::#variant_name(#(#names),*) => write!(fmt, #message)
                }
            }
            Fields::Unit => quote! {
                #name::#variant_name => write!(fmt, #message)
            },
        }
    });

    let result = quote! {
        #[derive(::std::fmt::Debug, ::std::clone::Clone)]
        #[repr(u32)]
        #(#attrs)*
        #visibility enum #name #generic_params #generic_where_clause {
            #(#final_enum_variants,)*
        }

        #[automatically_derived]
        impl #generic_params #name #generic_params #generic_where_clause {
            pub fn name(&self) -> &'static str {
                match self {
                    #(#name_fn_variants),*
                }
            }

            pub fn message(&self) -> String {
                format!("{}", self)
            }

            pub fn error_code(&self) -> u32 {
                match self {
                    #(#from_u32_fn_variants,)*
                }
            }
        }

        #[automatically_derived]
        impl #generic_params ::std::fmt::Display for #name #generic_params #generic_where_clause {
            fn fmt(&self, fmt: &mut ::std::fmt::Formatter<'_>) -> ::std::result::Result<(), ::std::fmt::Error> {
                match self {
                    #(#display_fn_variants),*
                }
            }
        }

        #[automatically_derived]
        impl #generic_params From<#name #generic_params> for fankor::errors::Error #generic_where_clause {
            fn from(error_code: #name #generic_params) -> fankor::errors::Error {
                fankor::errors::Error::from(fankor::errors::FankorError {
                    error_name: error_code.name().to_string(),
                    error_code_number: error_code.error_code(),
                    error_msg: error_code.to_string(),
                })
            }
        }
    };

    Ok(result.into())
}

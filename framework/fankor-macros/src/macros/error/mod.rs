use quote::{format_ident, quote};
use std::collections::HashSet;
use syn::spanned::Spanned;
use syn::{Error, Fields, Item};

use crate::Result;

use crate::fnk_syn::FnkMetaArgumentList;
use crate::macros::error::arguments::ErrorArguments;
use crate::macros::error::variant::ErrorVariant;

mod arguments;
mod variant;

pub fn processor(args: FnkMetaArgumentList, input: Item) -> Result<proc_macro::TokenStream> {
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
    let attributes = ErrorArguments::from(args)?;

    let name = enum_item.ident;
    let discriminant_name = format_ident!("{}Discriminant", name);

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
    let (impl_generics, ty_generics, where_clause) = enum_item.generics.split_for_impl();

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
    let mut discriminant_fields = Vec::with_capacity(variants.len());
    let mut discriminant_codes = Vec::with_capacity(variants.len());
    let mut discriminant_maps = Vec::with_capacity(variants.len());

    for v in &variants {
        let span = v.name.span();
        let ErrorVariant {
            name: variant_name,
            fields,
            code,
            ..
        } = &v;
        let discriminant = {
            if let Some(v) = code {
                if *v < u32_index {
                    return Err(Error::new(
                        span,
                        "Errors must always be sorted from lowest to highest discriminant",
                    ));
                }

                u32_index = *v;
            }

            if used_codes.contains(&u32_index) {
                return Err(Error::new(
                    span,
                    format!("The code attribute is already in use: {}", u32_index),
                ));
            }

            used_codes.insert(u32_index);

            let result = quote! { #u32_index };
            u32_index += 1;
            result
        };

        discriminant_codes.push(quote! {
            Self::#variant_name => (#discriminant) #offset
        });

        discriminant_fields.push(quote! {
            #variant_name
        });

        match fields {
            Fields::Named(_) => {
                discriminant_maps.push(quote! {
                    Self::#variant_name{..} =>#discriminant_name::#variant_name
                });
            }
            Fields::Unnamed(_) => {
                discriminant_maps.push(quote! {
                    Self::#variant_name(..) =>#discriminant_name::#variant_name
                });
            }
            Fields::Unit => {
                discriminant_maps.push(quote! {
                    Self::#variant_name =>#discriminant_name::#variant_name
                });
            }
        }
    }

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

    let ts_gen = if attributes.skip_ts_gen {
        quote! {}
    } else {
        quote! {
            #[derive(TsGen)]
        }
    };

    let result = quote! {
        #[derive(::std::fmt::Debug, ::std::clone::Clone)]
        #[repr(u32)]
        #ts_gen
        #[non_exhaustive]
        #visibility enum #name #ty_generics #where_clause {
            #(#final_enum_variants,)*
        }

        #[automatically_derived]
        impl #impl_generics #name #ty_generics #where_clause {
            pub fn name(&self) -> &'static str {
                match self {
                    #(#name_fn_variants),*
                }
            }

            pub fn message(&self) -> String {
                format!("{}", self)
            }

            #[inline(always)]
            pub const fn discriminant(&self) -> #discriminant_name {
                match self {
                    #(#discriminant_maps,)*
                }
            }

            pub fn error_code(&self) -> u32 {
                self.discriminant().code()
            }
        }

        #[automatically_derived]
        impl #impl_generics ::std::fmt::Display for #name #ty_generics #where_clause {
            fn fmt(&self, fmt: &mut ::std::fmt::Formatter<'_>) -> ::std::result::Result<(), ::std::fmt::Error> {
                match self {
                    #(#display_fn_variants),*
                }
            }
        }

        #[automatically_derived]
        impl #impl_generics From<#name #ty_generics> for fankor::errors::Error #where_clause {
            fn from(error_code: #name #ty_generics) -> fankor::errors::Error {
                fankor::errors::Error::from(fankor::errors::FankorError {
                    error_name: error_code.name().to_string(),
                    error_code_number: error_code.error_code(),
                    error_msg: error_code.to_string(),
                })
            }
        }

        #[allow(dead_code)]
        #[automatically_derived]
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
        #[non_exhaustive]
        #[repr(u32)]
        #visibility enum #discriminant_name {
            #(#discriminant_fields,)*
        }

        #[automatically_derived]
        impl #discriminant_name {
            #[inline(always)]
            pub const fn code(&self) -> u32 {
                match self {
                    #(#discriminant_codes,)*
                }
            }
        }
    };

    Ok(result.into())
}

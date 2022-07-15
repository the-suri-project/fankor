use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{AttributeArgs, Error, Fields, Item};

use fankor_syn::Result;

use crate::macros::error::attributes::ErrorAttributes;
use crate::macros::error::variant::ErrorVariant;

mod attributes;
mod variant;

pub fn processor(args: AttributeArgs, input: Item) -> Result<proc_macro::TokenStream> {
    // Process arguments.
    let attributes = ErrorAttributes::from(args)?;

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

    let name = enum_item.ident;

    // Parse fields.
    let variants = enum_item
        .variants
        .into_iter()
        .map(ErrorVariant::from)
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

    let offset = match attributes.offset {
        Some(offset) => quote! { + #offset },
        None => quote! { + 6000 },
    };

    let mut named_args = false;
    let mut u32_index = 0u32;
    let mut discriminators = Vec::with_capacity(variants.len());
    let from_u32_fn_variants = variants.iter().map(|v| {
        let ErrorVariant {
            name: variant_name,
            fields,
            discriminant,
            continue_from,
            ..
        } = &v;
        let discriminant = match discriminant {
            Some(v) => {named_args = true; quote! {#v} },
            None => {
                if named_args {
                    return Err(Error::new(
                        variant_name.span(),
                        "The variant without a discriminator are not allowed after another one that has it",
                    ));
                }

                if let Some(v) = continue_from {
                    let new_value = v.base10_parse::<u32>()?;

                    if new_value < u32_index {
                        return Err(Error::new(
                            v.span(),
                            format!("The continue_from attribute cannot be lower than the current one: {}", u32_index),
                        ));
                    }

                    u32_index = new_value;
                }


                let result = quote! { #u32_index};
                u32_index += 1;
                result
            }
        };

        let discriminant_field_name = format!("{}::{}", name, variant_name);
        discriminators.push(quote! {
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
    }).collect::<Result<Vec<_>>>()?;

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

    let test_unique_variant_error_codes = format_ident!(
        "__fankor_internal__test__unique_variant_error_codes_{}",
        name
    );

    let result = quote! {
        #[derive(::std::fmt::Debug, ::std::clone::Clone)]
        #[repr(u32)]
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

        #[allow(non_snake_case)]
        #[automatically_derived]
        #[cfg(test)]
        #[test]
        fn #test_unique_variant_error_codes() {
            let discriminators = [#(#discriminators),*];
            let helper = &crate::__internal__idl_builder_test__root::ERROR_HELPER;

            for (name, discriminator) in discriminators {
                if let Err(item) = helper.add_error(name, discriminator) {
                    panic!("There is a discriminator collision between errors. First: {}, Second: {}, Discriminator: {}", name, item.name, discriminator);
                }
            }
        }
    };

    Ok(result.into())
}

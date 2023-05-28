use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Error, Fields, Ident, ItemStruct};

use crate::fnk_syn::FnkMetaArgumentList;
use crate::macros::deserialize::enums::{contains_initialize_with, contains_skip};

pub fn struct_de(input: &ItemStruct, crate_name: Ident) -> syn::Result<TokenStream2> {
    let name = &input.ident;

    // Check for fankor attribute.
    let mut account_discriminants = None;

    for attr in &input.attrs {
        if attr.path().is_ident("fankor") {
            if let Ok(mut args) = attr.parse_args::<FnkMetaArgumentList>() {
                args.error_on_duplicated()?;

                account_discriminants = args.pop_ident("account", true)?;

                if args.pop_plain("accounts", true)? {
                    return Err(Error::new(
                        input.span(),
                        "Accounts cannot be used with an struct",
                    ));
                }

                args.error_on_unknown()?;
            } else {
                return Err(Error::new(
                    attr.span(),
                    "The correct pattern is #[fankor(<meta_list>)]",
                ));
            };
            break;
        }
    }

    let account_discriminants = if let Some(account_discriminants) = account_discriminants {
        let message = format!("Invalid discriminant for enum variant {}", name);
        quote! {
            let discriminant:u8 = #crate_name::BorshDeserialize::deserialize(buf)?;
            if discriminant != #account_discriminants::#name.code() {
                return Err(
                    std::io::Error::new(std::io::ErrorKind::Other, #message)
                );
            }
        }
    } else {
        quote! {}
    };

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let init_method = contains_initialize_with(&input.attrs)?;
    let return_value = match &input.fields {
        Fields::Named(fields) => {
            let mut body = TokenStream2::new();
            for field in &fields.named {
                let field_name = field.ident.as_ref().unwrap();

                let delta = if contains_skip(&field.attrs) {
                    quote! {
                        #field_name: Default::default(),
                    }
                } else {
                    quote! {
                        #field_name: #crate_name::BorshDeserialize::deserialize(buf)?,
                    }
                };
                body.extend(delta);
            }
            quote! {
                Self { #body }
            }
        }
        Fields::Unnamed(fields) => {
            let mut body = TokenStream2::new();
            for _ in 0..fields.unnamed.len() {
                let delta = quote! {
                    #crate_name::BorshDeserialize::deserialize(buf)?,
                };
                body.extend(delta);
            }
            quote! {
                Self( #body )
            }
        }
        Fields::Unit => {
            quote! {
                Self {}
            }
        }
    };

    let init_method = if let Some(method_ident) = init_method {
        quote! {
            return_value.#method_ident();
        }
    } else {
        quote! {}
    };

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #crate_name::de::BorshDeserialize for #name #ty_generics #where_clause {
            fn deserialize(buf: &mut &[u8]) -> ::core::result::Result<Self, #crate_name::maybestd::io::Error> {
                #account_discriminants

                let mut return_value = #return_value;
                #init_method
                Ok(return_value)
            }
        }
    })
}

use crate::Result;
use std::collections::HashSet;

use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, Attribute, Error, Fields, Ident, ItemEnum, Lit, Meta, MetaList, NestedMeta,
    Path, WhereClause,
};

pub fn enum_de(input: &ItemEnum, crate_name: Ident) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let mut where_clause = where_clause.map_or_else(
        || WhereClause {
            where_token: Default::default(),
            predicates: Default::default(),
        },
        Clone::clone,
    );
    let init_method = contains_initialize_with(&input.attrs)?;
    let mut variant_arms = TokenStream2::new();
    let mut variant_idx = 0u8;
    let mut used_discriminants = HashSet::new();
    let mut is_last_deprecated = false;

    for variant in input.variants.iter() {
        let variant_ident = &variant.ident;
        let mut variant_header = TokenStream2::new();

        let is_deprecated = is_deprecated(&variant.attrs);
        let discriminant = get_discriminant(&variant.attrs)?;

        // Calculate the discriminant.
        if let Some(v) = discriminant {
            variant_idx = v;
        } else if is_last_deprecated {
            return Err(Error::new(
                variant.span(),
                format!(
                    "After a deprecated entity you must explicitly define the variant discriminant #[discriminant(u8)]: {}",
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
            Fields::Named(fields) => {
                for field in &fields.named {
                    let field_name = field.ident.as_ref().unwrap();
                    if contains_skip(&field.attrs) {
                        variant_header.extend(quote! {
                            #field_name: Default::default(),
                        });
                    } else {
                        let field_type = &field.ty;
                        where_clause.predicates.push(
                            syn::parse2(quote! {
                                #field_type: #crate_name::BorshDeserialize
                            })
                            .unwrap(),
                        );

                        variant_header.extend(quote! {
                            #field_name: #crate_name::BorshDeserialize::deserialize(buf)?,
                        });
                    }
                }
                variant_header = quote! { { #variant_header }};
            }
            Fields::Unnamed(fields) => {
                for field in fields.unnamed.iter() {
                    if contains_skip(&field.attrs) {
                        variant_header.extend(quote! { Default::default(), });
                    } else {
                        let field_type = &field.ty;
                        where_clause.predicates.push(
                            syn::parse2(quote! {
                                #field_type: #crate_name::BorshDeserialize
                            })
                            .unwrap(),
                        );

                        variant_header
                            .extend(quote! { #crate_name::BorshDeserialize::deserialize(buf)?, });
                    }
                }
                variant_header = quote! { ( #variant_header )};
            }
            Fields::Unit => {}
        }

        variant_arms.extend(quote! {
            #variant_idx => #name::#variant_ident #variant_header ,
        });

        variant_idx += 1;
        is_last_deprecated = is_deprecated;
    }
    let variant_idx = quote! {
        let variant_idx: u8 = #crate_name::BorshDeserialize::deserialize(buf)?;
    };
    if let Some(method_ident) = init_method {
        Ok(quote! {
            impl #impl_generics #crate_name::de::BorshDeserialize for #name #ty_generics #where_clause {
                fn deserialize(buf: &mut &[u8]) -> core::result::Result<Self, #crate_name::maybestd::io::Error> {
                    #variant_idx
                    let mut return_value = match variant_idx {
                        #variant_arms
                        _ => {
                            let msg = #crate_name::maybestd::format!("Unexpected variant index: {:?}", variant_idx);

                            return Err(#crate_name::maybestd::io::Error::new(
                                #crate_name::maybestd::io::ErrorKind::InvalidInput,
                                msg,
                            ));
                        }
                    };
                    return_value.#method_ident();
                    Ok(return_value)
                }
            }
        })
    } else {
        Ok(quote! {
            impl #impl_generics #crate_name::de::BorshDeserialize for #name #ty_generics #where_clause {
                fn deserialize(buf: &mut &[u8]) -> core::result::Result<Self, #crate_name::maybestd::io::Error> {
                    #variant_idx
                    let return_value = match variant_idx {
                        #variant_arms
                        _ => {
                            let msg = #crate_name::maybestd::format!("Unexpected variant index: {:?}", variant_idx);

                            return Err(#crate_name::maybestd::io::Error::new(
                                #crate_name::maybestd::io::ErrorKind::InvalidInput,
                                msg,
                            ));
                        }
                    };
                    Ok(return_value)
                }
            }
        })
    }
}

pub fn contains_skip(attrs: &[Attribute]) -> bool {
    for attr in attrs.iter() {
        if let Ok(Meta::Path(path)) = attr.parse_meta() {
            if path.to_token_stream().to_string().as_str() == "borsh_skip" {
                return true;
            }
        }
    }
    false
}

pub fn contains_initialize_with(attrs: &[Attribute]) -> syn::Result<Option<Path>> {
    for attr in attrs.iter() {
        if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
            if meta_list.path.to_token_stream().to_string().as_str() == "borsh_init" {
                if meta_list.nested.len() != 1 {
                    return Err(Error::new(
                        meta_list.span(),
                        "borsh_init requires exactly one initialization method.",
                    ));
                }
                let nested_meta = meta_list.nested.iter().next().unwrap();
                if let NestedMeta::Meta(Meta::Path(path)) = nested_meta {
                    return Ok(Some(path.clone()));
                }
            }
        }
    }
    Ok(None)
}

pub fn is_deprecated(attrs: &[Attribute]) -> bool {
    for attr in attrs.iter() {
        if let Ok(Meta::Path(path)) = attr.parse_meta() {
            if path.is_ident("deprecated") {
                return true;
            }
        }
    }
    false
}

pub fn get_discriminant(attrs: &[Attribute]) -> Result<Option<u8>> {
    let mut discriminant: Option<u8> = None;

    for attr in attrs.iter() {
        if attr.path.is_ident("discriminant") {
            let attr_span = attr.span();

            if discriminant.is_some() {
                return Err(Error::new(
                    attr_span,
                    "The discriminant attribute can only be used once",
                ));
            }

            let path = &attr.path;
            let tokens = &attr.tokens;
            let tokens = quote! {#path #tokens};

            let args = match parse_macro_input::parse::<MetaList>(tokens.into()) {
                Ok(v) => v,
                Err(_) => {
                    return Err(Error::new(
                        attr_span,
                        "The discriminant attribute expects one integer literal as arguments",
                    ));
                }
            };

            if args.nested.len() != 1 {
                return Err(Error::new(
                    attr_span,
                    "The discriminant attribute expects only one argument",
                ));
            }

            // Check first argument is a literal string.
            let first_argument = args.nested.first().unwrap();
            match first_argument {
                NestedMeta::Lit(v) => match v {
                    Lit::Int(v) => {
                        discriminant = Some(v.base10_parse()?);
                    }
                    v => {
                        return Err(Error::new(v.span(), "This must be a literal string"));
                    }
                },
                NestedMeta::Meta(v) => {
                    return Err(Error::new(v.span(), "This must be a literal string"));
                }
            }
        }
    }

    Ok(discriminant)
}

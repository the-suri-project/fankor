use crate::Result;
use core::convert::TryFrom;
use std::collections::HashSet;
use std::fmt::Display;
use std::str::FromStr;

use crate::utils::unwrap_int_from_literal;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::spanned::Spanned;
use syn::{Attribute, Error, Expr, Fields, Ident, ItemEnum, Meta, Variant, WhereClause};

pub fn enum_ser(input: &ItemEnum, crate_name: Ident) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let mut where_clause = where_clause.map_or_else(
        || WhereClause {
            where_token: Default::default(),
            predicates: Default::default(),
        },
        Clone::clone,
    );

    let mut variant_idx_body = TokenStream2::new();
    let mut fields_body = TokenStream2::new();
    let mut variant_idx = 0u8;
    let mut used_discriminants = HashSet::new();
    let mut is_last_deprecated = false;

    for variant in input.variants.iter() {
        let variant_ident = &variant.ident;
        let mut variant_header = TokenStream2::new();
        let mut variant_body = TokenStream2::new();

        let is_deprecated = is_deprecated(&variant.attrs);
        let discriminant = get_discriminant(variant)?;

        // Calculate the discriminant.
        if let Some(v) = discriminant {
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
            Fields::Named(fields) => {
                for field in &fields.named {
                    let field_name = field.ident.as_ref().unwrap();
                    if contains_skip(&field.attrs) {
                        variant_header.extend(quote! { #field_name, });
                        continue;
                    } else {
                        let field_type = &field.ty;
                        where_clause.predicates.push(
                            syn::parse2(quote! {
                                #field_type: #crate_name::ser::BorshSerialize
                            })
                            .unwrap(),
                        );
                        variant_header.extend(quote! { #field_name, });
                    }
                    variant_body.extend(quote! {
                         #crate_name::BorshSerialize::serialize(#field_name, writer)?;
                    })
                }
                variant_header = quote! { { #variant_header }};
                variant_idx_body.extend(quote!(
                    #name::#variant_ident { .. } => #variant_idx,
                ));
            }
            Fields::Unnamed(fields) => {
                for (field_idx, field) in fields.unnamed.iter().enumerate() {
                    let field_idx =
                        u32::try_from(field_idx).expect("up to 2^32 fields are supported");
                    if contains_skip(&field.attrs) {
                        let field_ident =
                            Ident::new(format!("_id{}", field_idx).as_str(), Span::call_site());
                        variant_header.extend(quote! { #field_ident, });
                        continue;
                    } else {
                        let field_type = &field.ty;
                        where_clause.predicates.push(
                            syn::parse2(quote! {
                                #field_type: #crate_name::ser::BorshSerialize
                            })
                            .unwrap(),
                        );

                        let field_ident =
                            Ident::new(format!("id{}", field_idx).as_str(), Span::call_site());
                        variant_header.extend(quote! { #field_ident, });
                        variant_body.extend(quote! {
                            #crate_name::BorshSerialize::serialize(#field_ident, writer)?;
                        })
                    }
                }
                variant_header = quote! { ( #variant_header )};
                variant_idx_body.extend(quote!(
                    #name::#variant_ident(..) => #variant_idx,
                ));
            }
            Fields::Unit => {
                variant_idx_body.extend(quote!(
                    #name::#variant_ident => #variant_idx,
                ));
            }
        }

        fields_body.extend(quote!(
            #name::#variant_ident #variant_header => {
                #variant_body
            }
        ));

        variant_idx += 1;
        is_last_deprecated = is_deprecated;
    }

    Ok(quote! {
        impl #impl_generics #crate_name::ser::BorshSerialize for #name #ty_generics #where_clause {
            fn serialize<W: #crate_name::maybestd::io::Write>(&self, writer: &mut W) -> core::result::Result<(), #crate_name::maybestd::io::Error> {
                let variant_idx: u8 = match self {
                    #variant_idx_body
                };
                writer.write_all(&variant_idx.to_le_bytes())?;

                match self {
                    #fields_body
                }
                Ok(())
            }
        }
    })
}

pub fn contains_skip(attrs: &[Attribute]) -> bool {
    for attr in attrs.iter() {
        if let Ok(Meta::Path(path)) = attr.parse_meta() {
            if path.is_ident("borsh_skip") {
                return true;
            }
        }
    }
    false
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

pub fn get_discriminant<N>(variant: &Variant) -> Result<Option<N>>
where
    N: FromStr,
    N::Err: Display,
{
    let (_, expr) = match &variant.discriminant {
        Some(v) => v,
        None => return Ok(None),
    };

    match expr {
        Expr::Lit(v) => {
            let literal = unwrap_int_from_literal(v.lit.clone())?;
            Ok(Some(literal.base10_parse()?))
        }
        _ => Err(Error::new(
            variant.span(),
            "Discriminant must be a literal number",
        )),
    }
}

use core::convert::TryFrom;

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{Attribute, Error, Fields, Ident, ItemEnum, Meta, WhereClause};

use crate::fnk_syn::FnkMetaArgumentList;

pub fn enum_ser(input: &ItemEnum, crate_name: Ident) -> syn::Result<TokenStream2> {
    let name = &input.ident;

    // Check for fankor attribute.
    let mut account_discriminants = None;
    let mut is_accounts = false;

    for attr in &input.attrs {
        if attr.path().is_ident("fankor") {
            if let Ok(mut args) = attr.parse_args::<FnkMetaArgumentList>() {
                args.error_on_duplicated()?;

                if let Some(v) = args.pop_ident("account", true)? {
                    if is_accounts {
                        return Err(Error::new(
                            attr.span(),
                            "Cannot define both fankor::accounts and fankor::account attributes",
                        ));
                    }

                    account_discriminants = Some(v);
                }

                if args.pop_plain("accounts", true)? {
                    if account_discriminants.is_some() {
                        return Err(Error::new(
                            attr.span(),
                            "Cannot define both fankor::accounts and fankor::account attributes",
                        ));
                    }

                    is_accounts = true;
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
        quote! {
            #crate_name::BorshSerialize::serialize(&#account_discriminants::#name.code(), writer)?;
        }
    } else {
        quote! {}
    };

    let discriminant_name = format_ident!("{}Discriminant", name);
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

    for variant in input.variants.iter() {
        let variant_ident = &variant.ident;
        let mut variant_header = TokenStream2::new();
        let mut variant_body = TokenStream2::new();

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
                    #name::#variant_ident { .. } => #discriminant_name::#variant_ident.code(),
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
                    #name::#variant_ident(..) => #discriminant_name::#variant_ident.code(),
                ));
            }
            Fields::Unit => {
                variant_idx_body.extend(quote!(
                    #name::#variant_ident => #discriminant_name::#variant_ident.code(),
                ));
            }
        }

        fields_body.extend(quote!(
            #name::#variant_ident #variant_header => {
                #variant_body
            }
        ));
    }

    let variant_writer = if is_accounts {
        quote! {}
    } else {
        quote! {
            let variant_idx: u8 = match self {
                #variant_idx_body
            };
            writer.write_all(&variant_idx.to_le_bytes())?;
        }
    };

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #crate_name::ser::BorshSerialize for #name #ty_generics #where_clause {
            fn serialize<W: #crate_name::maybestd::io::Write>(&self, writer: &mut W) -> core::result::Result<(), #crate_name::maybestd::io::Error> {
                #account_discriminants
                #variant_writer

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
        if let Meta::Path(path) = &attr.meta {
            if path.is_ident("borsh_skip") {
                return true;
            }
        }
    }
    false
}

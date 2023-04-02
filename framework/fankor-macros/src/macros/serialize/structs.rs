use crate::fnk_syn::FnkMetaArgumentList;
use crate::macros::serialize::enums::contains_skip;
use core::convert::TryFrom;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::spanned::Spanned;
use syn::{Error, Fields, Ident, Index, ItemStruct, WhereClause};

pub fn struct_ser(input: &ItemStruct, crate_name: Ident) -> syn::Result<TokenStream2> {
    let name = &input.ident;

    // Check for fankor attribute.
    let mut account_discriminants = None;

    for attr in &input.attrs {
        if attr.path.is_ident("fankor") {
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
        quote! {
            #crate_name::BorshSerialize::serialize(&#account_discriminants::#name.code(), writer)?;
        }
    } else {
        quote! {}
    };

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let mut where_clause = where_clause.map_or_else(
        || WhereClause {
            where_token: Default::default(),
            predicates: Default::default(),
        },
        Clone::clone,
    );
    let mut body = TokenStream2::new();
    match &input.fields {
        Fields::Named(fields) => {
            for field in &fields.named {
                if contains_skip(&field.attrs) {
                    continue;
                }
                let field_name = field.ident.as_ref().unwrap();
                let delta = quote! {
                    #crate_name::BorshSerialize::serialize(&self.#field_name, writer)?;
                };
                body.extend(delta);

                let field_type = &field.ty;
                where_clause.predicates.push(
                    syn::parse2(quote! {
                        #field_type: #crate_name::ser::BorshSerialize
                    })
                    .unwrap(),
                );
            }
        }
        Fields::Unnamed(fields) => {
            for field_idx in 0..fields.unnamed.len() {
                let field_idx = Index {
                    index: u32::try_from(field_idx).expect("up to 2^32 fields are supported"),
                    span: Span::call_site(),
                };
                let delta = quote! {
                    #crate_name::BorshSerialize::serialize(&self.#field_idx, writer)?;
                };
                body.extend(delta);
            }
        }
        Fields::Unit => {}
    }
    Ok(quote! {
        impl #impl_generics #crate_name::ser::BorshSerialize for #name #ty_generics #where_clause {
            fn serialize<W: #crate_name::maybestd::io::Write>(&self, writer: &mut W) -> ::core::result::Result<(), #crate_name::maybestd::io::Error> {
                #account_discriminants
                #body
                Ok(())
            }
        }
    })
}

use quote::{format_ident, quote};
use std::collections::HashSet;
use syn::spanned::Spanned;
use syn::{Error, Fields, Item};

use crate::macros::serialize::{get_discriminant, is_deprecated};
use crate::Result;

pub fn processor(input: Item) -> Result<proc_macro::TokenStream> {
    // Process input.
    let result = match &input {
        Item::Enum(item) => {
            let visibility = &item.vis;
            let name = &item.ident;
            let discriminant_name = format_ident!("{}Discriminants", item.ident);

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

            quote! {
                #[allow(dead_code)]
                #[automatically_derived]
                #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
                #visibility enum #discriminant_name {
                    #(#fields,)*
                }

                #[automatically_derived]
                impl #discriminant_name {
                    pub fn code(&self) -> u8 {
                        match self {
                            #(#codes,)*
                        }
                    }
                }

                #[automatically_derived]
                impl #impl_generics #name #ty_generics #where_clause {
                    pub fn discriminant(&self) -> #discriminant_name {
                        match self {
                            #(#discriminants,)*
                        }
                    }
                }
            }
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

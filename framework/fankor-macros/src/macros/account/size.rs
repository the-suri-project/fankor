use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{Error, Fields, Item, Type};

use fankor_syn::Result;

pub fn processor(input: Item) -> Result<proc_macro::TokenStream> {
    // Process input.
    let result = match &input {
        Item::Struct(item) => {
            let name = &item.ident;
            let generic_where_clause = &item.generics.where_clause;
            let generic_params = &item.generics.params;
            let generic_params = if generic_params.is_empty() {
                quote! {}
            } else {
                quote! { < #generic_params > }
            };

            let min_size_fields = item
                .fields
                .iter()
                .map(|v| get_min_size_of(&v.ty))
                .collect::<Vec<_>>();

            let actual_size_fields = item
                .fields
                .iter()
                .map(|v| {
                    let name = v.ident.as_ref().unwrap();

                    quote! {
                        ::fankor::traits::AccountSize::actual_account_size(&self.#name)
                    }
                })
                .collect::<Vec<_>>();

            quote! {
                #[automatically_derived]
                 impl #generic_params ::fankor::traits::AccountSize for #name #generic_params #generic_where_clause {
                    fn min_account_size() -> usize {
                        let mut min_size = 0;

                        #(min_size += #min_size_fields;)*

                        min_size
                    }

                    fn actual_account_size(&self) -> usize {
                        let mut actual_size = 0;

                        #(actual_size += #actual_size_fields;)*

                        actual_size
                    }
                }
            }
        }
        Item::Enum(item) => {
            let name = &item.ident;
            let generic_where_clause = &item.generics.where_clause;
            let generic_params = &item.generics.params;
            let generic_params = if generic_params.is_empty() {
                quote! {}
            } else {
                quote! { < #generic_params > }
            };

            let min_size_variants = item
                .variants
                .iter()
                .filter_map(|v| {
                    let fields = match &v.fields {
                        Fields::Named(v) => &v.named,
                        Fields::Unnamed(v) => &v.unnamed,
                        Fields::Unit => return None,
                    };

                    let min_size_variants = fields
                        .iter()
                        .map(|v| get_min_size_of(&v.ty))
                        .collect::<Vec<_>>();

                    Some(quote! {{
                        let mut min_size = 1;
                        #(min_size += #min_size_variants;)*
                        min_size
                    }})
                })
                .collect::<Vec<_>>();

            let actual_size_variants = item
                .variants
                .iter()
                .filter_map(|variant| match &variant.fields {
                    Fields::Named(v) => {
                        let variant_names = v
                            .named
                            .iter()
                            .map(|v| v.ident.as_ref().unwrap())
                            .collect::<Vec<_>>();

                        let actual_size_variants = variant_names.iter().map(|name| {
                            quote! {
                                ::fankor::traits::AccountSize::actual_account_size(#name)
                            }
                        });

                        let variant_name = &variant.ident;
                        Some(quote! {#name::#variant_name {#(#variant_names),*} => {
                            let mut actual_size = 1;

                            #(actual_size += #actual_size_variants;)*

                            actual_size
                        }})
                    }
                    Fields::Unnamed(v) => {
                        let variant_names = v
                            .unnamed
                            .iter()
                            .enumerate()
                            .map(|(i, _)| format_ident!("v{}", i))
                            .collect::<Vec<_>>();

                        let actual_size_variants = variant_names.iter().map(|name| {
                            quote! {
                                ::fankor::traits::AccountSize::actual_account_size(#name)
                            }
                        });

                        let variant_name = &variant.ident;
                        Some(quote! {#name::#variant_name(#(#variant_names),*) => {
                            let mut actual_size = 1;

                            #(actual_size += #actual_size_variants;)*

                            actual_size
                        }})
                    }
                    Fields::Unit => None,
                })
                .collect::<Vec<_>>();

            quote! {
                #[automatically_derived]
                impl #generic_params ::fankor::traits::AccountSize for #name #generic_params #generic_where_clause {
                    fn min_account_size() -> usize {
                        let mut min_size = 1;

                        #(min_size = min_size.max(#min_size_variants);)*

                        min_size
                    }

                    fn actual_account_size(&self) -> usize {
                        let mut actual_size = match self {
                            #(#actual_size_variants,)*
                            _ => 1
                        };

                        actual_size
                    }
                }
            }
        }
        _ => {
            return Err(Error::new(
                input.span(),
                "AccountSize macro can only be applied to struct or enum declarations",
            ));
        }
    };

    Ok(result.into())
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub fn get_min_size_of(ty: &Type) -> TokenStream {
    quote! {
        <#ty as ::fankor::traits::AccountSize>::min_account_size()
    }
}

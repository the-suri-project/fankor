use convert_case::{Case, Converter};
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{Error, Fields, Item};

use crate::macros::account::size::get_min_size_of;
use fankor_syn::Result;

pub fn processor(input: Item) -> Result<proc_macro::TokenStream> {
    let case_converter = Converter::new()
        .from_case(Case::Snake)
        .to_case(Case::Pascal);

    // Process input.
    let result = match &input {
        Item::Struct(item) => {
            let visibility = &item.vis;
            let name = format_ident!("{}Fields", item.ident);

            let fields = item
                .fields
                .iter()
                .map(|v| {
                    let name = v.ident.as_ref().unwrap();
                    format_ident!(
                        "{}",
                        case_converter.convert(name.to_string()),
                        span = name.span()
                    )
                })
                .collect::<Vec<_>>();

            let mut offset_variant_acc = quote!(0);
            let offset_variants = item.fields.iter().zip(&fields).map(|(v, field)| {
                let min_size = get_min_size_of(&v.ty);
                let result = quote!(
                    #name::#field => #offset_variant_acc
                );

                offset_variant_acc = quote!(
                    #offset_variant_acc + #min_size
                );

                result
            });

            quote!(
                #[automatically_derived]
                #visibility enum #name {
                    #(#fields,)*
                }

                #[automatically_derived]
                impl #name {
                    pub fn offset(&self) -> usize {
                        match self {
                            #(#offset_variants,)*
                        }
                    }
                }
            )
        }
        Item::Enum(item) => {
            let visibility = &item.vis;
            let fields_ident = format_ident!("{}Fields", item.ident);

            let fields = item
                .variants
                .iter()
                .filter_map(|variant| match &variant.fields {
                    Fields::Named(v) => {
                        let variant_name = &variant.ident;

                        let variants = v
                            .named
                            .iter()
                            .map(|v| {
                                let name = v.ident.as_ref().unwrap();
                                format_ident!(
                                    "{}{}",
                                    variant_name,
                                    case_converter.convert(name.to_string()),
                                    span = name.span()
                                )
                            })
                            .collect::<Vec<_>>();

                        Some(quote!(
                            #(#variants,)*
                        ))
                    }
                    Fields::Unnamed(v) => {
                        let variant_name = &variant.ident;

                        let variants = v
                            .unnamed
                            .iter()
                            .enumerate()
                            .map(|(i, _)| format_ident!("{}{}", variant_name, i))
                            .collect::<Vec<_>>();

                        Some(quote!(
                            #(#variants,)*
                        ))
                    }
                    Fields::Unit => None,
                });

            let offset_variants =
                item.variants
                    .iter()
                    .filter_map(|variant| match &variant.fields {
                        Fields::Named(v) => {
                            let variant_name = &variant.ident;

                            let mut offset_variant_acc = quote!(1);
                            let variants = v
                                .named
                                .iter()
                                .map(|v| {
                                    let name = v.ident.as_ref().unwrap();
                                    let name = format_ident!(
                                        "{}{}",
                                        variant_name,
                                        case_converter.convert(name.to_string()),
                                        span = name.span()
                                    );
                                    let min_size = get_min_size_of(&v.ty);
                                    let result = quote!(
                                        #fields_ident::#name => #offset_variant_acc
                                    );

                                    offset_variant_acc = quote!(
                                        #offset_variant_acc + #min_size
                                    );

                                    result
                                })
                                .collect::<Vec<_>>();

                            Some(quote!(
                                #(#variants,)*
                            ))
                        }
                        Fields::Unnamed(v) => {
                            let variant_name = &variant.ident;

                            let mut offset_variant_acc = quote!(1);
                            let variants = v
                                .unnamed
                                .iter()
                                .enumerate()
                                .map(|(i, v)| {
                                    let name = format_ident!("{}{}", variant_name, i);
                                    let min_size = get_min_size_of(&v.ty);
                                    let result = quote!(
                                        #fields_ident::#name => #offset_variant_acc
                                    );

                                    offset_variant_acc = quote!(
                                        #offset_variant_acc + #min_size
                                    );

                                    result
                                })
                                .collect::<Vec<_>>();

                            Some(quote!(
                                #(#variants,)*
                            ))
                        }
                        Fields::Unit => None,
                    });

            quote!(
                #[automatically_derived]
                #visibility enum #fields_ident {
                    #(#fields)*
                }

                #[automatically_derived]
                impl #fields_ident {
                    pub fn offset(&self) -> usize {
                        match self {
                            #(#offset_variants)*
                        }
                    }
                }
            )
        }
        _ => {
            return Err(Error::new(
                input.span(),
                "AccountOffsets macro can only be applied to struct or enum declarations",
            ));
        }
    };

    Ok(result.into())
}

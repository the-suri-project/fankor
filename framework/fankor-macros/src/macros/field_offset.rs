use convert_case::{Case, Converter};
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{Error, Fields, Item};

use crate::Result;

pub fn processor(input: Item) -> Result<proc_macro::TokenStream> {
    let case_converter = Converter::new()
        .from_case(Case::Snake)
        .to_case(Case::Pascal);

    // Process input.
    let result = match &input {
        Item::Struct(item) => {
            let name = &item.ident;
            let fields_name = format_ident!("{}Fields", name);

            let (_, ty_generics, _) = item.generics.split_for_impl();

            let mut offsets_acc = quote! {  0};
            let mut offsets = Vec::new();
            let mut actual_offsets = Vec::new();

            for field in &item.fields {
                let ty = &field.ty;
                let original_field_name = field.ident.as_ref().unwrap();
                let field_name = format_ident!(
                    "{}",
                    case_converter.convert(original_field_name.to_string()),
                    span = original_field_name.span()
                );

                let min_size = quote! {
                    <#ty as ::fankor::traits::CopyType>::min_byte_size()
                };

                offsets.push(quote! {
                    #fields_name::#field_name => #offsets_acc
                });
                actual_offsets.push(quote! {
                    if *self == #fields_name::#field_name {
                        return size;
                    }

                    size += ::fankor::traits::CopyType::byte_size(&obj.#original_field_name);
                });

                offsets_acc = quote! {
                    #offsets_acc + #min_size
                };
            }

            let result = quote! {
                #[automatically_derived]
                impl #fields_name {
                    pub fn offset(&self) -> usize {
                        match self {
                            #(#offsets,)*
                        }
                    }

                    pub fn actual_offset(&self, obj: &#name #ty_generics) -> usize {
                        let mut size = 0;
                        #(#actual_offsets)*
                        size
                    }
                }
            };

            // TypeScript generation.
            let mut ts_enum_replacements = Vec::new();
            let ts_offsets = item
                .fields
                .iter()
                .map(|field| {
                    let original_field_name = field.ident.as_ref().unwrap();
                    let field_name = format_ident!(
                        "{}",
                        case_converter.convert(original_field_name.to_string()),
                        span = original_field_name.span()
                    );
                    let replacement_str = format!("_r_{}_r_", field_name);

                    ts_enum_replacements.push(quote! {
                        .replace(#replacement_str, &#fields_name::#field_name.offset().to_string())
                    });

                    format!("{} = {},", field_name, replacement_str)
                })
                .collect::<Vec<_>>();

            let ts_enum_name_str = format!("{}FieldOffset", name);
            let ts_enum = format!(
                "export enum {} {{
                    {}
                }}",
                ts_enum_name_str,
                ts_offsets.join("\n"),
            );

            let test_name = format_ident!("__ts_gen_test__account_offset_{}", ts_enum_name_str);
            let test_name_str = test_name.to_string();
            let result = quote! {
                #result

                #[cfg(feature = "ts-gen")]
                #[automatically_derived]
                #[allow(non_snake_case)]
                mod #test_name {
                    use super::*;

                    #[test]
                    fn build() {
                         // Register action.
                        crate::__ts_gen_test__setup::BUILD_CONTEXT.register_action(#test_name_str, file!(), move |action_context| {
                            let ts_enum = #ts_enum .to_string() #(#ts_enum_replacements)*;
                            action_context.add_created_type(#ts_enum_name_str, std::borrow::Cow::Owned(ts_enum)).unwrap();
                        })
                    }
                }
            };

            result
        }
        Item::Enum(item) => {
            let visibility = &item.vis;
            let name = &item.ident;
            let fields_name = format_ident!("{}Fields", item.ident);

            let (_, ty_generics, _) = item.generics.split_for_impl();

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

                        Some(quote! {
                            #(#variants,)*
                        })
                    }
                    Fields::Unnamed(v) => {
                        let variant_name = &variant.ident;

                        let variants = v
                            .unnamed
                            .iter()
                            .enumerate()
                            .map(|(i, _)| format_ident!("{}{}", variant_name, i))
                            .collect::<Vec<_>>();

                        Some(quote! {
                            #(#variants,)*
                        })
                    }
                    Fields::Unit => None,
                });

            let offsets = item
                .variants
                .iter()
                .filter_map(|variant| match &variant.fields {
                    Fields::Named(v) => {
                        let variant_name = &variant.ident;

                        let mut offset_variant_acc = quote! {1};
                        let variants = v
                            .named
                            .iter()
                            .map(|v| {
                                let name = v.ident.as_ref().unwrap();
                                let ty = &v.ty;
                                let name = format_ident!(
                                    "{}{}",
                                    variant_name,
                                    case_converter.convert(name.to_string()),
                                    span = name.span()
                                );
                                let min_size = quote! {
                                    <#ty as ::fankor::traits::CopyType>::min_byte_size()
                                };
                                let result = quote! {
                                    #fields_name::#name => #offset_variant_acc
                                };

                                offset_variant_acc = quote! {
                                    #offset_variant_acc + #min_size
                                };

                                result
                            })
                            .collect::<Vec<_>>();

                        Some(quote! {
                                #(#variants,)*
                        })
                    }
                    Fields::Unnamed(v) => {
                        let variant_name = &variant.ident;

                        let mut offset_variant_acc = quote! { 1};
                        let variants = v
                            .unnamed
                            .iter()
                            .enumerate()
                            .map(|(i, v)| {
                                let name = format_ident!("{}{}", variant_name, i);
                                let ty = &v.ty;
                                let min_size = quote! {
                                    <#ty as ::fankor::traits::CopyType>::min_byte_size()
                                };
                                let result = quote! {
                                    #fields_name::#name => #offset_variant_acc
                                };

                                offset_variant_acc = quote! {
                                    #offset_variant_acc + #min_size
                                };

                                result
                            })
                            .collect::<Vec<_>>();

                        Some(quote! {
                            #(#variants,)*
                        })
                    }
                    Fields::Unit => None,
                });

            let actual_offsets = item.variants.iter().map(|variant| match &variant.fields {
                Fields::Named(v) => {
                    let variant_name = &variant.ident;

                    let args = v.named.iter().map(|v| v.ident.as_ref().unwrap());

                    let actual_offsets = v.named.iter().map(|v| {
                        let arg_name = v.ident.as_ref().unwrap();
                        let field = format_ident!(
                            "{}{}",
                            variant_name,
                            case_converter.convert(arg_name.to_string()),
                            span = arg_name.span()
                        );

                        quote! {
                            if *self == #fields_name::#field {
                                return Some(size);
                            }

                            size += ::fankor::traits::CopyType::byte_size(#arg_name);
                        }
                    });

                    Some(quote! {
                        #name::#variant_name {#(#args),*} => {
                            let mut size = 1;
                            #(#actual_offsets)*
                            None
                        }
                    })
                }
                Fields::Unnamed(v) => {
                    let variant_name = &variant.ident;

                    let args = v
                        .unnamed
                        .iter()
                        .enumerate()
                        .map(|(i, _)| format_ident!("v{}", i));

                    let actual_offsets = v.unnamed.iter().enumerate().map(|(i, _)| {
                        let arg_name = format_ident!("v{}", i);
                        let field = format_ident!("{}{}", variant_name, i);

                        quote! {
                            if *self == #fields_name::#field {
                                return Some(size);
                            }

                            size += ::fankor::traits::CopyType::byte_size(#arg_name);
                        }
                    });

                    Some(quote! {
                        #name::#variant_name (#(#args),*) => {
                            let mut size = 1;
                            #(#actual_offsets)*
                            None
                        }
                    })
                }
                Fields::Unit => {
                    let variant_name = &variant.ident;

                    Some(quote! {
                        #name::#variant_name => None,
                    })
                }
            });

            let result = quote! {
                #[allow(dead_code)]
                #[automatically_derived]
                #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
                #visibility enum #fields_name {
                    #(#fields)*
                }

                #[automatically_derived]
                impl #fields_name {
                    pub fn offset(&self) -> usize {
                        match self {
                            #(#offsets)*
                        }
                    }

                    pub fn actual_offset(&self, obj: &#name #ty_generics) -> Option<usize> {
                        match obj {
                            #(#actual_offsets)*
                        }
                    }
                }
            };

            // TypeScript generation.
            let mut ts_enum_replacements = Vec::new();
            let ts_offsets = item
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
                                let name = format_ident!(
                                    "{}{}",
                                    variant_name,
                                    case_converter.convert(name.to_string()),
                                    span = name.span()
                                );

                                let replacement_str = format!("_r_{}_r_", name);

                                ts_enum_replacements.push(quote! {
                                    .replace(#replacement_str, &#fields_name::#name.offset().to_string())
                                });

                                format!("{} = {},", name, replacement_str)
                            })
                            .collect::<Vec<_>>();

                        Some(variants.join("\n"))
                    }
                    Fields::Unnamed(v) => {
                        let variant_name = &variant.ident;

                        let variants = v
                            .unnamed
                            .iter()
                            .enumerate()
                            .map(|(i, _)| {
                                let name = format_ident!("{}{}", variant_name, i);
                                let replacement_str = format!("_r_{}_r_", name);

                                ts_enum_replacements.push(quote! {
                                    .replace(#replacement_str, &#fields_name::#name.offset().to_string())
                                });

                                format!("{} = {},", name, replacement_str)
                            })
                            .collect::<Vec<_>>();

                        Some(variants.join("\n"))
                    }
                    Fields::Unit => None,
                }).collect::<Vec<_>>();

            let ts_enum_name_str = format!("{}FieldOffset", name);
            let ts_enum = format!(
                "export enum {} {{
                    {}
                }}",
                ts_enum_name_str,
                ts_offsets.join("\n"),
            );

            let test_name = format_ident!("__ts_gen_test__account_offset_{}", ts_enum_name_str);
            let test_name_str = test_name.to_string();
            let result = quote! {
                #result

                #[cfg(feature = "ts-gen")]
                #[automatically_derived]
                #[allow(non_snake_case)]
                mod #test_name {
                    use super::*;

                    #[test]
                    fn build() {
                         // Register action.
                        crate::__ts_gen_test__setup::BUILD_CONTEXT.register_action(#test_name_str, file!(), move |action_context| {
                            let ts_enum = #ts_enum .to_string() #(#ts_enum_replacements)*;
                            action_context.add_created_type(#ts_enum_name_str, std::borrow::Cow::Owned(ts_enum)).unwrap();
                        })
                    }
                }
            };

            result
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

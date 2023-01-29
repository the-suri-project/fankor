use convert_case::{Case, Converter};
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{Error, Fields, Item};

use crate::Result;

pub fn processor(input: Item) -> Result<proc_macro::TokenStream> {
    let result = match &input {
        Item::Struct(item) => {
            let name = &item.ident;

            let (_, ty_generics, where_clause) = item.generics.split_for_impl();

            let byte_size_from_instance_method = item.fields.iter().map(|field| {
                let field_name = &field.ident;

                quote! {
                    size += self.#field_name.byte_size_from_instance();
                }
            });

            let read_byte_size_from_bytes_method = item.fields.iter().map(|field| {
                let field_ty = &field.ty;

                quote! {
                    size += <#field_ty as CopyType>::ZeroCopyType::read_byte_size_from_bytes(&bytes[size..])?;
                }
            });

            let zc_name = format_ident!("Zc{}", name);
            let name_fields = format_ident!("{}ZcFields", name);
            let mut aux_zc_generics = item.generics.clone();
            aux_zc_generics
                .params
                .insert(0, syn::parse_quote! { 'info });

            let (zc_impl_generics, zc_ty_generics, zc_where_clause) =
                aux_zc_generics.split_for_impl();

            let case_converter = Converter::new()
                .from_case(Case::Snake)
                .to_case(Case::Pascal);

            let mut zc_field_names = Vec::with_capacity(item.fields.len());
            let mut zc_field_methods_aux = Vec::with_capacity(item.fields.len());
            let mut zc_from_previous_methods = Vec::with_capacity(item.fields.len());
            let mut zc_from_previous_methods_lasts = Vec::with_capacity(item.fields.len());
            let zc_field_methods = item.fields.iter().map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                let from_previous_method_name = format_ident!("{}_from_previous_unchecked", field_name);
                let field_ty = &field.ty;

                let pascal_field_name = format_ident!("{}", case_converter.convert(field_name.to_string()), span = field_name.span());
                zc_field_names.push(pascal_field_name.clone());

                let res = quote! {
                    pub fn #field_name(&self) -> FankorResult<Zc<'info, #field_ty>> {
                        let bytes = self.info.try_borrow_data()
                            .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock {
                                type_name: std::any::type_name::<Self>(),
                            })?;
                        let bytes = &bytes[self.offset..];
                        let mut size = 0;

                        #(#zc_field_methods_aux)*

                        Ok(Zc::new_unchecked(self.info, self.offset + size))
                    }
                };

                if !zc_from_previous_methods_lasts.is_empty() {
                    zc_from_previous_methods.push(quote! {
                        pub fn #from_previous_method_name(&self, previous: #name_fields, mut offset: usize) -> FankorResult<Zc<'info, #field_ty>> {
                            let bytes = self.info.try_borrow_data().map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock { type_name: std::any::type_name::<Self>() })?;
                            let mut processed = false;

                            #(#zc_from_previous_methods_lasts)*

                            if !processed {
                                return Err(FankorErrorCode::ZeroCopyIncorrectPrecedingField.into());
                            }

                            Ok(Zc::new_unchecked(self.info, offset))
                        }
                    });
                }

                zc_field_methods_aux.push(quote! {
                    size += <#field_ty as CopyType>::ZeroCopyType::read_byte_size_from_bytes(&bytes[size..])?;
                });

                zc_from_previous_methods_lasts.push(quote! {
                    if processed || previous == #name_fields::#pascal_field_name {
                        offset += <#field_ty as CopyType>::ZeroCopyType::read_byte_size_from_bytes(&bytes[offset..])?;
                        processed = true;
                    }
                });

                res
            }).collect::<Vec<_>>();

            quote! {
                #[automatically_derived]
                impl #zc_impl_generics CopyType<'info> for #name #ty_generics #where_clause {
                    type ZeroCopyType = #zc_name #zc_ty_generics;

                    fn byte_size_from_instance(&self) -> usize {
                        let mut size = 0;
                        #(#byte_size_from_instance_method)*
                        size
                    }
                }

                #[allow(dead_code)]
                #[automatically_derived]
                #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
                pub enum #name_fields { #(#zc_field_names),* }

                // TODO process generics to add phantom data if needed
                pub struct #zc_name #zc_ty_generics #zc_where_clause {
                    info: &'info AccountInfo<'info>,
                    offset: usize,
                }

                #[automatically_derived]
                impl #zc_impl_generics ZeroCopyType<'info> for #zc_name #zc_ty_generics #zc_where_clause {
                    fn new(info: &'info AccountInfo<'info>, offset: usize) -> FankorResult<(Self, Option<usize>)> {
                        Ok((
                            #zc_name {
                                info,
                                offset,
                            },
                            None,
                        ))
                    }

                    fn read_byte_size_from_bytes(bytes: &[u8]) -> FankorResult<usize> {
                        let mut size = 0;
                        #(#read_byte_size_from_bytes_method)*
                        Ok(size)
                    }
                }

                #[automatically_derived]
                impl #zc_impl_generics #zc_name #zc_ty_generics #zc_where_clause {
                    #(#zc_field_methods)*
                    #(#zc_from_previous_methods)*
                }
            }
        }
        Item::Enum(item) => {
            let name = &item.ident;

            let (_, ty_generics, where_clause) = item.generics.split_for_impl();

            let mut aux_zc_generics = item.generics.clone();
            aux_zc_generics
                .params
                .insert(0, syn::parse_quote! { 'info });

            let (zc_impl_generics, zc_ty_generics, zc_where_clause) =
                aux_zc_generics.split_for_impl();

            let byte_size_from_instance_method = item.variants.iter().map(|variant| {
                let variant_name = &variant.ident;

                match &variant.fields {
                    Fields::Named(fields) => {
                        let mut field_names = Vec::with_capacity(fields.named.len());
                        let fields = fields
                            .named
                            .iter()
                            .map(|field| {
                                let field_name = field.ident.as_ref().unwrap();

                                field_names.push(quote! {
                                    #field_name
                                });

                                quote! {
                                    size += #field_name.byte_size_from_instance();
                                }
                            })
                            .collect::<Vec<_>>();

                        quote! {
                            #name::#variant_name { #(#field_names),* } => {
                                #(#fields)*
                            }
                        }
                    }
                    Fields::Unnamed(fields) => {
                        let mut field_names = Vec::with_capacity(fields.unnamed.len());
                        let fields = fields
                            .unnamed
                            .iter()
                            .enumerate()
                            .map(|(i, _)| {
                                let field_name = format_ident!("v{}", i);

                                field_names.push(quote! {
                                    #field_name
                                });

                                quote! {
                                    size += #field_name.byte_size_from_instance();
                                }
                            })
                            .collect::<Vec<_>>();

                        quote! {
                            #name::#variant_name(#(#field_names),*) => {
                                #(#fields)*
                            }
                        }
                    }
                    Fields::Unit => {
                        quote! {
                            #name::#variant_name => {}
                        }
                    }
                }
            });

            let zc_name = format_ident!("Zc{}", name);
            let zc_name_variants = item.variants.iter().map(|variant| {
                let variant_name = &variant.ident;

                match &variant.fields {
                    Fields::Named(fields) => {
                        let fields = fields.named.iter().map(|field| {
                            let field_name = field.ident.as_ref().unwrap();
                            let field_ty = &field.ty;

                            quote! {
                                #field_name: Zc<'info, #field_ty>
                            }
                        });

                        quote! {
                            #variant_name { #(#fields),* }
                        }
                    }
                    Fields::Unnamed(fields) => {
                        let fields = fields.unnamed.iter().map(|field| {
                            let field_ty = &field.ty;

                            quote! {
                                Zc<'info, #field_ty>
                            }
                        });

                        quote! {
                            #variant_name(#(#fields),*)
                        }
                    }
                    Fields::Unit => {
                        quote! {
                            #variant_name
                        }
                    }
                }
            });

            let new_method =
                item.variants
                    .iter()
                    .enumerate()
                    .map(|(i_variant, variant)| {
                        let variant_name = &variant.ident;
                        let i_variant = i_variant as u8;

                        match &variant.fields {
                            Fields::Named(fields) => {
                                let mut field_names = Vec::with_capacity(fields.named.len());
                                let fields = fields.named.iter().map(|field| {
                                    let field_name = field.ident.as_ref().unwrap();
                                    let field_ty = &field.ty;

                                    field_names.push(quote! {
                                        #field_name
                                    });

                                    quote! {
                                        let #field_name = Zc::new_unchecked(info, __offset + size);
                                        size += <#field_ty as CopyType>::ZeroCopyType::read_byte_size_from_bytes(&bytes[size..])?;
                                    }
                                });

                                quote! {
                                    #i_variant => {
                                        #(#fields)*

                                        #zc_name::#variant_name{#(#field_names),*}
                                    }
                                }
                            }
                            Fields::Unnamed(fields) => {
                                let mut field_names = Vec::with_capacity(fields.unnamed.len());
                                let fields = fields.unnamed.iter().enumerate().map(|(i, field)| {
                                    let field_name = format_ident!("v{}", i);
                                    let field_ty = &field.ty;

                                    field_names.push(quote! {
                                        #field_name
                                    });

                                    quote! {
                                        let #field_name = Zc::new_unchecked(info, __offset + size);
                                        size += <#field_ty as CopyType>::ZeroCopyType::read_byte_size_from_bytes(&bytes[size..])?;
                                    }
                                });

                                quote! {
                                    #i_variant => {
                                        #(#fields)*

                                        #zc_name::#variant_name(#(#field_names),*)
                                    }
                                }
                            }
                            Fields::Unit => {
                                quote! {
                                    #i_variant => #zc_name::#variant_name
                                }
                            }
                        }
                    });

            let read_byte_size_from_bytes_method =
                item.variants
                    .iter()
                    .enumerate()
                    .map(|(i_variant, variant)| {
                        let i_variant = i_variant as u8;

                        match &variant.fields {
                            Fields::Named(fields) => {
                                let fields = fields.named.iter().map(|field| {
                                    let field_ty = &field.ty;

                                    quote! {
                                        size += <#field_ty as CopyType>::ZeroCopyType::read_byte_size_from_bytes(&bytes[size..])?;
                                    }
                                });

                                quote! {
                                    #i_variant => {
                                        #(#fields)*
                                    }
                                }
                            }
                            Fields::Unnamed(fields) => {
                                let fields = fields.unnamed.iter().map(|field| {
                                    let field_ty = &field.ty;

                                    quote! {
                                        size += <#field_ty as CopyType>::ZeroCopyType::read_byte_size_from_bytes(&bytes[size..])?;
                                    }
                                });

                                quote! {
                                    #i_variant => {
                                        #(#fields)*
                                    }
                                }
                            }
                            Fields::Unit => {
                                quote! {
                                    #i_variant => {}
                                }
                            }
                        }
                    });

            let is_all_empty = item
                .variants
                .iter()
                .map(|variant| matches!(&variant.fields, Fields::Unit))
                .reduce(|a, b| a && b)
                .unwrap_or(true);

            if is_all_empty {
                let new_method = item
                    .variants
                    .iter()
                    .enumerate()
                    .map(|(i_variant, variant)| {
                        let variant_name = &variant.ident;
                        let i_variant = i_variant as u8;

                        quote! {
                            #i_variant => #name::#variant_name
                        }
                    });

                quote! {
                    #[automatically_derived]
                    impl #zc_impl_generics CopyType<'info> for #name #ty_generics #where_clause {
                        type ZeroCopyType = #name #ty_generics;

                        fn byte_size_from_instance(&self) -> usize {
                            1
                        }
                    }

                    #[automatically_derived]
                    impl #zc_impl_generics ZeroCopyType<'info> for #name #ty_generics #where_clause {
                        fn new(info: &'info AccountInfo<'info>, offset: usize) -> FankorResult<(Self, Option<usize>)> {
                            let bytes = info
                                .try_borrow_data()
                                .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock { type_name: std::any::type_name::<Self>() })?;
                            let bytes = &bytes[offset..];

                            if bytes.is_empty() {
                                return Err(FankorErrorCode::ZeroCopyNotEnoughLength { type_name: std::any::type_name::<Self>() }.into());
                            }

                            let mut size = 1;
                            let flag = bytes[0];

                            let result = match flag {
                                #(#new_method,)*
                                _ => return Err(FankorErrorCode::ZeroCopyInvalidEnumDiscriminant { type_name: std::any::type_name::<Self>() }.into()),
                            };

                            Ok((result, Some(size)))
                        }

                        fn read_byte_size_from_bytes(bytes: &[u8]) -> FankorResult<usize> {
                            if bytes.is_empty() {
                                return Err(FankorErrorCode::ZeroCopyNotEnoughLength { type_name: std::any::type_name::<Self>() }.into());
                            }

                            let mut size = 1;
                            let flag = bytes[0];

                            match flag {
                                #(#read_byte_size_from_bytes_method,)*
                                _ => return Err(FankorErrorCode::ZeroCopyInvalidEnumDiscriminant { type_name: std::any::type_name::<Self>() }.into()),
                            }

                            Ok(size)
                        }
                    }
                }
            } else {
                quote! {
                    #[automatically_derived]
                    impl #zc_impl_generics CopyType<'info> for #name #ty_generics #where_clause {
                        type ZeroCopyType = #zc_name #zc_ty_generics;

                        fn byte_size_from_instance(&self) -> usize {
                            let mut size = 1;

                            match self {
                                #(#byte_size_from_instance_method),*
                            }

                            size
                        }
                    }

                    pub enum #zc_name #zc_ty_generics #zc_where_clause {
                        #(#zc_name_variants),*
                    }

                    #[automatically_derived]
                    impl #zc_impl_generics ZeroCopyType<'info> for #zc_name #zc_ty_generics #zc_where_clause {
                        fn new(info: &'info AccountInfo<'info>, offset: usize) -> FankorResult<(Self, Option<usize>)> {
                            let __offset = offset;
                            let bytes = info
                                .try_borrow_data()
                                .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock { type_name: std::any::type_name::<Self>() })?;
                            let bytes = &bytes[__offset..];

                            if bytes.is_empty() {
                                return Err(FankorErrorCode::ZeroCopyNotEnoughLength { type_name: std::any::type_name::<Self>() }.into());
                            }

                            let mut size = 1;
                            let flag = bytes[0];

                            let result = match flag {
                                #(#new_method,)*
                                _ => return Err(FankorErrorCode::ZeroCopyInvalidEnumDiscriminant { type_name: std::any::type_name::<Self>() }.into()),
                            };

                            Ok((result, Some(size)))
                        }

                        fn read_byte_size_from_bytes(bytes: &[u8]) -> FankorResult<usize> {
                            if bytes.is_empty() {
                                return Err(FankorErrorCode::ZeroCopyNotEnoughLength { type_name: std::any::type_name::<Self>() }.into());
                            }

                            let mut size = 1;
                            let flag = bytes[0];

                            match flag {
                                #(#read_byte_size_from_bytes_method,)*
                                _ => return Err(FankorErrorCode::ZeroCopyInvalidEnumDiscriminant { type_name: std::any::type_name::<Self>() }.into()),
                            }

                            Ok(size)
                        }
                    }
                }
            }
        }
        _ => {
            return Err(Error::new(
                input.span(),
                "account macro can only be applied to struct or enum declarations",
            ));
        }
    };

    Ok(result.into())
}

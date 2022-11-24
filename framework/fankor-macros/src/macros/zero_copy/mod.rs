use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{Error, Field, Fields, Item};

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
            let zc_name_refs = format_ident!("Zc{}Refs", name);
            let mut aux_zc_generics = item.generics.clone();
            aux_zc_generics
                .params
                .insert(0, syn::parse_quote! { 'info });

            let (zc_impl_generics, zc_ty_generics, zc_where_clause) =
                aux_zc_generics.split_for_impl();

            let mut zc_field_methods_aux = Vec::with_capacity(item.fields.len());
            let mut zc_unsafe_field_methods = Vec::with_capacity(item.fields.len());
            let mut last: Option<&Field> = None;
            let zc_field_methods = item.fields.iter().map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                let unsafe_field_name = format_ident!("{}_from_previous", field_name);
                let field_ty = &field.ty;

                let res = quote! {
                    pub fn #field_name(&self) -> FankorResult<Zc<'info, #field_ty>> {
                        let bytes = self.info.try_borrow_data()
                            .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock {
                                type_name: std::any::type_name::<Self>(),
                            })?;
                        let bytes = &bytes[self.offset..];
                        let mut size = 0;

                        #(#zc_field_methods_aux)*

                        Ok(unsafe { Zc::new(self.info, self.offset + size) })
                    }
                };

                if let Some(last ) = last {
                    let last_ty = &last.ty;

                    zc_unsafe_field_methods.push(quote! {
                        pub unsafe fn #unsafe_field_name(&self, last: Zc<'info, #last_ty>) -> FankorResult<Zc<'info, #field_ty>> {
                            let size = last.byte_size()?;

                            Ok(unsafe { Zc::new(last.info(), last.offset() + size) })
                        }
                    });
                }

                zc_field_methods_aux.push(quote! {
                    size += <#field_ty as CopyType>::ZeroCopyType::read_byte_size_from_bytes(&bytes[size..])?;
                });

                last = Some(field);
                res
            });

            let zc_name_refs_fields = item.fields.iter().map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                let field_ty = &field.ty;

                quote! {
                    pub #field_name: Zc<'info, #field_ty>,
                }
            });

            let to_refs_method = item.fields.iter().enumerate().map(|(i, field)| {
                let field_name = field.ident.as_ref().unwrap();
                let field_ty = &field.ty;

                if i + 1 == item.fields.len() {
                    quote! {
                        let #field_name = unsafe { Zc::new(self.info, self.offset + size) };
                    }
                } else {
                    quote! {
                        let #field_name = {
                            let zc = unsafe { Zc::new(self.info, self.offset + size) };

                            size += <#field_ty as CopyType>::ZeroCopyType::read_byte_size_from_bytes(&bytes[size..])?;

                            zc
                        };
                    }
                }
            });

            let to_refs_method_names = item.fields.iter().map(|field| {
                let field_name = field.ident.as_ref().unwrap();

                quote! {
                    #field_name
                }
            });

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

                // TODO process generics to add phantom data if needed
                pub struct #zc_name #zc_ty_generics #zc_where_clause {
                    info: &'info AccountInfo<'info>,
                    offset: usize,
                }

                pub struct #zc_name_refs #zc_ty_generics #zc_where_clause {
                    #(#zc_name_refs_fields)*
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
                    #(#zc_unsafe_field_methods)*

                    pub fn to_refs(&self) -> FankorResult<#zc_name_refs #zc_ty_generics> {
                        let bytes = self.info.try_borrow_data()
                            .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock {
                                type_name: std::any::type_name::<Self>(),
                            })?;
                        let bytes = &bytes[self.offset..];
                        let mut size = 0;

                        #(#to_refs_method)*

                        Ok(#zc_name_refs {
                            #(#to_refs_method_names),*
                        })
                    }
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
                                        let #field_name = unsafe { Zc::new(info, offset) };
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
                                        let #field_name = unsafe { Zc::new(info, offset) };
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
                quote! {
                    #[automatically_derived]
                    impl #zc_impl_generics CopyType<'info> for #name #ty_generics #where_clause {
                        type ZeroCopyType = #zc_name #ty_generics;

                        fn byte_size_from_instance(&self) -> usize {
                            1
                        }
                    }

                    pub enum #zc_name #ty_generics #where_clause {
                        #(#zc_name_variants),*
                    }

                    #[automatically_derived]
                    impl #zc_impl_generics ZeroCopyType<'info> for #zc_name #ty_generics #where_clause {
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
                                _ => return Err(FankorErrorCode::ZeroCopyInvalidEnumDiscriminator { type_name: std::any::type_name::<Self>() }.into()),
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
                                _ => return Err(FankorErrorCode::ZeroCopyInvalidEnumDiscriminator { type_name: std::any::type_name::<Self>() }.into()),
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
                                _ => return Err(FankorErrorCode::ZeroCopyInvalidEnumDiscriminator { type_name: std::any::type_name::<Self>() }.into()),
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
                                _ => return Err(FankorErrorCode::ZeroCopyInvalidEnumDiscriminator { type_name: std::any::type_name::<Self>() }.into()),
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

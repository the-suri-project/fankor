use convert_case::{Case, Converter};
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{Error, Fields, Item};

use crate::fnk_syn::FnkMetaArgumentList;
use crate::Result;

pub fn processor(input: Item) -> Result<proc_macro::TokenStream> {
    let result = match &input {
        Item::Struct(item) => {
            let name = &item.ident;
            let visibility = &item.vis;

            let (_, ty_generics, where_clause) = item.generics.split_for_impl();

            // Check for zero_copy attribute.
            let mut extra_offset = 0usize;

            for attr in &item.attrs {
                if attr.path.is_ident("fankor") {
                    if let Ok(mut args) = attr.parse_args::<FnkMetaArgumentList>() {
                        args.error_on_duplicated()?;

                        if args.pop_ident("account", true)?.is_some() {
                            extra_offset = 1;
                        }

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
                            "The correct pattern is #[zero_copy(<meta_list>)]",
                        ));
                    };
                    break;
                }
            }

            let byte_size_method = item.fields.iter().map(|field| {
                let field_name = &field.ident;

                quote! {
                    size += self.#field_name.byte_size();
                }
            });

            let min_byte_size_method = item.fields.iter().map(|field| {
                let field_type = &field.ty;

                quote! {
                    size += <#field_type>::min_byte_size();
                }
            });

            let read_byte_size_method = item.fields.iter().map(|field| {
                let field_ty = &field.ty;

                quote! {
                    size += <#field_ty as CopyType>::ZeroCopyType::read_byte_size(&bytes[size..])?;
                }
            });

            let zc_name = format_ident!("Zc{}", name);
            let fields_name = format_ident!("{}Fields", name);
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
                    #visibility fn #field_name(&self) -> FankorResult<Zc<'info, #field_ty>> {
                        let offset = self.offset + #extra_offset; // Account discriminant
                        let bytes = self.info.try_borrow_data()
                            .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock {
                                type_name: std::any::type_name::<Self>(),
                            })?;
                        let bytes = &bytes[offset..];
                        let mut size = 0;

                        #(#zc_field_methods_aux)*

                        Ok(Zc::new_unchecked(self.info, offset + size))
                    }
                };

                if !zc_from_previous_methods_lasts.is_empty() {
                    zc_from_previous_methods.push(quote! {
                        #visibility fn #from_previous_method_name(&self, previous: #fields_name, mut offset: usize) -> FankorResult<Zc<'info, #field_ty>> {
                            if previous == #fields_name::#pascal_field_name {
                                return Ok(Zc::new_unchecked(self.info, offset))
                            }

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
                    size += <#field_ty as CopyType>::ZeroCopyType::read_byte_size(&bytes[size..])?;
                });

                zc_from_previous_methods_lasts.push(quote! {
                    if processed || previous == #fields_name::#pascal_field_name {
                        offset += <#field_ty as CopyType>::ZeroCopyType::read_byte_size(&bytes[offset..])?;
                        processed = true;
                    }
                });

                res
            }).collect::<Vec<_>>();

            quote! {
                #[automatically_derived]
                impl #zc_impl_generics CopyType<'info> for #name #ty_generics #where_clause {
                    type ZeroCopyType = #zc_name #zc_ty_generics;

                    fn byte_size(&self) -> usize {
                        let mut size = #extra_offset; // Account discriminant
                        #(#byte_size_method)*
                        size
                    }

                    fn min_byte_size() -> usize {
                        let mut size = #extra_offset; // Account discriminant
                        #(#min_byte_size_method)*
                        size
                    }
                }

                #[allow(dead_code)]
                #[automatically_derived]
                #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
                #visibility enum #fields_name { #(#zc_field_names),* }

                // TODO process generics to add phantom data if needed
                #visibility struct #zc_name #zc_ty_generics #zc_where_clause {
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

                    fn read_byte_size(bytes: &[u8]) -> FankorResult<usize> {
                        let mut size = #extra_offset; // Account discriminant
                        #(#read_byte_size_method)*
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
            let discriminants_name = format_ident!("{}Discriminant", name);
            let visibility = &item.vis;
            let (_, ty_generics, where_clause) = item.generics.split_for_impl();

            let mut aux_zc_generics = item.generics.clone();
            aux_zc_generics
                .params
                .insert(0, syn::parse_quote! { 'info });

            let (zc_impl_generics, zc_ty_generics, zc_where_clause) =
                aux_zc_generics.split_for_impl();

            // Check for zero_copy attribute.
            let mut initial_size = 1usize;

            for attr in &item.attrs {
                if attr.path.is_ident("fankor") {
                    if let Ok(mut args) = attr.parse_args::<FnkMetaArgumentList>() {
                        args.error_on_duplicated()?;

                        if args.pop_plain("accounts", true)? {
                            initial_size = 0;
                        }

                        if args.pop_plain("account", true)? {
                            return Err(Error::new(
                                input.span(),
                                "Accounts cannot be used with an enum",
                            ));
                        }

                        args.error_on_unknown()?;
                    } else {
                        return Err(Error::new(
                            attr.span(),
                            "The correct pattern is #[zero_copy(<meta_list>)]",
                        ));
                    };
                    break;
                }
            }

            let mut min_byte_size_method = Vec::with_capacity(item.variants.len());
            let byte_size_method = item.variants.iter().map(|variant| {
                let variant_name = &variant.ident;

                match &variant.fields {
                    Fields::Named(named_fields) => {
                        let mut field_names = Vec::with_capacity(named_fields.named.len());
                        let fields = named_fields
                            .named
                            .iter()
                            .map(|field| {
                                let field_name = field.ident.as_ref().unwrap();

                                field_names.push(quote! {
                                    #field_name
                                });

                                quote! {
                                    size += #field_name.byte_size();
                                }
                            })
                            .collect::<Vec<_>>();
                        let min_fields = named_fields
                            .named
                            .iter()
                            .map(|field| {
                                let field_type = &field.ty;

                                quote! {
                                    variant_size += <#field_type as ::fankor::traits::CopyType>::min_byte_size();
                                }
                            })
                            .collect::<Vec<_>>();

                        min_byte_size_method.push(quote! {
                            {
                                let mut variant_size = 1;
                                #(#min_fields)*
                                size = size.min(variant_size);
                            }
                        });

                        quote! {
                            #name::#variant_name { #(#field_names),* } => {
                                #(#fields)*
                            }
                        }
                    }
                    Fields::Unnamed(unnamed_fields) => {
                        let mut field_names = Vec::with_capacity(unnamed_fields.unnamed.len());
                        let fields = unnamed_fields
                            .unnamed
                            .iter()
                            .enumerate()
                            .map(|(i, _)| {
                                let field_name = format_ident!("v{}", i);

                                field_names.push(quote! {
                                    #field_name
                                });

                                quote! {
                                    size += #field_name.byte_size();
                                }
                            })
                            .collect::<Vec<_>>();
                        let min_fields = unnamed_fields
                            .unnamed
                            .iter()
                            .map(|field| {
                                let field_type = &field.ty;

                                quote! {
                                    variant_size += <#field_type as ::fankor::traits::CopyType>::min_byte_size();
                                }
                            })
                            .collect::<Vec<_>>();

                        min_byte_size_method.push(quote! {
                            {
                                let mut variant_size = #initial_size;
                                #(#min_fields)*
                                size = size.min(variant_size);
                            }
                        });

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
            }).collect::<Vec<_>>();

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

            let mut variant_consts = Vec::with_capacity(item.variants.len());
            let new_method =
                item.variants
                    .iter()
                    .map(|variant| {
                        let variant_name = &variant.ident;
                        let variant_const_name = format_ident!("{}Const", variant_name);

                        variant_consts.push(quote! {
                            const #variant_const_name: u8 = #discriminants_name::#variant_name.code();
                        });

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
                                        size += <#field_ty as CopyType>::ZeroCopyType::read_byte_size(&bytes[size..])?;
                                    }
                                });

                                quote! {
                                    #variant_const_name => {
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
                                        size += <#field_ty as CopyType>::ZeroCopyType::read_byte_size(&bytes[size..])?;
                                    }
                                });

                                quote! {
                                    #variant_const_name => {
                                        #(#fields)*

                                        #zc_name::#variant_name(#(#field_names),*)
                                    }
                                }
                            }
                            Fields::Unit => {
                                quote! {
                                    #variant_const_name => #zc_name::#variant_name
                                }
                            }
                        }
                    }).collect::<Vec<_>>();

            let read_byte_size_method =
                item.variants
                    .iter()
                    .map(|variant| {
                        let variant_const_name = format_ident!("{}Const", variant.ident);

                        match &variant.fields {
                            Fields::Named(fields) => {
                                let fields = fields.named.iter().map(|field| {
                                    let field_ty = &field.ty;

                                    quote! {
                                        size += <#field_ty as CopyType>::ZeroCopyType::read_byte_size(&bytes[size..])?;
                                    }
                                });

                                quote! {
                                    #variant_const_name => {
                                        #(#fields)*
                                    }
                                }
                            }
                            Fields::Unnamed(fields) => {
                                let fields = fields.unnamed.iter().map(|field| {
                                    let field_ty = &field.ty;

                                    quote! {
                                        size += <#field_ty as CopyType>::ZeroCopyType::read_byte_size(&bytes[size..])?;
                                    }
                                });

                                quote! {
                                    #variant_const_name => {
                                        #(#fields)*
                                    }
                                }
                            }
                            Fields::Unit => {
                                quote! {
                                    #variant_const_name => {}
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

                        fn min_byte_size() -> usize {
                            1
                        }
                    }

                    #[automatically_derived]
                    #[allow(non_upper_case_globals)]
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

                            #(#variant_consts)*

                            let result = match flag {
                                #(#new_method,)*
                                _ => return Err(FankorErrorCode::ZeroCopyInvalidEnumDiscriminant { type_name: std::any::type_name::<Self>() }.into()),
                            };

                            Ok((result, Some(size)))
                        }

                        fn read_byte_size(bytes: &[u8]) -> FankorResult<usize> {
                            if bytes.is_empty() {
                                return Err(FankorErrorCode::ZeroCopyNotEnoughLength { type_name: std::any::type_name::<Self>() }.into());
                            }

                            let mut size = 1;
                            let flag = bytes[0];

                            #(#variant_consts)*

                            match flag {
                                #(#read_byte_size_method,)*
                                _ => return Err(FankorErrorCode::ZeroCopyInvalidEnumDiscriminant { type_name: std::any::type_name::<Self>() }.into()),
                            }

                            Ok(size)
                        }
                    }
                }
            } else {
                let min_byte_size_body = if min_byte_size_method.is_empty() {
                    quote! {
                        1
                    }
                } else {
                    quote! {
                        let mut size = usize::MAX;

                        #(#min_byte_size_method)*

                        size
                    }
                };
                
                quote! {
                    #[automatically_derived]
                    impl #zc_impl_generics CopyType<'info> for #name #ty_generics #where_clause {
                        type ZeroCopyType = #zc_name #zc_ty_generics;

                        fn byte_size(&self) -> usize {
                            let mut size = #initial_size;

                            match self {
                                #(#byte_size_method),*
                            }

                            size
                        }

                        fn min_byte_size() -> usize {
                            #min_byte_size_body
                        }
                    }

                    #visibility enum #zc_name #zc_ty_generics #zc_where_clause {
                        #(#zc_name_variants),*
                    }

                    #[automatically_derived]
                    #[allow(non_upper_case_globals)]
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

                            let mut size = #initial_size;
                            let flag = bytes[0];

                            #(#variant_consts)*

                            let result = match flag {
                                #(#new_method,)*
                                _ => return Err(FankorErrorCode::ZeroCopyInvalidEnumDiscriminant { type_name: std::any::type_name::<Self>() }.into()),
                            };

                            Ok((result, Some(size)))
                        }

                        fn read_byte_size(bytes: &[u8]) -> FankorResult<usize> {
                            if bytes.is_empty() {
                                return Err(FankorErrorCode::ZeroCopyNotEnoughLength { type_name: std::any::type_name::<Self>() }.into());
                            }

                            let mut size = #initial_size;
                            let flag = bytes[0];

                            #(#variant_consts)*

                            match flag {
                                #(#read_byte_size_method,)*
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

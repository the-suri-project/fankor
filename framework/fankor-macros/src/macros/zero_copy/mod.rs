use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{Error, Field, Item};

use fankor_syn::Result;

pub fn processor(input: Item) -> Result<proc_macro::TokenStream> {
    let result = match &input {
        Item::Struct(item) => {
            let name = &item.ident;
            let generics = &item.generics;

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
            let mut zc_generics = generics.clone();
            zc_generics.params.insert(0, syn::parse_quote! { 'info });

            let zc_generic_params = &zc_generics.params;
            let zc_generic_params = if zc_generic_params.is_empty() {
                quote! {}
            } else {
                quote! { < #zc_generic_params > }
            };

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
                                type_name: stringify!(Self),
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
                impl #zc_generic_params CopyType<'info> for #name #generics {
                    type ZeroCopyType = #zc_name<'info>;

                    fn byte_size_from_instance(&self) -> usize {
                        let mut size = 0;
                        #(#byte_size_from_instance_method)*
                        size
                    }
                }

                // TODO process generics to add phantom data if needed
                pub struct #zc_name<'info> {
                    info: &'info AccountInfo<'info>,
                    offset: usize,
                }

                pub struct #zc_name_refs #zc_generic_params {
                    #(#zc_name_refs_fields)*
                }

                #[automatically_derived]
                impl #zc_generic_params ZeroCopyType<'info> for #zc_name #zc_generics {
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
                impl #zc_generic_params #zc_name #zc_generics {
                    #(#zc_field_methods)*
                    #(#zc_unsafe_field_methods)*

                    pub fn to_refs(&self) -> FankorResult<#zc_name_refs #zc_generics> {
                        let bytes = self.info.try_borrow_data()
                            .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock {
                                type_name: stringify!(Self),
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
        Item::Enum(_item) => todo!(),
        _ => {
            return Err(Error::new(
                input.span(),
                "account macro can only be applied to struct or enum declarations",
            ));
        }
    };

    Ok(result.into())
}
